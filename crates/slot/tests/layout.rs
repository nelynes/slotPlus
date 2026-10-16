use slot::app::{App, Phase, IN_GAME_MENU};
use slot::video_mode::VideoMode;
use slot_gfx::{Canvas, Draw, Layout};
use slot_input::Action;
use slot_store::{Cart, Platform};

const PANEL_4_3: (u32, u32) = (640, 480);
const NATIVE: Canvas = Canvas { w: 640.0, h: 480.0 };

fn app_with_carts(stems: &[&str], panel: (u32, u32)) -> App {
    app_on(Platform::Gba, stems, panel)
}

fn app_on(platform: Platform, stems: &[&str], panel: (u32, u32)) -> App {
    let mut a = App::new(
        stems
            .iter()
            .map(|stem| Cart {
                platform,
                stem: (*stem).to_string(),
                rom: format!("Games/{}/{stem}.rom", platform.dir_name()).into(),
                label: None,
                code: String::new(),
                shell: None,
                title: stem.to_uppercase(),
            })
            .collect(),
    );
    a.set_panel(panel);
    a
}

fn tick(a: &mut App) {
    a.update(1.0 / 60.0);
}

fn playing(panel: (u32, u32)) -> App {
    let mut a = app_with_carts(&["Emerald", "Zzz"], panel);
    a.apply(Action::Insert);
    a.on_core_ready();
    for _ in 0..120 {
        tick(&mut a);
    }
    a
}

#[test]
fn the_shelf_is_native_and_a_game_is_framed_on_a_4_by_3_panel() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], PANEL_4_3);
    assert_eq!(a.layout(), Layout::Native);
    assert_eq!(a.canvas(), NATIVE);
    a.apply(Action::Insert);
    assert_eq!(
        a.layout(),
        Layout::Native,
        "the cart going in left the shelf"
    );
    let a = playing(PANEL_4_3);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.layout(), Layout::Framed);
    assert_eq!(a.canvas(), Canvas::FRAMED);
}

#[test]
fn the_in_game_menu_takes_its_own_layout_once_it_shows() {
    let mut a = playing(PANEL_4_3);
    a.apply(Action::QuickMenu);
    assert_eq!(
        a.layout(),
        Layout::Framed,
        "the overlay left before the menu showed"
    );
    for _ in 0..30 {
        tick(&mut a);
    }
    assert_eq!(a.layout(), IN_GAME_MENU);
    assert_eq!(IN_GAME_MENU, Layout::Native);
}

#[test]
fn the_photos_and_the_power_menu_over_a_game_stay_framed() {
    let mut a = playing(PANEL_4_3);
    a.apply(Action::PowerHold);
    assert!(a.power_menu().is_some(), "the power menu did not open");
    assert_eq!(a.layout(), Layout::Framed, "the power menu over a game");
    let mut a = playing(PANEL_4_3);
    a.apply(Action::Polaroids);
    if matches!(a.phase(), Phase::Polaroids { .. }) {
        assert_eq!(a.layout(), Layout::Framed, "the save state photos");
    }
}

#[test]
fn the_shelf_menu_and_the_power_menu_over_the_shelf_stay_native() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], PANEL_4_3);
    a.apply(Action::QuickMenu);
    assert!(matches!(a.phase(), Phase::QuickMenu { .. }));
    assert_eq!(a.layout(), Layout::Native);
    let mut a = app_with_carts(&["Emerald", "Zzz"], PANEL_4_3);
    a.apply(Action::PowerHold);
    assert!(a.power_menu().is_some());
    assert_eq!(a.layout(), Layout::Native);
}

#[test]
fn every_screen_is_the_framed_canvas_on_a_720_panel() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], (720, 480));
    assert_eq!(a.canvas(), Canvas::FRAMED);
    a.apply(Action::QuickMenu);
    assert_eq!(a.canvas(), Canvas::FRAMED);
    let mut a = playing((720, 480));
    assert_eq!(a.canvas(), Canvas::FRAMED);
    a.apply(Action::QuickMenu);
    for _ in 0..30 {
        tick(&mut a);
    }
    assert_eq!(
        a.canvas(),
        Canvas::FRAMED,
        "the in-game menu on a 720 panel"
    );
}

