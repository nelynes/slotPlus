use std::path::Path;
use std::sync::OnceLock;

use slot_gfx::{Canvas, Draw, TexId, OUT_H, OUT_W};
use slot_store::Platform;

use crate::art;

const DEFAULT_PNG: &[u8] = include_bytes!("../../../assets/background_default.png");
const BEZEL_GB_PNG: &[u8] = include_bytes!("../../../assets/bezel_gb.png");
const BEZEL_GBC_PNG: &[u8] = include_bytes!("../../../assets/bezel_gbc.png");
// slotPlus's own: the Game Gear's drawn by `assets/src/bezels.py`, the
// NeoGeo Pocket's made by hand.
const BEZEL_GG_PNG: &[u8] = include_bytes!("../../../assets/bezel_gg.png");
const BEZEL_NGP_PNG: &[u8] = include_bytes!("../../../assets/bezel_ngp.png");
// The TurboGrafx-16's, made by hand: the TurboGrafx-16 logo, and the PC
// Engine's own white case, each with a Wide one for games that draw 320
// wide, whose picture spans 640 of the screen.
const BEZEL_TG16_PNG: &[u8] = include_bytes!("../../../assets/bezel_tg16.png");
const BEZEL_TG16_WIDE_PNG: &[u8] = include_bytes!("../../../assets/bezel_tg16_wide.png");
const BEZEL_PCE_PNG: &[u8] = include_bytes!("../../../assets/bezel_pce.png");
const BEZEL_PCE_WIDE_PNG: &[u8] = include_bytes!("../../../assets/bezel_pce_wide.png");
const BLUR_PNG: &[u8] = include_bytes!("../../../assets/background_default_blur.png");

const SCRIM: f32 = 0.62;

pub fn wallpaper_face(path: &Path) -> Option<Vec<u8>> {
    art::cover(path, OUT_W, OUT_H)
}

pub fn bezel_face(path: &Path, w: u32, h: u32) -> Option<Vec<u8>> {
    art::cover(path, w, h)
}

pub fn game_bezel_face(path: &Path) -> Option<Vec<u8>> {
    art::cover(path, OUT_W, OUT_H)
}

/// The bezel bundled for a platform, for the platforms that have one.
pub fn default_game_bezel_face(platform: Platform) -> Option<Vec<u8>> {
    let png = match platform {
        Platform::Gb => BEZEL_GB_PNG,
        Platform::Gbc => BEZEL_GBC_PNG,
        Platform::Gg => BEZEL_GG_PNG,
        Platform::Ngp => BEZEL_NGP_PNG,
        _ => return None,
    };
    art::cover_bytes(png, OUT_W, OUT_H)
}

/// The bezels bundled for a platform under names of their own, in the order
/// the Bezel row offers them, with their art.
fn named_game_bezels(platform: Platform) -> &'static [(&'static str, &'static [u8])] {
    match platform {
        Platform::Tg16 => &[
            ("TG16", BEZEL_TG16_PNG),
            ("TG16 Wide", BEZEL_TG16_WIDE_PNG),
            ("PCE", BEZEL_PCE_PNG),
            ("PCE Wide", BEZEL_PCE_WIDE_PNG),
        ],
        _ => &[],
    }
}

/// The names of the bezels bundled for a platform under names of their own.
pub fn bundled_game_bezels(platform: Platform) -> Vec<&'static str> {
    named_game_bezels(platform)
        .iter()
        .map(|(name, _)| *name)
        .collect()
}

/// A bezel bundled for a platform under a name of its own.
pub fn bundled_game_bezel_face(platform: Platform, name: &str) -> Option<Vec<u8>> {
    let (_, png) = named_game_bezels(platform)
        .iter()
        .find(|(n, _)| *n == name)?;
    art::cover_bytes(png, OUT_W, OUT_H)
}

/// Whether a platform has a bezel bundled for it, offered as Default.
pub fn has_default_game_bezel(platform: Platform) -> bool {
    matches!(
        platform,
        Platform::Gb | Platform::Gbc | Platform::Gg | Platform::Ngp
    )
}

/// The carousel ground, scaled to cover the whole screen, including behind the footer.
pub fn default_background() -> Option<&'static [u8]> {
    static P: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    P.get_or_init(|| art::cover_bytes(DEFAULT_PNG, OUT_W, OUT_H))
        .as_deref()
}

/// The blurred carousel ground, scaled to cover the whole screen the same way as the default.
pub fn blur_background() -> Option<&'static [u8]> {
    static P: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    P.get_or_init(|| art::cover_bytes(BLUR_PNG, OUT_W, OUT_H))
        .as_deref()
}

pub fn draw_ground(canvas: Canvas, face: Option<TexId>, out: &mut Vec<Draw>) {
    let Some(tex) = face else {
        return;
    };
    out.push(ground(canvas, tex));
}

pub fn draw_backdrop(canvas: Canvas, face: Option<TexId>, out: &mut Vec<Draw>) {
    let Some(tex) = face else {
        return;
    };
    out.push(ground(canvas, tex));
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: canvas.w,
        h: OUT_H as f32,
        colour: [0.0, 0.0, 0.0, SCRIM],
    });
}

fn ground(canvas: Canvas, tex: TexId) -> Draw {
    Draw::Part {
        x: 0.0,
        y: 0.0,
        w: canvas.w,
        h: OUT_H as f32,
        tex,
        alpha: 1.0,
        uv: canvas.window(OUT_W as f32),
    }
}
