use std::path::Path;
use std::sync::OnceLock;

use slot_gfx::{Canvas, Draw, TexId, OUT_H, OUT_W};
use slot_store::Platform;

use crate::art;

const DEFAULT_PNG: &[u8] = include_bytes!("../../../assets/background_default.png");
const BEZEL_GB_PNG: &[u8] = include_bytes!("../../../assets/bezel_gb.png");
const BEZEL_GBC_PNG: &[u8] = include_bytes!("../../../assets/bezel_gbc.png");
const BEZEL_GB_FIT_3X2_PNG: &[u8] = include_bytes!("../../../assets/bezel_gb_fit_3x2.png");
const BEZEL_GBC_FIT_3X2_PNG: &[u8] = include_bytes!("../../../assets/bezel_gbc_fit_3x2.png");
const BEZEL_GB_INTEGER_4X3_PNG: &[u8] = include_bytes!("../../../assets/bezel_gb_integer_4x3.png");
const BEZEL_GBC_INTEGER_4X3_PNG: &[u8] =
    include_bytes!("../../../assets/bezel_gbc_integer_4x3.png");
const BEZEL_GB_FIT_4X3_PNG: &[u8] = include_bytes!("../../../assets/bezel_gb_fit_4x3.png");
const BEZEL_GBC_FIT_4X3_PNG: &[u8] = include_bytes!("../../../assets/bezel_gbc_fit_4x3.png");
// slotPlus's own: the Game Gear's drawn by `assets/src/bezels.py`, the
// NeoGeo Pocket's made by hand.
const BEZEL_GG_PNG: &[u8] = include_bytes!("../../../assets/bezel_gg.png");
const BEZEL_NGP_PNG: &[u8] = include_bytes!("../../../assets/bezel_ngp.png");
// The TurboGrafx-16's, made by hand: the TurboGrafx-16 logo, and the PC
// Engine's own white case, each with a Wide one for games that draw 320
// wide, whose picture spans 640 of the screen.
const BEZEL_TG16_PNG: &[u8] = include_bytes!("../../../assets/bezel_tg16.png");
// The DS Lite set, by nelynes: one bezel for each of the DS's screen layouts.
const BEZEL_NDS_LITE: [(&str, &[u8]); 5] = [
    (
        "both-screens.png",
        include_bytes!("../../../assets/bezel_nds_lite_both_screens.png"),
    ),
    (
        "one-screen.png",
        include_bytes!("../../../assets/bezel_nds_lite_one_screen.png"),
    ),
    (
        "full-screen.png",
        include_bytes!("../../../assets/bezel_nds_lite_full_screen.png"),
    ),
    (
        "side-by-side.png",
        include_bytes!("../../../assets/bezel_nds_lite_side_by_side.png"),
    ),
    (
        "picture-in-picture.png",
        include_bytes!("../../../assets/bezel_nds_lite_picture_in_picture.png"),
    ),
];
/// The name the DS Lite set is offered under, and the DS's default bezel.
pub const DS_LITE_BEZEL: &str = "DS Lite";
const BEZEL_TG16_WIDE_PNG: &[u8] = include_bytes!("../../../assets/bezel_tg16_wide.png");
const BEZEL_PCE_PNG: &[u8] = include_bytes!("../../../assets/bezel_pce.png");
const BEZEL_PCE_WIDE_PNG: &[u8] = include_bytes!("../../../assets/bezel_pce_wide.png");
const BLUR_PNG: &[u8] = include_bytes!("../../../assets/background_default_blur.png");

const SCRIM: f32 = 0.62;

pub fn wallpaper_face(path: &Path, w: u32, h: u32) -> Option<Vec<u8>> {
    art::cover(path, w, h)
}

pub fn bezel_face(path: &Path, w: u32, h: u32) -> Option<Vec<u8>> {
    art::cover(path, w, h)
}