#[test]
fn the_overlay_rises_while_the_screen_powers_on() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], PANEL_4_3);
    a.apply(Action::Insert);
    a.on_core_ready();
    while !matches!(a.phase(), Phase::Playing { .. }) {
        assert_eq!(a.overlay_slide(), 0.0, "the overlay rose before the game");
        tick(&mut a);
    }
    let mut frames = 0;
    while a.overlay_slide() < 1.0 {
        tick(&mut a);
        frames += 1;
        assert!(
            frames <= 16,
            "the overlay took more than the power-on to rise"
        );
    }
    assert!(
        frames >= 12,
        "the overlay rose in {frames} frames, faster than the screen"
    );
}

#[test]
fn the_exit_powers_off_then_drops_the_overlay_then_goes_native() {
    let mut a = playing(PANEL_4_3);
    a.apply(Action::Eject);
    while a.screen_power() > 0.0 {
        assert_eq!(
            a.overlay_slide(),
            1.0,
            "the overlay moved before the screen was off"
        );
        assert_eq!(a.layout(), Layout::Framed);
        tick(&mut a);
    }
    let mut frames = 0;
    while a.overlay_slide() > 0.0 {
        assert_eq!(
            a.layout(),
            Layout::Framed,
            "the shelf showed under a sliding overlay"
        );
        tick(&mut a);
        frames += 1;
        assert!(frames <= 16, "the overlay took longer than it took to rise");
    }
    assert!(frames >= 12, "the overlay dropped in {frames} frames");
    assert_eq!(a.layout(), Layout::Native);
    assert!(
        matches!(a.phase(), Phase::Ejecting { .. }),
        "the cart left before the shelf showed"
    );
}

#[test]
fn the_framed_area_is_black_behind_the_game_as_it_powers_on_a_4_by_3_panel() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], PANEL_4_3);
    a.apply(Action::Insert);
    a.on_core_ready();
    while !matches!(a.phase(), Phase::Playing { .. }) {
        tick(&mut a);
    }
    tick(&mut a);
    assert!(a.screen_power() < 1.0);
    let mut out = Vec::new();
    a.draw(&mut out);
    assert!(
        matches!(out.first(),
            Some(Draw::Rect { x, y, w, h, colour: [0.0, 0.0, 0.0, 1.0] })
                if *x == 0.0 && *y == 0.0 && *w == 720.0 && *h == 480.0),
        "{:?}",
        out.first()
    );
    assert!(
        !out.iter()
            .any(|d| matches!(d, Draw::Tex { .. } | Draw::Part { .. })),
        "the shelf shows behind the game"
    );
}

#[test]
fn the_shelf_still_shows_behind_the_power_on_on_a_720_panel() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], (720, 480));
    a.set_ground(slot_gfx::TexId::from_raw(900));
    a.apply(Action::Insert);
    a.on_core_ready();
    while !matches!(a.phase(), Phase::Playing { .. }) {
        tick(&mut a);
    }
    tick(&mut a);
    let mut out = Vec::new();
    a.draw(&mut out);
    assert!(
        out.iter()
            .any(|d| matches!(d, Draw::Part { tex, .. } if *tex == slot_gfx::TexId::from_raw(900))),
        "the 720 power-on lost the wallpaper behind it"
    );
}

fn eject_until_shelf(a: &mut App, mut each: impl FnMut(&App)) {
    a.apply(Action::Eject);
    for _ in 0..240 {
        each(a);
        if matches!(a.phase(), Phase::Shelf) {
            return;
        }
        tick(a);
    }
    panic!("the cart never came out");
}

#[test]
fn ejecting_a_cart_that_is_still_going_in_keeps_the_shelf_native() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], PANEL_4_3);
    a.apply(Action::Insert);
    for _ in 0..10 {
        tick(&mut a);
    }
    assert!(matches!(a.phase(), Phase::Inserting { .. }));
    eject_until_shelf(&mut a, |a| {
        assert_eq!(
            a.overlay_slide(),
            0.0,
            "the overlay rose over a cart going in"
        );
        assert_eq!(a.layout(), Layout::Native, "the shelf went framed");
    });
}

