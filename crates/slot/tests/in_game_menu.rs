mod common;

use common::{app_playing_in, tmp_root_with_carts};
use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{Platform, Shader, StateRing};
use slot_ui::QuickRow;
use tempfile::TempDir;

fn playing() -> (TempDir, App) {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let a = app_playing_in(d.path(), "Emerald");
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "{:?}",
        a.phase()
    );
    (d, a)
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn go_to(a: &mut App, row: QuickRow) {
    a.apply(Action::QuickMenu);
    let path: &[QuickRow] = match row.page() {
        p if p == QuickRow::PLAY => &[],
        p if p == QuickRow::SETTINGS => &[QuickRow::Settings],
        p if p == QuickRow::PLAY_SCREEN => &[QuickRow::Settings, QuickRow::PlayScreen],
        _ => &[QuickRow::Settings, QuickRow::PlayGameplay],
    };
    for step in path.iter().copied().chain([row]) {
        for _ in 0..12 {
            if a.in_game_menu() == Some(step) {
                break;
            }
            press(a, Btn::Down);
        }
        assert_eq!(a.in_game_menu(), Some(step), "never reached {step:?}");
        if step != row {
            press(a, Btn::A);
        }
    }
}

#[test]
fn a_tap_of_menu_in_game_opens_on_resume_over_the_running_cart() {
    let (_d, mut a) = playing();
    a.apply(Action::QuickMenu);
    assert_eq!(a.in_game_menu(), Some(QuickRow::Resume));
    assert!(matches!(a.phase(), Phase::Playing { cart } if cart == "Emerald"));
    assert_eq!(a.quick_menu(), None);
}

#[test]
fn resume_and_b_each_close_it_back_into_the_game() {
    for close in [Btn::A, Btn::B] {
        let (_d, mut a) = playing();
        a.apply(Action::QuickMenu);
        press(&mut a, close);
        assert_eq!(a.in_game_menu(), None, "{close:?}");
        assert!(matches!(a.phase(), Phase::Playing { .. }));
    }
}

#[test]
fn a_tap_of_menu_closes_it_from_any_page() {
    let (_d, mut a) = playing();
    go_to(&mut a, QuickRow::Rumble);
    a.apply(Action::QuickMenu);
    assert_eq!(a.in_game_menu(), None);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn b_climbs_the_in_game_pages_and_never_the_shelf_ones() {
    let (_d, mut a) = playing();
    go_to(&mut a, QuickRow::Rumble);
    press(&mut a, Btn::B);
    assert_eq!(a.in_game_menu(), Some(QuickRow::PlayGameplay));
    press(&mut a, Btn::B);
    assert_eq!(a.in_game_menu(), Some(QuickRow::Settings));
    press(&mut a, Btn::B);
    assert_eq!(a.in_game_menu(), None);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn screen_shows_the_palette_only_when_palettes_are_live() {
    let (_d, mut a) = playing();
    assert_eq!(a.in_game_hidden(), &[QuickRow::Palette, QuickRow::Bezel]);
    a.set_palette_live(true);
    assert_eq!(a.in_game_hidden(), &[QuickRow::Bezel]);
}

#[test]
fn shader_edits_the_running_platforms_shader() {
    let (_d, mut a) = playing();
    let before = a.screen_shader();
    go_to(&mut a, QuickRow::Shader);
    press(&mut a, Btn::Right);
    assert_ne!(a.screen_shader(), before);
    assert_ne!(a.screen_shader(), Shader::Off);
    assert_eq!(
        a.quick_value(QuickRow::Shader),
        a.quick_value(QuickRow::GbaShader)
    );
}

#[test]
fn colour_correction_toggles_the_running_platform_and_asks_the_core() {
    let (_d, mut a) = playing();
    let before = a.colour_correction();
    go_to(&mut a, QuickRow::ColourCorrection);
    press(&mut a, Btn::Right);
    assert_eq!(a.colour_correction(), !before);
    assert_eq!(a.take_colour_correction(), Some(!before));
}

#[test]
fn palette_steps_the_palette_and_asks_the_core() {
    let (_d, mut a) = playing();
    a.set_palette_live(true);
    let before = a.gb_palette_label();
    go_to(&mut a, QuickRow::Palette);
    press(&mut a, Btn::Right);
    assert_ne!(a.gb_palette_label(), before);
    assert!(a.take_gb_palette().is_some());
}

#[test]
fn states_with_nothing_saved_is_refused_and_the_menu_stays() {
    let (_d, mut a) = playing();
    go_to(&mut a, QuickRow::States);
    press(&mut a, Btn::A);
    assert_eq!(a.in_game_menu(), Some(QuickRow::States));
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn states_opens_the_switcher_and_b_comes_back_to_the_menu() {
    let (_d, mut a) = playing();
    a.apply(Action::SaveState);
    go_to(&mut a, QuickRow::States);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
    press(&mut a, Btn::B);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.in_game_menu(), Some(QuickRow::States));
}

#[test]
fn loading_from_states_lands_in_the_game_with_no_menu() {
    let (_d, mut a) = playing();
    a.apply(Action::SaveState);
    go_to(&mut a, QuickRow::States);
    press(&mut a, Btn::A);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.in_game_menu(), None);
}

#[test]
fn a_double_tap_inside_the_menu_opens_states() {
    let (_d, mut a) = playing();
    a.apply(Action::SaveState);
    a.apply(Action::QuickMenu);
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
}

#[test]
fn a_double_tap_in_game_opens_the_switcher_and_b_goes_back_to_the_game() {
    let (_d, mut a) = playing();
    a.apply(Action::SaveState);
    a.apply(Action::QuickMenu);
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
    press(&mut a, Btn::B);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.in_game_menu(), None);
}

