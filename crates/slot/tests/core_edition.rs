//! slotPlus Core: Ultra set up for you, with fewer rows. This file runs as Core
//! (`edition::run_as_core`), whichever edition the tests are built as; every test in it does.

mod common;

use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use slot::app::App;
use slot_input::{Action, Btn};
use slot_store::{
    read_controls, read_shortcuts, read_slot_state, write_slot_state, Appearance, BatteryDisplay,
    KidMode, Shader, Shortcut, SlotState, WallpaperTint,
};
use slot_ui::{QuickRow, QuickValue};
use tempfile::TempDir;

/// Which edition runs is one setting for the whole process, so these tests take turns.
static TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn as_edition(core: bool) -> std::sync::MutexGuard<'static, ()> {
    let turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    slot::edition::run_as_core(core);
    turn
}

fn as_core() -> std::sync::MutexGuard<'static, ()> {
    as_edition(true)
}

/// A card with a game for every system, so each Screen page shows.
fn card(state: SlotState) -> TempDir {
    let d = tmp_root_with_carts(&["Emerald"]);
    for (dir, file) in [
        ("GB", "Tetris.gb"),
        ("GBC", "Crystal.gbc"),
        ("WS", "Gunpey.ws"),
        ("Lynx", "Chips.lnx"),
        ("PokeMini", "Pinball.min"),
        ("GG", "Columns.gg"),
        ("NGP", "Puzzle.ngp"),
        ("TG16", "Bonk.pce"),
        ("NDS", "Kirby.nds"),
    ] {
        let dir = d.path().join("Games").join(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(file), vec![0u8; 0x8000]).unwrap();
    }
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..state
        },
    )
    .unwrap();
    d
}

