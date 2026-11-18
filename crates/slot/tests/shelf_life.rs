//! slotPlus Shelf Life: carts show how they've been kept, and the card remembers how long
//! each game has been played.

mod common;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use common::tmp_root_with_carts;
use slot::app::App;
use slot_input::{Action, Btn};
use slot_store::{
    read_play_log, write_play_log, write_slot_state, Platform, PlayLog, Recent, SlotState,
};
use slot_ui::shelf_life::Look;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn boot_with(root: &Path, state: SlotState) -> App {
    write_slot_state(
        root,
        &SlotState {
            clock_set: true,
            ..state
        },
    )
    .expect("write slot.state");
    App::boot(root)
}

fn look(a: &App) -> Look {
    let cart = a.selected_cart().expect("a cart on the shelf").clone();
    a.face_look(&cart)
}

fn selected(a: &App) -> (Platform, String) {
    let c = a.selected_cart().unwrap();
    (c.platform, c.stem.clone())
}

/// The play log is written off the main thread; wait for it to land.
fn landed(root: &Path, done: impl Fn(&PlayLog) -> bool) -> PlayLog {
    for _ in 0..200 {
        let log = read_play_log(root);
        if done(&log) {
            return log;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the play log never got there");
}

#[test]
fn a_game_never_played_comes_shrink_wrapped_and_one_played_does_not() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let a = boot_with(d.path(), SlotState::default());
    assert!(look(&a).wrap);
    let recent = Recent {
        platform: Platform::Gba,
        stem: "Emerald".into(),
    };
    let a = boot_with(
        d.path(),
        SlotState {
            recents: vec![recent],
            ..SlotState::default()
        },
    );
    assert!(!look(&a).wrap);
}

#[test]
fn shelf_life_can_be_switched_off() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let a = boot_with(
        d.path(),
        SlotState {
            shrink_wrap: false,
            ..SlotState::default()
        },
    );
    assert_eq!(look(&a), Look::default());
}

#[test]
fn the_card_notes_when_it_first_saw_each_game() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let _a = boot_with(d.path(), SlotState::default());
    let log = landed(d.path(), |l| l.get(Platform::Gba, "Fusion").is_some());
    let seen = log.record(Platform::Gba, "Emerald").seen.unwrap();
    assert!(now().abs_diff(seen) < 60);
    assert!(!log.record(Platform::Gba, "Emerald").played());
}

#[test]
fn a_cart_left_alone_gathers_dust_and_holding_b_wipes_it() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut log = PlayLog::default();
    log.add_play(Platform::Gba, "Emerald", 12 * 3600, now() - 100 * 86_400);
    write_play_log(d.path(), &log).unwrap();
    let mut a = boot_with(d.path(), SlotState::default());
    assert_eq!(
        look(&a),
        Look {
            wrap: false,
            dust: 3,
            wear: 2
        }
    );

    let t = a.now();
    a.apply_at(Action::GbaDown(Btn::B), t);
    a.tick_ms(t + 300);
    assert_eq!(look(&a).dust, 3, "a short press isn't a wipe");
    a.tick_ms(t + 700);
    assert_eq!(look(&a), Look::default());
    a.apply_at(Action::GbaUp(Btn::B), t + 800);

    let log = landed(d.path(), |l| {
        l.record(Platform::Gba, "Emerald").cleaned.is_some()
    });
    let r = log.record(Platform::Gba, "Emerald");
    assert_eq!((r.seconds, r.worn_seconds()), (12 * 3600, 0));
}

/// Putting a dusty cart in the slot blows the dust off: its face is dropped to be drawn
/// again clean, not kept dusty until something else redraws it.
#[test]
fn inserting_a_dusty_cart_redraws_it_clean() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut log = PlayLog::default();
    log.add_play(Platform::Gba, "Emerald", 3600, now() - 120 * 86_400);
    write_play_log(d.path(), &log).unwrap();
    let mut a = boot_with(d.path(), SlotState::default());
    assert_eq!(look(&a).dust, 3);
    let want = a.face_wants()[0];
    a.set_cart_face(want.shelf, want.index, slot_gfx::TexId::from_raw(1));
    assert!(a.shed_faces().is_empty());

    a.apply(Action::Insert);
    assert_eq!(look(&a).dust, 0, "the dust starts again from now");
    assert_eq!(
        a.shed_faces(),
        vec![slot_gfx::TexId::from_raw(1)],
        "the dusty face is dropped to be drawn again"
    );
}

/// Hold B on the selected cart long enough to wipe it.
fn hold_b(a: &mut App) {
    let t = a.now();
    a.apply_at(Action::GbaDown(Btn::B), t);
    a.tick_ms(t + 300);
    a.tick_ms(t + 700);
    a.apply_at(Action::GbaUp(Btn::B), t + 800);
}