#[test]
fn a_refused_double_tap_in_game_leaves_the_game_playing() {
    let (_d, mut a) = playing();
    a.apply(Action::QuickMenu);
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.in_game_menu(), None);
}

#[test]
fn a_double_tap_inside_the_menu_comes_back_to_the_menu() {
    let (_d, mut a) = playing();
    a.apply(Action::SaveState);
    go_to(&mut a, QuickRow::Settings);
    a.apply(Action::QuickMenu);
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
    press(&mut a, Btn::B);
    assert_eq!(a.in_game_menu(), Some(QuickRow::States));
}

#[test]
fn dozing_in_states_wakes_back_to_the_menu_and_forgets_it_after() {
    let (_d, mut a) = playing();
    a.apply(Action::SaveState);
    go_to(&mut a, QuickRow::States);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
    a.apply(Action::PowerTap);
    assert!(matches!(a.phase(), Phase::Doze { .. }));
    a.apply(Action::PowerTap);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.in_game_menu(), Some(QuickRow::States));
    press(&mut a, Btn::B);
    for _ in 0..30 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::QuickMenu);
    a.apply(Action::Polaroids);
    press(&mut a, Btn::B);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.in_game_menu(), None);
}

#[test]
fn the_menu_waits_for_the_screen_to_finish_powering_on() {
    let (_d, mut a) = playing();
    a.apply(Action::QuickMenu);
    a.apply(Action::PowerTap);
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::PowerTap);
    a.update(1.0 / 60.0);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.in_game_menu(), Some(QuickRow::Resume));
    assert!(a.screen_power() < 1.0, "the screen was already fully on");
    let mut out = Vec::new();
    a.draw(&mut out);
    let scrim = out.iter().any(
        |d| matches!(d, slot_ui::Draw::Rect { colour, .. } if (colour[3] - 0.65).abs() < 0.001),
    );
    assert!(
        !scrim,
        "the menu drew over the slot while the screen powered on"
    );
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    let mut out = Vec::new();
    a.draw(&mut out);
    assert!(out.iter().any(|d| {
        matches!(d, slot_ui::Draw::Rect { colour, .. } if (colour[3] - 0.65).abs() < 0.001)
    }));
}

#[test]
fn exit_game_ejects_like_holding_menu() {
    let (_d, mut a) = playing();
    go_to(&mut a, QuickRow::ExitGame);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Ejecting { .. }));
    assert_eq!(a.in_game_menu(), None);
}

#[test]
fn holding_menu_with_the_menu_open_ejects_and_drops_it() {
    let (_d, mut a) = playing();
    a.apply(Action::QuickMenu);
    a.apply(Action::Eject);
    assert!(matches!(a.phase(), Phase::Ejecting { .. }));
    assert_eq!(a.in_game_menu(), None);
}

#[test]
fn play_chords_do_nothing_while_the_menu_is_open() {
    let (d, mut a) = playing();
    let shader = a.screen_shader();
    let colour = a.colour_correction();
    a.apply(Action::QuickMenu);
    a.apply(Action::SaveState);
    a.apply(Action::ShaderNext);
    a.apply(Action::ColourCorrectionToggle);
    assert_eq!(a.screen_shader(), shader);
    assert_eq!(a.colour_correction(), colour);
    let ring = StateRing::new(d.path(), Platform::Gba, a.core(), "Emerald");
    assert!(ring.list().unwrap_or_default().is_empty());
    assert_eq!(a.in_game_menu(), Some(QuickRow::Resume));
}

#[test]
fn the_shelf_menu_still_opens_from_the_shelf() {
    let (_d, mut a) = playing();
    a.apply(Action::Eject);
    for _ in 0..240 {
        a.update(1.0 / 60.0);
    }
    assert!(matches!(a.phase(), Phase::Shelf), "{:?}", a.phase());
    a.apply(Action::QuickMenu);
    assert_eq!(a.quick_menu(), Some(QuickRow::MAIN[0]));
    assert_eq!(a.in_game_menu(), None);
}

#[test]
fn exit_game_is_not_offered_with_only_one_cart() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::QuickMenu);
    let mut seen = Vec::new();
    for _ in 0..4 {
        seen.push(a.in_game_menu().expect("the menu closed"));
        press(&mut a, Btn::Down);
    }
    assert_eq!(
        seen,
        [
            QuickRow::Resume,
            QuickRow::States,
            QuickRow::Settings,
            QuickRow::Resume
        ]
    );
    assert!(a.in_game_hidden().contains(&QuickRow::ExitGame));
}

#[test]
fn exit_game_is_offered_with_two_carts() {
    let (_d, a) = playing();
    assert!(!a.in_game_hidden().contains(&QuickRow::ExitGame));
}