fn boot(d: &TempDir) -> App {
    app_booting_at(d.path(), CLOCK_IS_SET).0
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn open_at(a: &mut App, row: QuickRow) {
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

fn shown(a: &App, page: QuickRow) -> Vec<QuickRow> {
    page.child().unwrap().shown(&a.hidden_rows())
}

#[test]
fn core_shows_only_the_rows_it_keeps() {
    let _turn = as_core();
    let d = card(SlotState::default());
    let a = boot(&d);
    let hidden = a.hidden_rows();
    assert_eq!(
        QuickRow::MAIN
            .into_iter()
            .filter(|r| !hidden.contains(r))
            .collect::<Vec<_>>(),
        QuickRow::MAIN,
        "every top-level page has something on it"
    );
    assert_eq!(shown(&a, QuickRow::General), [QuickRow::ShelfLifeSwitch]);
    assert_eq!(shown(&a, QuickRow::Network), [QuickRow::FetchArt]);
    assert_eq!(
        shown(&a, QuickRow::Game),
        [QuickRow::FastForward, QuickRow::Rewind]
    );
    assert_eq!(shown(&a, QuickRow::ScreenGeneral), [QuickRow::Brightness]);
    assert_eq!(
        shown(&a, QuickRow::ScreenGba),
        [QuickRow::GbaShader, QuickRow::GbaColourCorrection]
    );
    assert_eq!(
        shown(&a, QuickRow::ScreenGbc),
        [QuickRow::GbcShader, QuickRow::GbcColourCorrection]
    );
    assert_eq!(
        shown(&a, QuickRow::ScreenGb),
        [QuickRow::GbShader, QuickRow::GbPalette]
    );
    assert_eq!(shown(&a, QuickRow::ScreenPm), [QuickRow::PmShader]);
    assert_eq!(shown(&a, QuickRow::ScreenNds), [QuickRow::NdsShader]);
    assert_eq!(shown(&a, QuickRow::Controls), [QuickRow::ControlsInGame]);
    assert_eq!(
        shown(&a, QuickRow::Online),
        [QuickRow::Wifi, QuickRow::CheckUpdates]
    );
    assert_eq!(
        shown(&a, QuickRow::System),
        [
            QuickRow::Kid,
            QuickRow::DateTime,
            QuickRow::Nickname,
            QuickRow::About
        ]
    );
    assert_eq!(
        QuickRow::PLAY
            .into_iter()
            .filter(|r| !hidden.contains(r))
            .collect::<Vec<_>>(),
        [QuickRow::Resume, QuickRow::Settings, QuickRow::ExitGame]
    );
    assert!(
        !slot::achievements_on(),
        "RetroAchievements are out of Core"
    );
}

#[test]
fn a_card_from_ultra_starts_on_cores_own_settings() {
    let _turn = as_core();
    let d = card(SlotState {
        appearance: Appearance::Light,
        wallpaper_tint: WallpaperTint::High,
        ff_speed: 8,
        ff_sound: true,
        shader_gba: Shader::Grid,
        shader_gb: Shader::Simpletex,
        colour_correction_gba: false,
        show_recents: true,
        show_titles: false,
        bespoke_slot: false,
        kid_mode: KidMode::Shelves,
        battery_display: BatteryDisplay::Percent,
        brightness: 6,
        nickname: "Carlos".into(),
        dust: false,
        ..SlotState::default()
    });
    std::fs::write(d.path().join("Config/edition.txt"), "ultra").unwrap();
    let a = boot(&d);
    let s = read_slot_state(d.path());
    assert_eq!(
        std::fs::read_to_string(d.path().join("Config/edition.txt")).unwrap(),
        "core"
    );
    // Set for you.
    assert_eq!(s.appearance, Appearance::Dark);
    assert_eq!(s.wallpaper_tint, WallpaperTint::Off);
    assert!(!s.show_recents && s.show_titles && s.bespoke_slot);
    assert_eq!(s.battery_display, BatteryDisplay::Icon);
    assert_eq!(s.lid_off_min, Some(1));
    assert!(!s.ff_sound);
    // Where Core starts each row it shows.
    assert_eq!(s.ff_speed, 16, "4×");
    assert_eq!(s.kid_mode, KidMode::Off);
    assert!(s.shrink_wrap && s.dust && s.wear);
    assert_eq!(s.shader_gba, Shader::Lcd3x);
    assert_eq!(s.shader_gb, Shader::Transparent);
    assert!(s.colour_correction_gba && s.colour_correction_gbc);
    // What carries over.
    assert_eq!(s.brightness, 6);
    assert_eq!(s.nickname, "Carlos");
    // Core's shortcuts: Slot's own, less those it leaves out.
    let keys = read_shortcuts(d.path());
    assert!(keys.get(Shortcut::SaveState).is_some());
    for gone in [
        Shortcut::ShaderNext,
        Shortcut::ColorCorrection,
        Shortcut::BlueLightUp,
        Shortcut::Reset,
    ] {
        assert_eq!(keys.get(gone), None, "{gone:?}");
    }
    assert_eq!(read_controls(d.path()), slot_store::Controls::default());
    assert_eq!(a.quick_value(QuickRow::GbaShader), Some(QuickValue::Retro));
}

#[test]
fn a_core_card_keeps_its_choices_from_one_start_to_the_next() {
    let _turn = as_core();
    let d = card(SlotState::default());
    std::fs::write(d.path().join("Config/edition.txt"), "core").unwrap();
    let mut s = read_slot_state(d.path());
    s.ff_speed = 0;
    s.shader_ws = Shader::Shimmerless;
    s.kid_mode = KidMode::OneShelf;
    write_slot_state(d.path(), &s).unwrap();
    let a = boot(&d);
    let s = read_slot_state(d.path());
    assert_eq!(s.ff_speed, 0);
    assert_eq!(s.shader_ws, Shader::Shimmerless);
    assert_eq!(s.kid_mode, KidMode::OneShelf);
    assert_eq!(a.quick_value(QuickRow::WsShader), Some(QuickValue::Clean));
    assert_eq!(a.quick_value(QuickRow::FastForward), Some(QuickValue::Off));
}

#[test]
fn fast_forward_is_on_at_four_times_or_off() {
    let _turn = as_core();
    let d = card(SlotState::default());
    let mut a = boot(&d);
    open_at(&mut a, QuickRow::FastForward);
    assert_eq!(a.quick_value(QuickRow::FastForward), Some(QuickValue::On));
    press(&mut a, Btn::Right);
    assert_eq!(a.quick_value(QuickRow::FastForward), Some(QuickValue::Off));
    assert_eq!(read_slot_state(d.path()).ff_speed, 0);
    press(&mut a, Btn::Left);
    assert_eq!(read_slot_state(d.path()).ff_speed, 16, "4×");
}

#[test]
fn each_systems_shader_is_clean_or_retro() {
    let _turn = as_core();
    let d = card(SlotState::default());
    let mut a = boot(&d);
    open_at(&mut a, QuickRow::GbShader);
    assert_eq!(a.quick_value(QuickRow::GbShader), Some(QuickValue::Retro));
    press(&mut a, Btn::Right);
    assert_eq!(a.quick_value(QuickRow::GbShader), Some(QuickValue::Clean));
    assert_eq!(read_slot_state(d.path()).shader_gb, Shader::Shimmerless);
    press(&mut a, Btn::Right);
    assert_eq!(read_slot_state(d.path()).shader_gb, Shader::Transparent);
}

#[test]
fn clean_on_a_gba_on_a_three_by_two_screen_is_plain_pixels() {
    let _turn = as_core();
    let d = card(SlotState::default());
    let mut a = boot(&d);
    a.set_panel((720, 480));
    open_at(&mut a, QuickRow::GbaShader);
    press(&mut a, Btn::Right);
    assert_eq!(read_slot_state(d.path()).shader_gba, Shader::Off);
    assert_eq!(a.quick_value(QuickRow::GbaShader), Some(QuickValue::Clean));
    a.set_panel((640, 480));
    press(&mut a, Btn::Right);
    press(&mut a, Btn::Right);
    assert_eq!(read_slot_state(d.path()).shader_gba, Shader::Shimmerless);
}

#[test]
fn shelf_life_is_one_switch_and_kid_mode_is_one_shelf_or_off() {
    let _turn = as_core();
    let d = card(SlotState::default());
    let mut a = boot(&d);
    open_at(&mut a, QuickRow::ShelfLifeSwitch);
    press(&mut a, Btn::Right);
    let s = read_slot_state(d.path());
    assert!(!s.shrink_wrap && !s.dust && !s.wear);
    press(&mut a, Btn::Right);
    let s = read_slot_state(d.path());
    assert!(s.shrink_wrap && s.dust && s.wear);
    press(&mut a, Btn::B);
    press(&mut a, Btn::B);
    open_at(&mut a, QuickRow::KidMode);
    press(&mut a, Btn::Left);
    assert_eq!(
        read_slot_state(d.path()).kid_mode,
        KidMode::OneShelf,
        "never Shelves"
    );
}

#[test]
fn ultra_marks_the_card_and_starts_nothing_fresh() {
    let _turn = as_edition(false);
    let d = card(SlotState {
        ff_speed: 8,
        ..SlotState::default()
    });
    std::fs::write(d.path().join("Config/edition.txt"), "core").unwrap();
    let a = boot(&d);
    assert_eq!(
        std::fs::read_to_string(d.path().join("Config/edition.txt")).unwrap(),
        "ultra"
    );
    assert_eq!(read_slot_state(d.path()).ff_speed, 8);
    assert!(a.hidden_rows().contains(&QuickRow::ShelfLifeSwitch));
    assert!(!a.hidden_rows().contains(&QuickRow::ShelfLife));
}
