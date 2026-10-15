#![cfg(target_os = "macos")]

use slot_gfx::{Canvas, Compositor, Draw, HeadlessSurface, Layout, ScreenEffect, SRC_H, SRC_W};
use std::sync::{Mutex, MutexGuard, PoisonError};

static GL: Mutex<()> = Mutex::new(());

fn compositor() -> Option<(MutexGuard<'static, ()>, HeadlessSurface, Compositor)> {
    let guard = GL.lock().unwrap_or_else(PoisonError::into_inner);
    let surface = HeadlessSurface::new().ok()?;
    let compositor = Compositor::new(&surface).ok()?;
    Some((guard, surface, compositor))
}

fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ *b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

fn pattern() -> Vec<u8> {
    (0..SRC_W * SRC_H)
        .flat_map(|i| {
            let (x, y) = (i % SRC_W, i / SRC_W);
            [(x * 7) as u8, (y * 11) as u8, ((x + y) * 3) as u8, 255]
        })
        .collect()
}

fn framed_frame(c: &mut Compositor) -> Vec<u8> {
    c.fit((640, 480));
    c.set_screen_effect(ScreenEffect::Lcd3x);
    c.set_screen_power(1.0);
    let badge = c.create_texture(2, 2, &[200, 40, 40, 255].repeat(4));
    c.begin_frame(Layout::Framed);
    c.upload_game(&pattern());
    c.draw_list(&[
        Draw::Game,
        Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: 720.0,
            h: 40.0,
            colour: [0.0, 0.0, 0.0, 0.5],
        },
        Draw::Tex {
            x: 690.0,
            y: 10.0,
            w: 20.0,
            h: 20.0,
            tex: badge,
            alpha: 1.0,
        },
    ]);
    c.read_frame()
}

const FRAMED_HASH: u64 = 0xe3b9_652d_e7d2_eb8c;

#[test]
fn a_framed_frame_on_a_4_by_3_panel_is_todays_frame() {
    let Some((_g, _s, mut c)) = compositor() else {
        return;
    };
    let frame = framed_frame(&mut c);
    assert_eq!(frame.len(), 640 * 427 * 4);
    assert_eq!(fnv(&frame), FRAMED_HASH, "hash is {:#x}", fnv(&frame));
}

#[test]
fn a_native_canvas_on_a_4_by_3_panel_is_the_panel_at_480_lines() {
    assert_eq!(
        Canvas::of(Layout::Native, (640, 480)),
        Canvas { w: 640.0, h: 480.0 }
    );
    assert_eq!(
        Canvas::of(Layout::Native, (1280, 960)),
        Canvas { w: 640.0, h: 480.0 }
    );
    assert_eq!(Canvas::of(Layout::Framed, (640, 480)), Canvas::FRAMED);
}

#[test]
fn every_layout_is_the_framed_canvas_on_a_panel_that_is_not_4_by_3() {
    for panel in [(720, 480), (1440, 960), (0, 0)] {
        assert_eq!(Canvas::of(Layout::Native, panel), Canvas::FRAMED);
        assert_eq!(Canvas::of(Layout::Framed, panel), Canvas::FRAMED);
    }
}