/// A cart with 3 levels of dust and 2 of wear, booted with Clean Button Resets at `resets`.
fn dusty_and_worn(resets: slot_store::CleanResets) -> (tempfile::TempDir, App) {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut log = PlayLog::default();
    log.add_play(Platform::Gba, "Emerald", 12 * 3600, now() - 100 * 86_400);
    write_play_log(d.path(), &log).unwrap();
    let a = boot_with(
        d.path(),
        SlotState {
            clean_resets: resets,
            ..SlotState::default()
        },
    );
    assert_eq!((look(&a).dust, look(&a).wear), (3, 2));
    (d, a)
}

/// slotPlus: Clean Button Resets says what holding B takes off a cart.
#[test]
fn the_clean_button_takes_off_only_what_it_is_set_to() {
    use slot_store::CleanResets;
    for (resets, left) in [
        (CleanResets::Both, (0, 0)),
        (CleanResets::Wear, (3, 0)),
        (CleanResets::Dust, (0, 2)),
        (CleanResets::Disabled, (3, 2)),
    ] {
        let (_d, mut a) = dusty_and_worn(resets);
        hold_b(&mut a);
        assert_eq!((look(&a).dust, look(&a).wear), left, "{resets:?}");
    }
}

/// With Wear only, a cart that's dusty but not worn has nothing to wipe: B leaves it be.
#[test]
fn a_cart_with_nothing_the_button_takes_off_is_left_alone() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut log = PlayLog::default();
    log.add_play(Platform::Gba, "Emerald", 60, now() - 100 * 86_400);
    write_play_log(d.path(), &log).unwrap();
    let mut a = boot_with(
        d.path(),
        SlotState {
            clean_resets: slot_store::CleanResets::Wear,
            ..SlotState::default()
        },
    );
    assert_eq!((look(&a).dust, look(&a).wear), (3, 0));
    assert!(!a.clean_selected());
    hold_b(&mut a);
    assert_eq!(look(&a).dust, 3);
    assert!(read_play_log(d.path())
        .record(Platform::Gba, "Emerald")
        .cleaned
        .is_none());
}

#[test]
fn letting_go_of_b_early_leaves_the_dust() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut log = PlayLog::default();
    log.add_play(Platform::Gba, "Emerald", 60, now() - 50 * 86_400);
    write_play_log(d.path(), &log).unwrap();
    let mut a = boot_with(d.path(), SlotState::default());
    let t = a.now();
    a.apply_at(Action::GbaDown(Btn::B), t);
    a.apply_at(Action::GbaUp(Btn::B), t + 200);
    a.tick_ms(t + 900);
    assert_eq!(look(&a).dust, 2);
}

#[test]
fn play_time_adds_up_and_wears_the_cart() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = boot_with(d.path(), SlotState::default());
    let (platform, stem) = selected(&a);
    assert_eq!(a.play_seconds(a.selected_cart().unwrap()), None);
    a.add_play_time(platform, &stem, 4 * 3600);
    assert_eq!(a.play_seconds(a.selected_cart().unwrap()), Some(4 * 3600));
    assert_eq!(
        look(&a),
        Look {
            wrap: false,
            dust: 0,
            wear: 1
        }
    );
    landed(d.path(), |l| l.record(platform, &stem).seconds == 4 * 3600);
}

#[test]
fn a_cart_with_a_save_never_comes_wrapped() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let a = boot_with(d.path(), SlotState::default());
    let cart = a.selected_cart().unwrap().clone();
    let wrapped = Look {
        wrap: true,
        dust: 0,
        wear: 0,
    };
    let plain = || slot_ui::cart_face_with(&cart, None);
    let built = slot::cart_faces::looked(plain(), &cart, wrapped, d.path());
    assert_ne!(built.rgba, plain().rgba);
    std::fs::create_dir_all(d.path().join("Saves/GBA")).unwrap();
    std::fs::write(d.path().join("Saves/GBA/Emerald.sav"), [1u8; 16]).unwrap();
    let built = slot::cart_faces::looked(plain(), &cart, wrapped, d.path());
    assert_eq!(built.rgba, plain().rgba);
}

#[test]
fn a_wrapped_cart_has_its_film_torn_off_before_it_goes_in() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = boot_with(d.path(), SlotState::default());
    a.show_tears();
    assert!(look(&a).wrap);
    let first = selected(&a);

    let t = a.now();
    a.apply_at(Action::Insert, t);
    assert!(a.tearing(), "the film comes off first");
    assert_eq!(a.take_sfx(), Some(slot::audio::Sfx::WrapTear), "with a rip");
    assert!(a.tear_wants().is_some(), "and its film is asked for");
    assert!(!look(&a).wrap, "the cart under it is bare");
    assert!(a.seated_cart().is_none(), "the cart waits on the shelf");

    a.apply_at(Action::ShelfRight, t + 100);
    a.tick_ms(t + 200);
    assert_eq!(selected(&a), first, "the shelf holds still while it tears");
    assert!(a.tearing());

    a.tick_ms(t + slot::app::TEAR_MS + 10);
    assert!(!a.tearing());
    assert_eq!(
        a.seated_cart().map(|c| c.stem.as_str()),
        Some(first.1.as_str())
    );
}