#[test]
fn exit_from_the_in_game_menu_never_brings_the_overlay_back() {
    let mut a = playing(PANEL_4_3);
    a.apply(Action::QuickMenu);
    for _ in 0..30 {
        tick(&mut a);
    }
    assert_eq!(a.layout(), Layout::Native);
    let mut powering = 0;
    eject_until_shelf(&mut a, |a| {
        assert_eq!(
            a.overlay_slide(),
            0.0,
            "the overlay popped in on the way out"
        );
        assert_eq!(a.layout(), Layout::Native);
        if matches!(a.phase(), Phase::Ejecting { .. }) && a.screen_power() > 0.0 {
            powering += 1;
            let mut out = Vec::new();
            a.draw(&mut out);
            assert!(
                matches!(
                    out.first(),
                    Some(Draw::Rect {
                        colour: [0.0, 0.0, 0.0, 1.0],
                        ..
                    })
                ),
                "the game powered off over something other than black"
            );
            assert!(
                !out.iter().any(|d| matches!(d, Draw::Part { .. })),
                "the shelf showed behind the game powering off"
            );
        }
    });
    assert!(powering > 0, "the screen never powered off");
}

#[test]
fn ejecting_while_the_overlay_rises_never_raises_it_further() {
    let mut a = app_with_carts(&["Emerald", "Zzz"], PANEL_4_3);
    a.apply(Action::Insert);
    a.on_core_ready();
    while !matches!(a.phase(), Phase::Playing { .. }) {
        tick(&mut a);
    }
    for _ in 0..5 {
        tick(&mut a);
    }
    let mut high = a.overlay_slide();
    assert!(high > 0.0 && high < 1.0, "not mid-rise: {high}");
    eject_until_shelf(&mut a, |a| {
        let now = a.overlay_slide();
        assert!(
            now <= high + 1e-6,
            "the overlay jumped from {high} to {now}"
        );
        high = now;
    });
}

fn playing_on(platform: Platform, panel: (u32, u32)) -> App {
    let mut a = app_on(platform, &["Game", "Zzz"], panel);
    a.set_platform(platform);
    a.apply(Action::Insert);
    a.on_core_ready();
    for _ in 0..120 {
        tick(&mut a);
    }
    a
}

#[test]
fn a_stretched_game_fills_a_4_by_3_panel_but_the_game_boy_keeps_its_window() {
    for platform in [
        Platform::Gba,
        Platform::Ws,
        Platform::Lynx,
        Platform::Gg,
        Platform::Ngp,
    ] {
        let mut a = playing_on(platform, PANEL_4_3);
        assert!(matches!(a.phase(), Phase::Playing { .. }), "{platform:?}");
        assert!(!a.game_fills_panel(), "{platform:?} filled at actual size");
        assert_eq!(a.layout(), Layout::Framed, "{platform:?}");
        a.set_video_mode(VideoMode::Stretch);
        assert!(a.game_fills_panel(), "{platform:?} did not fill the panel");
        assert_eq!(a.layout(), Layout::Native, "{platform:?}");
        assert_eq!(a.canvas(), NATIVE, "{platform:?}");
    }
    for platform in [Platform::Gb, Platform::Gbc] {
        let mut a = playing_on(platform, PANEL_4_3);
        a.set_video_mode(VideoMode::Stretch);
        assert!(!a.game_fills_panel(), "{platform:?} left its 3:2 window");
        assert_eq!(a.layout(), Layout::Framed, "{platform:?}");
    }
}

#[test]
fn a_stretched_game_on_a_720_panel_needs_nothing_more() {
    let mut a = playing_on(Platform::Gg, (720, 480));
    a.set_video_mode(VideoMode::Stretch);
    assert!(!a.game_fills_panel());
    assert_eq!(a.canvas(), Canvas::FRAMED);
}
