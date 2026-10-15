#![cfg(target_os = "macos")]

mod common;

use std::collections::VecDeque;

use common::{clocked, tmp_root_with_carts};
use slot::frontend::Frontend;
use slot_gfx::{Canvas, Compositor, HeadlessSurface};
use slot_input::{InputSource, Millis, RawEvent};
use slot_power::SimPlatform;

struct Script(VecDeque<Vec<RawEvent>>);

impl InputSource for Script {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

fn write_png(name: &str, w: u32, h: u32, rgba: &[u8]) {
    let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") else {
        return;
    };
    let path = format!("{dir}/{name}.png");
    let file = std::fs::File::create(&path).expect("create png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .expect("png header")
        .write_image_data(rgba)
        .expect("png data");
}

#[test]
fn the_shelf_on_a_4_by_3_panel_fills_it_from_the_top_row() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Emerald", "Zzz"]);
    clocked(d.path());
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    f.upload_bezel(&mut c, (640, 480));
    let mut input = Script(VecDeque::new());
    for _ in 0..10 {
        f.advance(&mut input);
    }
    c.fit((640, 480));
    f.compose(&mut c);
    assert_eq!(
        c.frame_size(),
        (640, 480),
        "the shelf was not drawn at the panel size"
    );
    let frame = c.read_frame();
    write_png("native-shelf", 640, 480, &frame);
    let lit = frame[..640 * 4]
        .chunks(4)
        .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 30)
        .count();
    assert!(lit > 600, "the top row is black: {lit} of 640 pixels lit");
}

#[test]
fn a_host_window_resized_to_4_by_3_lays_the_shelf_out_for_it() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Emerald", "Zzz"]);
    clocked(d.path());
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    f.upload_bezel(&mut c, (720, 480));
    f.render(&mut c, (720, 480));
    assert_eq!(f.app().canvas(), Canvas::FRAMED);
    f.render(&mut c, (640, 480));
    assert_eq!(
        f.app().canvas(),
        Canvas { w: 640.0, h: 480.0 },
        "the shelf kept its 720 layout in a 4:3 window"
    );
    assert_eq!(c.frame_size(), (640, 480));
    f.render(&mut c, (1080, 720));
    assert_eq!(
        f.app().canvas(),
        Canvas::FRAMED,
        "a 3:2 window kept the 4:3 layout"
    );
}

#[test]
fn a_title_shows_under_the_cart_when_titles_are_on() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Pokemon - Emerald Version (USA, Europe)", "Zzz"]);
    clocked(d.path());
    let state = slot_store::read_slot_state(d.path());
    slot_store::write_slot_state(
        d.path(),
        &slot_store::SlotState {
            show_titles: true,
            ..state
        },
    )
    .unwrap();
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    f.upload_bezel(&mut c, (640, 480));
    let mut input = Script(VecDeque::new());
    for _ in 0..10 {
        f.advance(&mut input);
    }
    assert_eq!(
        f.app().title_text().as_deref(),
        Some("Pokemon Emerald Version")
    );
    c.fit((640, 480));
    f.compose(&mut c);
    let frame = c.read_frame();
    write_png("native-shelf-title", 640, 480, &frame);
    let y = slot_ui::slot_name_y() as usize + 8;
    let lit = (100..540)
        .filter(|&x| {
            let o = (y * 640 + x) * 4;
            frame[o] > 200 && frame[o + 1] > 200 && frame[o + 2] > 200
        })
        .count();
    assert!(lit > 20, "no title ink under the cart: {lit} white pixels");
}