pub fn game_bezel_face(path: &Path, w: u32, h: u32) -> Option<Vec<u8>> {
    let (src, sw, sh) = art::decode(path)?;
    panel_bezel(&src, (sw, sh), (w, h), false)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BezelArt {
    Default,
    Fit3x2,
    Integer4x3,
    Fit4x3,
}

/// The bezel bundled for a platform, for the platforms that have one, sized to a `w` x `h` panel
/// the way Slot sizes the Game Boy's. The Game Boy and Game Boy Color have one for each of Slot's
/// pictures; slotPlus's Game Gear and NeoGeo Pocket have one, framed as Slot's default.
pub fn default_game_bezel_face(
    platform: Platform,
    kind: BezelArt,
    w: u32,
    h: u32,
) -> Option<Vec<u8>> {
    let (png, kind) = match (platform, kind) {
        (Platform::Gb, BezelArt::Default) => (BEZEL_GB_PNG, kind),
        (Platform::Gbc, BezelArt::Default) => (BEZEL_GBC_PNG, kind),
        (Platform::Gb, BezelArt::Fit3x2) => (BEZEL_GB_FIT_3X2_PNG, kind),
        (Platform::Gbc, BezelArt::Fit3x2) => (BEZEL_GBC_FIT_3X2_PNG, kind),
        (Platform::Gb, BezelArt::Integer4x3) => (BEZEL_GB_INTEGER_4X3_PNG, kind),
        (Platform::Gbc, BezelArt::Integer4x3) => (BEZEL_GBC_INTEGER_4X3_PNG, kind),
        (Platform::Gb, BezelArt::Fit4x3) => (BEZEL_GB_FIT_4X3_PNG, kind),
        (Platform::Gbc, BezelArt::Fit4x3) => (BEZEL_GBC_FIT_4X3_PNG, kind),
        (Platform::Gg, _) => (BEZEL_GG_PNG, BezelArt::Default),
        (Platform::Ngp, _) => (BEZEL_NGP_PNG, BezelArt::Default),
        _ => return None,
    };
    let (src, sw, sh) = art::decode_png_bytes(png)?;
    match kind {
        BezelArt::Default => panel_bezel(&src, (sw, sh), (w, h), true),
        _ => art::cover_rgba(&src, sw, sh, w, h),
    }
}

const BEZEL_GRAIN_ROWS: usize = 10;

fn panel_bezel(
    src: &[u8],
    (sw, sh): (u32, u32),
    (w, h): (u32, u32),
    extend: bool,
) -> Option<Vec<u8>> {
    if !slot_gfx::framed((w, h)) || sw * 3 == sh * 4 {
        return art::cover_rgba(src, sw, sh, w, h);
    }
    let game_h = slot_gfx::screen_rect((w, h)).3 as usize;
    let inner = art::cover_rgba(src, sw, sh, w, game_h as u32)?;
    let (h, row) = (h as usize, w as usize * 4);
    let gap = h - game_h;
    let top = gap - gap / 2;
    let mut out = vec![0u8; row * h];
    out[top * row..(top + game_h) * row].copy_from_slice(&inner);
    if !extend {
        return Some(out);
    }
    let ping = |d: usize| {
        let p = d % (2 * BEZEL_GRAIN_ROWS);
        p.min(2 * BEZEL_GRAIN_ROWS - 1 - p)
    };
    for d in 0..top {
        let from = ping(d) * row;
        out[(top - 1 - d) * row..][..row].copy_from_slice(&inner[from..from + row]);
    }
    for d in 0..h - top - game_h {
        let from = (game_h - 1 - ping(d)) * row;
        out[(top + game_h + d) * row..][..row].copy_from_slice(&inner[from..from + row]);
    }
    Some(out)
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
        Platform::Nds => &[(DS_LITE_BEZEL, BEZEL_NDS_LITE[0].1)],
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
pub fn bundled_game_bezel_face(platform: Platform, name: &str, w: u32, h: u32) -> Option<Vec<u8>> {
    bundled_game_bezel_face_in(platform, name, None, w, h)
}

/// A bundled bezel's face, and for a DS bezel set the one for the layout on
/// screen (`layout`, the set's PNG name for it).
pub fn bundled_game_bezel_face_in(
    platform: Platform,
    name: &str,
    layout: Option<&str>,
    w: u32,
    h: u32,
) -> Option<Vec<u8>> {
    if platform == Platform::Nds && name == DS_LITE_BEZEL {
        let layout = layout?;
        let (_, png) = BEZEL_NDS_LITE.iter().find(|(f, _)| *f == layout)?;
        let (src, sw, sh) = art::decode_png_bytes(png)?;
        return panel_bezel(&src, (sw, sh), (w, h), false);
    }
    let (_, png) = named_game_bezels(platform)
        .iter()
        .find(|(n, _)| *n == name)?;
    let (src, sw, sh) = art::decode_png_bytes(png)?;
    panel_bezel(&src, (sw, sh), (w, h), true)
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

pub fn draw_ground(canvas: Canvas, face: Option<(TexId, u32)>, out: &mut Vec<Draw>) {
    let Some(face) = face else {
        return;
    };
    out.push(ground(canvas, face));
}

pub fn draw_backdrop(canvas: Canvas, face: Option<(TexId, u32)>, out: &mut Vec<Draw>) {
    let Some(face) = face else {
        return;
    };
    out.push(ground(canvas, face));
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: canvas.w,
        h: OUT_H as f32,
        colour: [0.0, 0.0, 0.0, SCRIM],
    });
}

fn ground(canvas: Canvas, (tex, tex_w): (TexId, u32)) -> Draw {
    Draw::Part {
        x: 0.0,
        y: 0.0,
        w: canvas.w,
        h: OUT_H as f32,
        tex,
        alpha: 1.0,
        uv: canvas.cover(tex_w as f32),
    }
}