#[test]
fn the_window_of_a_wider_texture_is_its_centre_and_a_fitting_one_is_whole() {
    let native = Canvas { w: 640.0, h: 480.0 };
    assert_eq!(
        native.window(720.0),
        [40.0 / 720.0, 0.0, 640.0 / 720.0, 1.0]
    );
    assert_eq!(Canvas::FRAMED.window(720.0), [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(native.centred(100.0), 270.0);
}

#[test]
fn a_part_draws_only_its_window_of_the_texture() {
    let Some((_g, _s, mut c)) = compositor() else {
        return;
    };
    let mut rgba = Vec::new();
    for _y in 0..2 {
        rgba.extend_from_slice(&[255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]);
    }
    let tex = c.create_texture_nearest(3, 2, &rgba);
    c.fit((720, 480));
    c.begin_frame(Layout::Framed);
    c.draw_list(&[Draw::Part {
        x: 0.0,
        y: 0.0,
        w: 720.0,
        h: 480.0,
        tex,
        alpha: 1.0,
        uv: [1.0 / 3.0, 0.0, 1.0 / 3.0, 1.0],
    }]);
    let frame = c.read_frame();
    let at = |x: usize, y: usize| {
        let o = (y * 720 + x) * 4;
        [frame[o], frame[o + 1], frame[o + 2]]
    };
    assert_eq!(
        at(10, 240),
        [0, 255, 0],
        "the left of the part is not the middle texel"
    );
    assert_eq!(
        at(710, 240),
        [0, 255, 0],
        "the right of the part is not the middle texel"
    );
}

#[test]
fn a_native_frame_on_a_4_by_3_panel_is_the_whole_panel() {
    let Some((_g, _s, mut c)) = compositor() else {
        return;
    };
    c.fit((640, 480));
    c.begin_frame(Layout::Native);
    c.draw_list(&[Draw::Rect {
        x: 630.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
        colour: [1.0, 1.0, 1.0, 1.0],
    }]);
    assert_eq!(c.frame_size(), (640, 480));
    let frame = c.read_frame();
    let o = (5 * 640 + 635) * 4;
    assert_eq!(
        &frame[o..o + 3],
        &[255, 255, 255],
        "x 635 is not 1:1 in a native frame"
    );
}

#[test]
fn a_framed_frame_after_a_native_one_is_todays_frame_again() {
    let Some((_g, _s, mut c)) = compositor() else {
        return;
    };
    c.fit((640, 480));
    c.begin_frame(Layout::Native);
    let frame = framed_frame(&mut c);
    assert_eq!(fnv(&frame), FRAMED_HASH);
}

fn game_only(c: &mut Compositor, layout: Layout) -> Vec<u8> {
    c.fit((640, 480));
    c.set_screen_effect(ScreenEffect::Lcd3x);
    c.set_screen_power(1.0);
    c.begin_frame(layout);
    c.upload_game(&pattern());
    c.draw_list(&[Draw::Game]);
    c.read_frame()
}

#[test]
fn a_game_in_a_native_frame_sits_where_the_framed_one_does() {
    let Some((_g, _s, mut c)) = compositor() else {
        return;
    };
    let framed = game_only(&mut c, Layout::Framed);
    let native = game_only(&mut c, Layout::Native);
    assert_eq!(native.len(), 640 * 480 * 4);
    let row = 640 * 4;
    let worst = native[..427 * row]
        .iter()
        .zip(&framed)
        .map(|(a, b)| a.abs_diff(*b))
        .max();
    assert!(
        worst.unwrap_or(0) <= 1,
        "the paused game moved or rescaled under a native frame: off by {worst:?}"
    );
    assert!(
        native[427 * row..].chunks(4).all(|p| p[..3] == [0, 0, 0]),
        "the strip under the game is not clear"
    );
}

fn strip_bezel(c: &mut Compositor) {
    let mut rgba = vec![0u8; 640 * 480 * 4];
    for y in 427..480 {
        for x in 0..640 {
            let o = (y * 640 + x) * 4;
            rgba[o..o + 4].copy_from_slice(&[220, 30, 30, 255]);
        }
    }
    c.set_bezel(640, 480, &rgba);
}

fn red_rows(panel: &[u8]) -> Vec<usize> {
    (0..480)
        .filter(|y| {
            let o = (y * 640 + 320) * 4;
            panel[o] > 200 && panel[o + 1] < 60
        })
        .collect()
}

#[test]
fn a_native_frame_reaches_the_panel_one_to_one() {
    let Some((_g, _s, mut c)) = compositor() else {
        return;
    };
    c.fit((640, 480));
    strip_bezel(&mut c);
    c.set_overlay_slide(1.0);
    c.begin_frame(Layout::Native);
    c.draw_list(&[Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: 640.0,
        h: 1.0,
        colour: [1.0, 1.0, 1.0, 1.0],
    }]);
    let canvas = c.read_frame();
    let panel = c.read_window((640, 480));
    assert_eq!(
        panel, canvas,
        "the native frame was moved or scaled on its way out"
    );
    assert!(
        red_rows(&panel).is_empty(),
        "the overlay drew over a native frame"
    );
}

#[test]
fn the_overlay_fills_the_strip_when_up_and_drops_as_it_slides() {
    let Some((_g, _s, mut c)) = compositor() else {
        return;
    };
    c.fit((640, 480));
    strip_bezel(&mut c);
    for (slide, top) in [(1.0, Some(427)), (0.5, Some(453)), (0.0, None)] {
        c.set_overlay_slide(slide);
        c.begin_frame(Layout::Framed);
        let panel = c.read_window((640, 480));
        let rows = red_rows(&panel);
        match top {
            Some(top) => {
                let first = *rows.first().expect("no overlay on the panel");
                assert!(
                    first.abs_diff(top) <= 1,
                    "at slide {slide} the overlay starts at row {first}, not {top}"
                );
                assert_eq!(
                    *rows.last().unwrap(),
                    479,
                    "the overlay does not reach the bottom"
                );
            }
            None => assert!(rows.is_empty(), "the overlay shows at slide 0"),
        }
    }
}
