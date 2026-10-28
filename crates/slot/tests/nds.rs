//! A DS game's views of its two screens, and its stylus.

mod common;

use slot::app::App;
use slot::video_mode::{DsLayout, DsScreen, DsView};
use slot_input::{Action, Btn};
use slot_store::{Platform, Shader};

fn playing_ds() -> (tempfile::TempDir, App) {
    playing_ds_with(Shader::Off)
}

fn playing_ds_with(shader: Shader) -> (tempfile::TempDir, App) {
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
            shader_nds: shader,
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
fn a_ds_game_starts_on_both_screens_and_l2_swaps_which_is_on_top() {
    let (d, mut a) = playing_ds();
    assert_eq!(a.ds_view().layout, DsLayout::Stacked);
    let (main, other) = a.ds_parts();
    assert_eq!(main.screen, DsScreen::Top);
    assert_eq!(main.dst, [200.0, 0.0, 320.0, 240.0], "1.25x, on top");
    assert_eq!(other.unwrap().dst, [200.0, 240.0, 320.0, 240.0]);
    assert_eq!(a.source_rect(), DsScreen::Top.source_rect());
    assert_eq!(a.game_layout().rect, main.dst);
    a.swap_ds_screens();
    let (main, other) = a.ds_parts();
    assert_eq!(main.screen, DsScreen::Bottom);
    assert_eq!(main.dst[1], 0.0, "the bottom screen now on top");
    assert_eq!(other.unwrap().screen, DsScreen::Top);
    assert_eq!(
        slot::video_mode::ds_view_for(d.path(), "Kart").main,
        DsScreen::Bottom,
        "the game remembers its main screen"
    );
}

#[test]
fn select_l2_steps_through_the_layouts_and_shaders_show_only_at_2x() {
    let (d, mut a) = playing_ds_with(Shader::Lcd3x);
    let mut seen = Vec::new();
    for _ in 0..DsLayout::ALL.len() {
        let (main, other) = a.ds_parts();
        seen.push((
            a.ds_view().layout,
            main.dst,
            other.map(|o| o.dst),
            a.drawn_shader(),
        ));
        a.next_ds_layout();
    }
    assert_eq!(a.ds_view().layout, DsLayout::Stacked, "and round again");
    let plain = Shader::Off;
    let lcd = Shader::Lcd3x;
    assert_eq!(
        seen,
        vec![
            (
                DsLayout::Stacked,
                [200.0, 0.0, 320.0, 240.0],
                Some([200.0, 240.0, 320.0, 240.0]),
                plain
            ),
            (DsLayout::Single, [104.0, 48.0, 512.0, 384.0], None, lcd),
            (DsLayout::Fill, [40.0, 0.0, 640.0, 480.0], None, plain),
            (
                DsLayout::Side,
                [0.0, 48.0, 512.0, 384.0],
                Some([512.0, 48.0, 208.0, 156.0]),
                lcd
            ),
            (
                DsLayout::Corner,
                [40.0, 0.0, 640.0, 480.0],
                Some([424.0, 288.0, 256.0, 192.0]),
                plain
            ),
        ]
    );
    a.next_ds_layout();
    assert_eq!(
        slot::video_mode::ds_view_for(d.path(), "Kart").layout,
        DsLayout::Single,
        "the game remembers its layout"
    );
}

#[test]
fn the_views_saved_before_layouts_still_read() {
    let view = |layout, main| Some(DsView { main, layout });
    assert_eq!(
        DsView::parse("both"),
        view(DsLayout::Stacked, DsScreen::Top)
    );
    assert_eq!(DsView::parse("top"), view(DsLayout::Single, DsScreen::Top));
    assert_eq!(
        DsView::parse("bottom"),
        view(DsLayout::Single, DsScreen::Bottom)
    );
    for layout in DsLayout::ALL {
        for main in [DsScreen::Top, DsScreen::Bottom] {
            let v = DsView { main, layout };
            assert_eq!(DsView::parse(&v.as_str()), Some(v));
        }
    }
    assert_eq!(DsView::parse("sideways,top"), None);
}

#[test]
fn the_stylus_moves_with_the_d_pad_touches_with_a_and_shows_the_bottom_screen() {
    let (_d, mut a) = playing_ds();
    a.next_ds_layout();
    assert_eq!(a.ds_view().layout, DsLayout::Single);
    assert_eq!(a.source_rect(), DsScreen::Top.source_rect());
    assert_eq!(a.pointer(), None, "no stylus until R2");
    a.stylus_out(true);
    assert_eq!(
        a.source_rect(),
        DsScreen::Bottom.source_rect(),
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
    assert_eq!(
        a.source_rect(),
        DsScreen::Top.source_rect(),
        "back to the top"
    );
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

#[test]
fn a_ds_game_offers_no_rewind() {
    let (_d, a) = playing_ds();
    let hidden = a.in_game_hidden();
    assert!(hidden.contains(&slot_ui::QuickRow::Rewind));
    assert!(hidden.contains(&slot_ui::QuickRow::ScRewind));
}