#[test]
fn a_cart_without_film_goes_straight_in() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = boot_with(
        d.path(),
        SlotState {
            shrink_wrap: false,
            ..SlotState::default()
        },
    );
    a.show_tears();
    let t = a.now();
    a.apply_at(Action::Insert, t);
    assert!(!a.tearing());
    assert!(a.seated_cart().is_some());
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

/// Open Settings at `row`, walking down from the top of the menu.
fn open_at(a: &mut App, row: slot_ui::QuickRow) {
    a.apply(Action::QuickMenu);
    let mut path = vec![row];
    while let Some(parent) = path.last().and_then(|r| r.parent()) {
        path.push(parent);
    }
    for (depth, step) in path.iter().rev().enumerate() {
        if depth > 0 {
            press(a, Btn::A);
        }
        for _ in 0..step.position(&a.hidden_rows()) {
            press(a, Btn::Down);
        }
    }
    assert_eq!(a.quick_menu(), Some(row), "the bar never reached {row:?}");
}

/// Emerald dusty and worn, Fusion never played (so sealed), at Clean All Carts' prompt.
fn at_clean_all(resets: slot_store::CleanResets) -> (tempfile::TempDir, App) {
    let (d, mut a) = dusty_and_worn(resets);
    open_at(&mut a, slot_ui::QuickRow::CleanAllCarts);
    press(&mut a, Btn::A);
    assert!(a.clean_all_prompt(), "A on the row asks first");
    (d, a)
}

fn looks(a: &App) -> Vec<(String, Look)> {
    let mut v: Vec<_> = a
        .carts()
        .map(|c| (c.stem.clone(), a.face_look(c)))
        .collect();
    v.sort_by(|x, y| x.0.cmp(&y.0));
    v
}

#[test]
fn a_tap_of_b_or_a_backs_out_of_clean_all_carts_and_changes_nothing() {
    for btn in [Btn::B, Btn::A] {
        let (_d, mut a) = at_clean_all(slot_store::CleanResets::Both);
        let before = looks(&a);
        press(&mut a, btn);
        assert!(!a.clean_all_prompt(), "{btn:?} backs out");
        assert_eq!(looks(&a), before);
    }
}

#[test]
fn letting_go_before_five_seconds_starts_the_count_again() {
    let (_d, mut a) = at_clean_all(slot_store::CleanResets::Both);
    let before = looks(&a);
    let t = a.now();
    a.apply_at(Action::GbaDown(Btn::B), t);
    a.tick_ms(t + 100);
    assert_eq!(a.clean_all_countdown(), Some(5));
    a.tick_ms(t + 3_100);
    assert_eq!(a.clean_all_countdown(), Some(2));
    a.apply_at(Action::GbaUp(Btn::B), t + 3_200);
    assert!(a.clean_all_prompt(), "still asking");
    assert_eq!(a.clean_all_countdown(), None, "the count starts again");
    a.tick_ms(t + 9_000);
    assert_eq!(looks(&a), before, "nothing was cleaned");
}

/// Five seconds of B: every cart loses its dust and wear and comes out of its shrink wrap,
/// even with Clean Button Resets set to do nothing.
#[test]
fn holding_b_five_seconds_cleans_and_opens_every_cart() {
    let (d, mut a) = at_clean_all(slot_store::CleanResets::Disabled);
    let fusion = |a: &App| {
        let c = a.carts().find(|c| c.stem == "Fusion").unwrap().clone();
        a.face_look(&c)
    };
    assert!(fusion(&a).wrap, "Fusion starts sealed");
    let t = a.now();
    a.apply_at(Action::GbaDown(Btn::B), t);
    a.tick_ms(t + 4_900);
    assert!(a.clean_all_prompt(), "not yet");
    a.tick_ms(t + 5_000);
    assert!(!a.clean_all_prompt(), "done, and the prompt goes");
    a.apply_at(Action::GbaUp(Btn::B), t + 5_100);
    for (stem, l) in looks(&a) {
        assert_eq!(l, Look::default(), "{stem} is clean and open");
    }
    let log = landed(d.path(), |l| {
        l.record(Platform::Gba, "Fusion").opened.is_some()
    });
    let fusion_record = log.record(Platform::Gba, "Fusion");
    assert!(!fusion_record.played(), "opening isn't playing");
    let emerald = log.record(Platform::Gba, "Emerald");
    assert_eq!(
        (emerald.seconds, emerald.worn_seconds()),
        (12 * 3600, 0),
        "the play time stays; the wear goes"
    );
}
