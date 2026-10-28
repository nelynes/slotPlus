//! A DS game's views of its two screens, and its stylus.

mod common;

use slot::app::App;
use slot::video_mode::DsView;
use slot_input::{Action, Btn};
use slot_store::Platform;

fn playing_ds() -> (tempfile::TempDir, App) {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    let nds = d.path().join("Games/NDS");
    std::fs::create_dir_all(&nds).unwrap();
    std::fs::write(nds.join("Kart.nds"), vec![0u8; 0x10000]).unwrap();
    slot_store::write_slot_state(
        d.path(),
        &slot_store::SlotState {
            cart: Some("Kart".into()),
            cart_platform: Some(Platform::Nds),
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    a.set_snapshot(common::StubSnapshot::boxed());
    a.set_core(slot_store::Core::Dsperate);
    a.on_core_ready();
    a.set_game_ready(true);
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert_eq!(a.playing_platform(), Platform::Nds);
    assert!(
        a.ds_controls(),
        "{:?} visible {}",
        a.phase(),
        a.game_visible()
    );
    (d, a)
}

#[test]
fn a_ds_game_shows_both_screens_until_l2_steps_to_one_then_the_other() {
    let (d, mut a) = playing_ds();
    assert!(a.ds_controls());
    assert_eq!(a.ds_view(), DsView::Both);
    assert_eq!(a.source_rect(), [0.1, 0.1, 0.8, 0.8], "both, at 1.25x");
    a.next_ds_view();
    assert_eq!(
        a.source_rect(),
        [0.25, 0.05, 0.5, 0.5],
        "the top screen at 2x"
    );
    a.next_ds_view();
    assert_eq!(a.source_rect(), [0.25, 0.45, 0.5, 0.5], "the bottom screen");
    assert_eq!(
        slot::video_mode::ds_view_for(d.path(), "Kart"),
        DsView::Bottom,
        "the game remembers its view"
    );
    a.next_ds_view();
    assert_eq!(a.ds_view(), DsView::Both, "and round again");
}

#[test]
fn the_stylus_moves_with_the_d_pad_touches_with_a_and_shows_the_bottom_screen() {
    let (_d, mut a) = playing_ds();
    a.next_ds_view();
    assert_eq!(a.ds_view(), DsView::Top);
    assert_eq!(a.pointer(), None, "no stylus until R2");
    a.stylus_out(true);
    assert_eq!(
        a.source_rect(),
        [0.25, 0.45, 0.5, 0.5],
        "the stylus brings the bottom screen up"
    );
    let at = a.pointer().unwrap();
    assert!(!at.pressed);
    assert!(a.stylus_takes(Btn::Right) && a.stylus_takes(Btn::A));
    assert!(!a.stylus_takes(Btn::B));
    a.stylus_press(Btn::Right, true);
    for _ in 0..30 {
        a.update(1.0 / 60.0);
    }
    a.stylus_press(Btn::Right, false);
    a.stylus_press(Btn::A, true);
    let moved = a.pointer().unwrap();
    assert!(moved.x > at.x, "moved right: {} to {}", at.x, moved.x);
    assert_eq!(moved.y, at.y);
    assert!(moved.pressed, "A touches");
    a.stylus_out(false);
    assert_eq!(a.pointer(), None);
    assert_eq!(a.source_rect(), [0.25, 0.05, 0.5, 0.5], "back to the top");
    a.stylus_out(true);
    let back = a.pointer().unwrap();
    assert_eq!(
        (back.x, back.y),
        (moved.x, moved.y),
        "it comes back where it was left"
    );
    assert!(!back.pressed);
}

#[test]
fn opening_the_menu_puts_the_stylus_away() {
    let (_d, mut a) = playing_ds();
    a.stylus_out(true);
    a.apply(Action::QuickMenu);
    for _ in 0..10 {
        a.update(1.0 / 60.0);
    }
    assert!(!a.ds_controls());
    assert_eq!(a.pointer(), None);
}

#[test]
fn a_ds_game_records_no_rewind_and_keeps_x_and_y_as_buttons() {
    let (_d, a) = playing_ds();
    assert!(!a.records_rewind());
    assert!(a.ws_buttons(), "X and Y are the DS's own");
}
