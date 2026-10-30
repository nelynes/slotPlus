use slot_store::gb::Class;
use slot_store::{Cart, Outline, Platform};

use crate::art;
use crate::bitmaps::{self, Mould};
use crate::board::{BOARD_H, BOARD_W};
use crate::shell::{shell_for, Finish, Shell};
use crate::silhouette::{cart_mask, extra_mask, gb_cart_mask, GbShell};
use crate::text;

pub const CART_W: u32 = 360;
pub const CART_H: u32 = (CART_W * SEATED_H + SEATED_W / 2) / SEATED_W;

pub const GB_CART_W: u32 = 312;

pub const GB_CART_H: u32 = (GB_CART_W * GB_SEATED_H + SEATED_W / 2) / SEATED_W;

/// The GBA cart's label box, as drawn in `assets/gba_neutral.png`: x 48..312, y 50..186
/// of 360x212.
pub const fn label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 133 + 500) / 1000,
        (h * 236 + 500) / 1000,
        (w * 867 + 500) / 1000,
        (h * 877 + 500) / 1000,
    )
}

pub const LABEL_WELL: [u8; 3] = [0x8d, 0x8d, 0x90];

pub const LABEL_X: u32 = label_panel(CART_W, CART_H).0;
pub const LABEL_Y: u32 = label_panel(CART_W, CART_H).1;
pub const LABEL_W: u32 = label_panel(CART_W, CART_H).2 - LABEL_X;
pub const LABEL_H: u32 = label_panel(CART_W, CART_H).3 - LABEL_Y;

pub const fn gb_label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 147 + 500) / 1000,
        (h * 291 + 500) / 1000,
        (w * 854 + 500) / 1000,
        (h * 868 + 500) / 1000,
    )
}

pub const GB_LABEL_X: u32 = gb_label_panel(GB_CART_W, GB_CART_H).0;
pub const GB_LABEL_Y: u32 = gb_label_panel(GB_CART_W, GB_CART_H).1;
pub const GB_LABEL_W: u32 = gb_label_panel(GB_CART_W, GB_CART_H).2 - GB_LABEL_X;
pub const GB_LABEL_H: u32 = gb_label_panel(GB_CART_W, GB_CART_H).3 - GB_LABEL_Y;

/// The Game Boy Color pak's label box: exactly the flat label well drawn in
/// `assets/gbc_neutral.png`, x 45..267, y 110..308 of 312x337. It is the same 223x199
/// well as the Game Boy's (x 44..266, y 95..293), 1px right and 15px lower.
pub const GBC_LABEL_X: u32 = 45;
pub const GBC_LABEL_Y: u32 = 110;
pub const GBC_LABEL_W: u32 = 223;
pub const GBC_LABEL_H: u32 = 199;

// WonderSwan: the label fills the face between the oval nameplate at the top
// and the shoulder above the exposed contacts.
pub use crate::bitmaps::{LYNX_CART_H, LYNX_CART_W, WS_CART_H, WS_CART_W};

pub const fn ws_label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 40 + 500) / 1000,
        (h * 140 + 500) / 1000,
        (w * 960 + 500) / 1000,
        (h * 765 + 500) / 1000,
    )
}

pub const WS_LABEL_X: u32 = ws_label_panel(WS_CART_W, WS_CART_H).0;
pub const WS_LABEL_Y: u32 = ws_label_panel(WS_CART_W, WS_CART_H).1;
pub const WS_LABEL_W: u32 = ws_label_panel(WS_CART_W, WS_CART_H).2 - WS_LABEL_X;
pub const WS_LABEL_H: u32 = ws_label_panel(WS_CART_W, WS_CART_H).3 - WS_LABEL_Y;

// Atari Lynx (curved-lip card): the label runs nearly edge to edge below the
// lip, down to the clear strip over the contacts.
pub const fn lynx_label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 45 + 500) / 1000,
        (h * 255 + 500) / 1000,
        (w * 955 + 500) / 1000,
        (h * 835 + 500) / 1000,
    )
}

pub const LYNX_LABEL_X: u32 = lynx_label_panel(LYNX_CART_W, LYNX_CART_H).0;
pub const LYNX_LABEL_Y: u32 = lynx_label_panel(LYNX_CART_W, LYNX_CART_H).1;
pub const LYNX_LABEL_W: u32 = lynx_label_panel(LYNX_CART_W, LYNX_CART_H).2 - LYNX_LABEL_X;
pub const LYNX_LABEL_H: u32 = lynx_label_panel(LYNX_CART_W, LYNX_CART_H).3 - LYNX_LABEL_Y;

// Game Gear: a nearly square pak. A raised hood with grip ridges runs over
// the top and down the sides to the flanges, the label sits in the U it
// leaves, and the SEGA plaque sits at the foot.
pub use crate::bitmaps::{GG_CART_H, GG_CART_W};

pub const fn gg_label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 88 + 500) / 1000,
        (h * 145 + 500) / 1000,
        (w * 912 + 500) / 1000,
        (h * 755 + 500) / 1000,
    )
}

pub const GG_LABEL_X: u32 = gg_label_panel(GG_CART_W, GG_CART_H).0;
pub const GG_LABEL_Y: u32 = gg_label_panel(GG_CART_W, GG_CART_H).1;
pub const GG_LABEL_W: u32 = gg_label_panel(GG_CART_W, GG_CART_H).2 - GG_LABEL_X;
pub const GG_LABEL_H: u32 = gg_label_panel(GG_CART_W, GG_CART_H).3 - GG_LABEL_Y;

// NeoGeo Pocket: a ridged cap across the top, the wide label in a recessed
// frame, and below a seam two plaques with a down arrow between them.
pub use crate::bitmaps::{NGP_CART_H, NGP_CART_W};

pub const fn ngp_label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 115 + 500) / 1000,
        (h * 215 + 500) / 1000,
        (w * 905 + 500) / 1000,
        (h * 712 + 500) / 1000,
    )
}

pub const NGP_LABEL_X: u32 = ngp_label_panel(NGP_CART_W, NGP_CART_H).0;
pub const NGP_LABEL_Y: u32 = ngp_label_panel(NGP_CART_W, NGP_CART_H).1;
pub const NGP_LABEL_W: u32 = ngp_label_panel(NGP_CART_W, NGP_CART_H).2 - NGP_LABEL_X;
pub const NGP_LABEL_H: u32 = ngp_label_panel(NGP_CART_W, NGP_CART_H).3 - NGP_LABEL_Y;

// TurboGrafx-16 HuCard: a portrait card with its contacts and a dark panel
// across the top; the game art fills the lower half, square-cornered.
pub use crate::bitmaps::{TG16_CART_H, TG16_CART_W};

pub const fn tg16_label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 60 + 500) / 1000,
        (h * 420 + 500) / 1000,
        (w * 955 + 500) / 1000,
        (h * 905 + 500) / 1000,
    )
}

pub const TG16_LABEL_X: u32 = tg16_label_panel(TG16_CART_W, TG16_CART_H).0;
pub const TG16_LABEL_Y: u32 = tg16_label_panel(TG16_CART_W, TG16_CART_H).1;
pub const TG16_LABEL_W: u32 = tg16_label_panel(TG16_CART_W, TG16_CART_H).2 - TG16_LABEL_X;
pub const TG16_LABEL_H: u32 = tg16_label_panel(TG16_CART_W, TG16_CART_H).3 - TG16_LABEL_Y;

// Nintendo DS game card: the portrait label sits in a shallow recess over
// most of the face, above a little arrow at the foot.
pub use crate::bitmaps::{NDS_CART_H, NDS_CART_W};

pub const fn nds_label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 100 + 500) / 1000,
        (h * 45 + 500) / 1000,
        (w * 900 + 500) / 1000,
        (h * 870 + 500) / 1000,
    )
}

pub const NDS_LABEL_X: u32 = nds_label_panel(NDS_CART_W, NDS_CART_H).0;
pub const NDS_LABEL_Y: u32 = nds_label_panel(NDS_CART_W, NDS_CART_H).1;
pub const NDS_LABEL_W: u32 = nds_label_panel(NDS_CART_W, NDS_CART_H).2 - NDS_LABEL_X;
pub const NDS_LABEL_H: u32 = nds_label_panel(NDS_CART_W, NDS_CART_H).3 - NDS_LABEL_Y;

const PAD: u32 = 10;
const MAX_LINES: usize = 3;
const MIN_PX: f32 = 10.0;

pub struct CartFace {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

struct Spec {
    w: u32,
    h: u32,
    label: (u32, u32, u32, u32),
    mask: &'static [u8],
    mould: Mould,
    paint: &'static [u8],
}

enum Shape {
    Gba,
    Gb(GbShell),
    Extra(Mould),
}

impl Shape {
    fn of(mould: Mould) -> Shape {
        match mould {
            Mould::Gba => Shape::Gba,
            Mould::Notched => Shape::Gb(GbShell::Notched),
            Mould::Rounded => Shape::Gb(GbShell::Rounded),
            extra => Shape::Extra(extra),
        }
    }
}

/// The mould a cart is drawn from: the GBA cart, the Game Boy pak notched or
/// rounded, or one of the shapes slotPlus adds.
pub fn mould_of(cart: &Cart) -> Mould {
    match cart.platform {
        Platform::Ws => Mould::Ws,
        Platform::Lynx => Mould::Lynx,
        Platform::Gg => Mould::Gg,
        Platform::Ngp => Mould::Ngp,
        Platform::Tg16 => Mould::Tg16,
        Platform::Nds => Mould::Nds,
        _ => match gb_shell_of(cart) {
            None => Mould::Gba,
            Some(GbShell::Notched) => Mould::Notched,
            Some(GbShell::Rounded) => Mould::Rounded,
        },
    }
}

/// Whether a platform's carts go into the slot upside down. A HuCard is drawn
/// as it is printed, contacts at the top, so it turns end over end before it
/// goes in contacts first, and turns back when it comes out.
pub fn inserts_turned(platform: Platform) -> bool {
    platform == Platform::Tg16
}

/// The mould a platform's carts are drawn from when the cart is not known. A
/// Game Boy platform's carts take the notched pak's size either way.
pub fn mould_for(platform: Platform) -> Mould {
    match platform {
        Platform::Gba | Platform::Gbav => Mould::Gba,
        Platform::Gb | Platform::Gbc => Mould::Notched,
        Platform::Ws => Mould::Ws,
        Platform::Lynx => Mould::Lynx,
        Platform::Gg => Mould::Gg,
        Platform::Ngp => Mould::Ngp,
        Platform::Tg16 => Mould::Tg16,
        Platform::Nds => Mould::Nds,
    }
}

pub fn gb_shell_of(cart: &Cart) -> Option<GbShell> {
    match cart.platform {
        Platform::Gba
        | Platform::Gbav
        | Platform::Ws
        | Platform::Lynx
        | Platform::Gg
        | Platform::Ngp
        | Platform::Tg16
        | Platform::Nds => None,
        Platform::Gb | Platform::Gbc => Some(match cart.shell.map(|c| c.outline) {
            Some(Outline::Notched) => GbShell::Notched,
            Some(Outline::Rounded) => GbShell::Rounded,
            _ => match slot_store::gb::cached_class(&cart.rom) {
                Class::ColourOnly => GbShell::Rounded,
                Class::Original | Class::DualMode => GbShell::Notched,
            },
        }),
    }
}

fn shape_of(cart: &Cart) -> Shape {
    Shape::of(mould_of(cart))
}

/// The label box of a mould, as fractions of the cart.
pub fn mould_label_rect(mould: Mould) -> (f32, f32, f32, f32) {
    let s = spec(Shape::of(mould));
    let (x, y, w, h) = s.label;
    (
        x as f32 / s.w as f32,
        y as f32 / s.h as f32,
        w as f32 / s.w as f32,
        h as f32 / s.h as f32,
    )
}

fn spec(shape: Shape) -> Spec {
    match shape {
        Shape::Gba => Spec {
            w: CART_W,
            h: CART_H,
            label: (LABEL_X, LABEL_Y, LABEL_W, LABEL_H),
            mask: cart_mask(),
            mould: Mould::Gba,
            paint: bitmaps::gba_cart(),
        },
        Shape::Gb(shell) => Spec {
            w: GB_CART_W,
            h: GB_CART_H,
            label: match shell {
                GbShell::Notched => (GB_LABEL_X, GB_LABEL_Y, GB_LABEL_W, GB_LABEL_H),
                GbShell::Rounded => (GBC_LABEL_X, GBC_LABEL_Y, GBC_LABEL_W, GBC_LABEL_H),
            },
            mask: gb_cart_mask(shell),
            mould: match shell {
                GbShell::Notched => Mould::Notched,
                GbShell::Rounded => Mould::Rounded,
            },
            paint: match shell {
                GbShell::Notched => bitmaps::gb_cart(),
                GbShell::Rounded => bitmaps::gbc_cart(),
            },
        },
        Shape::Extra(mould) => {
            let (w, h) = mould.size();
            Spec {
                w,
                h,
                label: match mould {
                    Mould::Ws => (WS_LABEL_X, WS_LABEL_Y, WS_LABEL_W, WS_LABEL_H),
                    Mould::Gg => (GG_LABEL_X, GG_LABEL_Y, GG_LABEL_W, GG_LABEL_H),
                    Mould::Ngp => (NGP_LABEL_X, NGP_LABEL_Y, NGP_LABEL_W, NGP_LABEL_H),
                    Mould::Tg16 => (TG16_LABEL_X, TG16_LABEL_Y, TG16_LABEL_W, TG16_LABEL_H),
                    Mould::Nds => (NDS_LABEL_X, NDS_LABEL_Y, NDS_LABEL_W, NDS_LABEL_H),
                    _ => (LYNX_LABEL_X, LYNX_LABEL_Y, LYNX_LABEL_W, LYNX_LABEL_H),
                },
                mask: extra_mask(mould),
                mould,
                paint: bitmaps::extra_cart(mould),
            }
        }
    }
}

pub const SEATED_W: u32 = 240;
const SEATED_H: u32 = 141;
const GB_SEATED_H: u32 = 259;
/// tools/slot_art/dslite.py cuts its slot to this width.
pub const NDS_SEATED_W: u32 = 168;

pub fn seated_box(platform: Platform) -> (u32, u32) {
    match platform {
        Platform::Gba | Platform::Gbav => (SEATED_W, SEATED_H),
        Platform::Gb | Platform::Gbc => (SEATED_W, GB_SEATED_H),
        // A DS card is far smaller than the others, so it seats narrower, in a
        // DS Lite slot narrowed to match.
        Platform::Nds => {
            let (w, h) = mould_for(Platform::Nds).size();
            (NDS_SEATED_W, (h * NDS_SEATED_W + w / 2) / w)
        }
        // The others are drawn at the seated width already scaled, so their seated
        // height follows from their own aspect.
        other => {
            let (w, h) = mould_for(other).size();
            (SEATED_W, (h * SEATED_W + w / 2) / w)
        }
    }
}

pub fn cart_box(platform: Platform) -> (u32, u32) {
    let s = spec(Shape::of(mould_for(platform)));
    (s.w, s.h)
}

/// The shadow of any mould's outline, drawn under a cart that is fading.
pub fn mould_shadow(mould: Mould) -> CartFace {
    let s = spec(Shape::of(mould));
    shadow(s.w, s.h, s.mask)
}

/// The board a clear cart of any mould is drawn over.
pub fn mould_backing(mould: Mould) -> CartFace {
    match Shape::of(mould) {
        Shape::Gba => cart_backing(None),
        Shape::Gb(shell) => cart_backing(Some(shell)),
        Shape::Extra(mould) => {
            let s = spec(Shape::Extra(mould));
            let mut rgba = bitmaps::extra_board(mould).to_vec();
            for (px, cover) in rgba.chunks_exact_mut(4).zip(s.mask) {
                px[3] = ((px[3] as u32 * *cover as u32 + 127) / 255) as u8;
            }
            CartFace {
                rgba,
                w: s.w,
                h: s.h,
            }
        }
    }
}

pub fn cart_shadow() -> CartFace {
    shadow(CART_W, CART_H, cart_mask())
}

pub fn gb_cart_shadow(shell: GbShell) -> CartFace {
    shadow(GB_CART_W, GB_CART_H, gb_cart_mask(shell))
}

fn shadow(w: u32, h: u32, mask: &[u8]) -> CartFace {
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for cover in mask {
        rgba.extend_from_slice(&[0, 0, 0, *cover]);
    }
    CartFace { rgba, w, h }
}

pub fn cart_face(cart: &Cart) -> CartFace {
    let art = cart.label.as_deref().and_then(|p| label_art(cart, p));
    cart_face_with(cart, art)
}

/// A cart's label art at its label size. Most labels fill their box, cropped to
/// it. A HuCard label is printed on the card under its dark panel, and real
/// ones come in different heights: it spans the box's width at its own shape,
/// sits on the box's foot, and leaves the card showing above it.
pub fn label_art(cart: &Cart, path: &std::path::Path) -> Option<Vec<u8>> {
    let (lw, lh) = label_size(cart);
    match label_on_card(mould_of(cart)) {
        true => art::fit_bottom(path, lw, lh),
        false => art::cover(path, lw, lh),
    }
}

/// Whether a mould's label is printed straight on the card, with the card's own
/// plastic, in the shell's colour, wherever the art leaves it showing.
pub fn label_on_card(mould: Mould) -> bool {
    mould == Mould::Tg16
}

pub fn is_clear(cart: &Cart) -> bool {
    shell_for(cart).finish != Finish::Solid
}

/// Whether the shelf draws this cart's coloured interior under its plastic: any clear cart.
pub fn shows_interior(cart: &Cart) -> bool {
    is_clear(cart)
}

/// The back plastic, with the game's colour blended onto it by the same overlay as the shells, at
/// 80% opacity. A GBA cart uses `gba_interior.png`, a Game Boy pak `gb_interior.png`, and a Game
/// Boy Color pak `gbc_interior.png`. Same size as that cart's board and face.
pub fn cart_interior(cart: &Cart) -> CartFace {
    // The inside of the back half, so a two-tone cart's back colour.
    let colour = shell_for(cart).back_colour();
    let (src, w, h, mask) = match Shape::of(mould_of(cart)) {
        Shape::Extra(mould) => {
            let s = spec(Shape::Extra(mould));
            (bitmaps::extra_interior(mould), s.w, s.h, Some(s.mask))
        }
        Shape::Gba => (bitmaps::gba_interior(), BOARD_W, BOARD_H, None),
        Shape::Gb(shell) => {
            let s = spec(Shape::Gb(shell));
            let src = match shell {
                GbShell::Notched => bitmaps::gb_interior(),
                GbShell::Rounded => bitmaps::gbc_interior(),
            };
            (src, s.w, s.h, Some(s.mask))
        }
    };
    let mut rgba = src.to_vec();
    for (i, px) in rgba.chunks_exact_mut(4).enumerate() {
        if let Some(mask) = mask {
            px[3] = ((px[3] as u32 * mask[i] as u32 + 127) / 255) as u8;
        }
        if px[3] == 0 {
            continue;
        }
        for c in 0..3 {
            px[c] = overlay(px[c], colour[c]);
        }
        px[3] = ((u32::from(px[3]) * u32::from(CLEAR_ALPHA) + 127) / 255) as u8;
    }
    CartFace { rgba, w, h }
}

/// The board a clear cart's plastic is drawn over, between the back and front shells. A GBA cart
/// gets `board.png`. A pak gets `gb_board.png`, cut to the notched or rounded outline.
pub fn cart_backing(shell: Option<GbShell>) -> CartFace {
    let Some(shell) = shell else {
        return CartFace {
            rgba: bitmaps::board().to_vec(),
            w: BOARD_W,
            h: BOARD_H,
        };
    };
    let s = spec(Shape::Gb(shell));
    let mut rgba = bitmaps::gb_board().to_vec();
    for (px, cover) in rgba.chunks_exact_mut(4).zip(s.mask) {
        px[3] = ((px[3] as u32 * *cover as u32 + 127) / 255) as u8;
    }
    CartFace {
        rgba,
        w: s.w,
        h: s.h,
    }
}

pub fn label_size(cart: &Cart) -> (u32, u32) {
    let (_, _, lw, lh) = spec(shape_of(cart)).label;
    (lw, lh)
}

pub fn cart_face_with(cart: &Cart, art: Option<Vec<u8>>) -> CartFace {
    let s = spec(shape_of(cart));
    let mut face = shell_face(&s, &shell_for(cart));
    let label = match art {
        Some(rgba) => rgba,
        None => generated_label(&s, &label_text(cart)),
    };
    paste_label(&s, &mut face, &label);
    if let Some(decal) = bitmaps::extra_decal(s.mould) {
        paste_decal(&mut face, decal);
    }
    clip_to_silhouette(&s, &mut face);
    face
}

/// Lay a decal over the face as it is, keeping its own colour: the shell's tint
/// is already on the face beneath it.
fn paste_decal(face: &mut CartFace, decal: &[u8]) {
    for (dst, src) in face.rgba.chunks_exact_mut(4).zip(decal.chunks_exact(4)) {
        let a = u32::from(src[3]);
        if a == 0 {
            continue;
        }
        for c in 0..3 {
            dst[c] = ((u32::from(src[c]) * a + u32::from(dst[c]) * (255 - a) + 127) / 255) as u8;
        }
        dst[3] = dst[3].max(src[3]);
    }
}

/// The back of a cart: its coloured shell with no label, seen from behind, so
/// mirrored left to right and clipped to the mirrored outline.
pub fn cart_back_shell(cart: &Cart) -> CartFace {
    let s = spec(shape_of(cart));
    let shell = shell_for(cart);
    let back = Shell {
        colour: shell.back_colour(),
        ..shell
    };
    let front = shell_face(&s, &back);
    let (w, h) = (front.w as usize, front.h as usize);
    let mut rgba = vec![0u8; front.rgba.len()];
    for y in 0..h {
        for x in 0..w {
            let from = (y * w + (w - 1 - x)) * 4;
            let to = (y * w + x) * 4;
            rgba[to..to + 4].copy_from_slice(&front.rgba[from..from + 4]);
            let cover = s.mask[y * w + (w - 1 - x)] as u32;
            rgba[to + 3] = ((rgba[to + 3] as u32 * cover + 127) / 255) as u8;
        }
    }
    CartFace {
        rgba,
        w: front.w,
        h: front.h,
    }
}

fn clip_to_silhouette(s: &Spec, face: &mut CartFace) {
    for (px, cover) in face.rgba.chunks_exact_mut(4).zip(s.mask) {
        px[3] = ((px[3] as u32 * *cover as u32 + 127) / 255) as u8;
    }
}

pub fn label_colour(title: &str) -> [u8; 3] {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in title.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hsv_to_rgb((h % 360) as f32, 0.52, 0.74)
}

pub fn label_text(cart: &Cart) -> String {
    clean_label(&cart.stem)
}

pub fn clean_label(stem: &str) -> String {
    let mut bare = String::with_capacity(stem.len());
    let mut depth = 0u32;
    for ch in stem.chars() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => bare.push(ch),
            _ => {}
        }
    }

    let mut out = String::with_capacity(bare.len());
    for word in bare.split_whitespace().filter(|w| *w != "-") {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    if out.is_empty() {
        stem.to_string()
    } else {
        out
    }
}

pub fn label_tags(stem: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0u32;
    let mut cur = String::new();
    for ch in stem.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
                if depth == 1 {
                    cur.clear();
                    continue;
                }
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let t = cur.trim();
                    if !t.is_empty() {
                        out.push(t.to_string());
                    }
                    continue;
                }
            }
            _ => {}
        }
        if depth >= 1 {
            cur.push(ch);
        }
    }
    out
}

fn luma(px: &[u8]) -> u32 {
    (px[0] as u32 * 77 + px[1] as u32 * 150 + px[2] as u32 * 29) >> 8
}

fn shade_of(paint: &[u8], w: u32, h: u32, label: (u32, u32, u32, u32)) -> Vec<u32> {
    let (lx, ly, lw, lh) = label;
    let mut sum = 0u64;
    let mut n = 0u64;
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            if paint[i + 3] < 128 {
                continue;
            }
            let on_label = (lx..lx + lw).contains(&x) && (ly..ly + lh).contains(&y);
            if on_label {
                continue;
            }
            sum += u64::from(luma(&paint[i..]));
            n += 1;
        }
    }
    let mean = (sum / n.max(1)).max(1) as u32;
    paint
        .chunks_exact(4)
        .map(|px| (luma(px) * 256 / mean).clamp(176, 256))
        .collect()
}

/// The neutral shell asset for a mould: 50% grey plastic with the console name embossed on the
/// shoulder and the grey label box. `clear` makes the see-through variant, which is its own
/// asset with a rim and a sheen. The game's colour is blended on by `overlay`.
pub fn neutral_shell(mould: Mould, clear: bool) -> Vec<u8> {
    let s = spec(Shape::of(mould));
    assert_eq!(
        s.paint.len(),
        (s.w * s.h * 4) as usize,
        "cartridge template is not {}x{}",
        s.w,
        s.h
    );
    let shade = shade_of(s.paint, s.w, s.h, s.label);
    let mut rgba = vec![0u8; (s.w * s.h * 4) as usize];
    let inside = |x: i64, y: i64| {
        (0..i64::from(s.w)).contains(&x)
            && (0..i64::from(s.h)).contains(&y)
            && s.mask[(y * i64::from(s.w) + x) as usize] >= 128
    };
    for (i, (dst, shade)) in rgba.chunks_exact_mut(4).zip(shade).enumerate() {
        let (x, y) = ((i as u32 % s.w) as i64, (i as u32 / s.w) as i64);
        let grey = if clear {
            // Glassy plastic: flat 50% grey body, a darker rim where the walls are thick, and a
            // faint lighter diagonal sheen across the face.
            let rim = (1..=CLEAR_RIM as i64).find(|d| {
                ![(x - d, y), (x + d, y), (x, y - d), (x, y + d)]
                    .iter()
                    .all(|&(px, py)| inside(px, py))
            });
            let sheen = (x + y - i64::from(s.w) / 3).abs() < i64::from(s.w) / 14;
            match (rim, sheen) {
                (Some(_), _) => NEUTRAL - 28,
                (None, true) => NEUTRAL + 24,
                (None, false) => NEUTRAL,
            }
        } else {
            // The mould's shading (256 is unshaded) falls below 50% grey.
            NEUTRAL - ((256 - shade.min(256)) / 2) as u8
        };
        dst.copy_from_slice(&[grey, grey, grey, 255]);
    }
    let mut face = CartFace {
        rgba,
        w: s.w,
        h: s.h,
    };
    clip_to_silhouette(&s, &mut face);
    dress(&s, &mut face.rgba);
    face.rgba
}

fn dress(s: &Spec, rgba: &mut [u8]) {
    let (x0, y0, lw, lh) = s.label;
    for y in y0..y0 + lh {
        for x in x0..x0 + lw {
            let i = ((y * s.w + x) * 4) as usize;
            if rgba[i + 3] == 0 {
                continue;
            }
            rgba[i..i + 3].copy_from_slice(&LABEL_WELL);
        }
    }
    let title = match s.mould {
        Mould::Gba => "GAME BOY ADVANCE",
        Mould::Notched => "GAME BOY",
        Mould::Rounded => "GAME BOY COLOR",
        // The slotPlus moulds carry their markings in their art.
        Mould::Ws | Mould::Lynx | Mould::Gg | Mould::Ngp | Mould::Tg16 | Mould::Nds => return,
    };
    stamp_console(rgba, s.w, s.h, title, y0);
}

fn stamp_console(rgba: &mut [u8], w: u32, h: u32, title: &str, label_top: u32) {
    let Some(font) = text::label_font() else {
        return;
    };
    let band = label_top.max(1);
    let layout = text::fit_box(font, title, w as f32 * 0.72, band as f32, 2, 26.0, 9.0);
    let cov = text::coverage(w, band, &layout);
    let mut ink_top = band;
    let mut ink_bottom = 0u32;
    for y in 0..band {
        let row = &cov[(y * w) as usize..(y * w + w) as usize];
        if row.iter().any(|a| *a > 0) {
            ink_top = ink_top.min(y);
            ink_bottom = y;
        }
    }
    if ink_bottom < ink_top {
        return;
    }
    let text_h = ink_bottom - ink_top + 1;
    let dest_top = label_top.saturating_sub(3 + text_h);
    for y in ink_top..=ink_bottom {
        let dy = dest_top + (y - ink_top);
        if dy >= h {
            continue;
        }
        for x in 0..w {
            let a = cov[(y * w + x) as usize] as u32;
            if a == 0 {
                continue;
            }
            let i = ((dy * w + x) * 4) as usize;
            if rgba[i + 3] == 0 {
                continue;
            }
            let base = [rgba[i], rgba[i + 1], rgba[i + 2]];
            let ink = relief(base);
            for c in 0..3 {
                rgba[i + c] = ((ink[c] as u32 * a + base[c] as u32 * (255 - a)) / 255) as u8;
            }
        }
    }
}

/// The embossed name is a darker grey than the plastic around it; the overlay then darkens
/// whatever colour the game sets by the same proportion.
fn relief(px: [u8; 3]) -> [u8; 3] {
    px.map(|c| (c as u32 * 3 / 5) as u8)
}

const CLEAR_RIM: u32 = 5;

/// How much of a clear shell is opaque: 80%.
const CLEAR_ALPHA: u8 = 204;

/// The neutral shell's flat plastic: 50% grey, the level that leaves the game's colour as it is.
const NEUTRAL: u8 = 128;

/// Photoshop's Overlay blend of one channel: the neutral shell is the base layer and the game's
/// colour the blend layer. A base of 50% grey gives the colour exactly, a darker base darkens it
/// toward black and a lighter base lightens it toward white, so the shell's highlights, shadows
/// and embossed console name all follow whatever colour the game sets.
fn overlay(base: u8, colour: u8) -> u8 {
    let (base, colour) = (i32::from(base), i32::from(colour));
    let out = if base >= i32::from(NEUTRAL) {
        colour + ((255 - colour) * (base - i32::from(NEUTRAL)) + 63) / 127
    } else {
        colour - (colour * (i32::from(NEUTRAL) - base) + 64) / i32::from(NEUTRAL)
    };
    out.clamp(0, 255) as u8
}

fn shell_face(s: &Spec, shell: &Shell) -> CartFace {
    let clear = shell.finish != Finish::Solid;
    let src = bitmaps::shell_asset(s.mould, clear);
    assert_eq!(
        src.len(),
        (s.w * s.h * 4) as usize,
        "shell bitmap is not {}x{}",
        s.w,
        s.h
    );
    let mut rgba = src.to_vec();
    // The neutral asset is blended with the game's colour everywhere but the label box, which
    // stays the grey of the asset. A clear shell is its own asset with real transparency: what the
    // compositor drew underneath (the board) shows through and stays put while the shell moves.
    // The label box stays opaque.
    let (lx, ly, lw, lh) = s.label;
    let r = label_corner(s.mould);
    let boxed = !label_on_card(s.mould);
    for y in 0..s.h {
        for x in 0..s.w {
            let on_label = boxed
                && (lx..lx + lw).contains(&x)
                && (ly..ly + lh).contains(&y)
                && corner_cover(x - lx, y - ly, lw, lh, r) >= 128;
            let i = ((y * s.w + x) * 4) as usize;
            if !on_label {
                for c in 0..3 {
                    rgba[i + c] = overlay(rgba[i + c], shell.colour[c]);
                }
            }
            // The asset's own alpha is kept: its transparent corners and sides hold stray RGB
            // (often white), and only the alpha stops that being drawn.
            let own = u32::from(rgba[i + 3]);
            rgba[i + 3] = if on_label {
                255
            } else if clear {
                ((own * u32::from(CLEAR_ALPHA) + 127) / 255) as u8
            } else {
                own as u8
            };
        }
    }
    if shell.finish == Finish::Glitter {
        for y in 0..s.h {
            for x in 0..s.w {
                if !fleck(x, y) {
                    continue;
                }
                let i = ((y * s.w + x) * 4) as usize;
                if rgba[i + 3] == 0 {
                    continue;
                }
                let body = [rgba[i], rgba[i + 1], rgba[i + 2]];
                rgba[i..i + 3].copy_from_slice(&lerp(body, [0xff; 3], 100, 255));
                rgba[i + 3] = 255;
            }
        }
    }
    CartFace {
        rgba,
        w: s.w,
        h: s.h,
    }
}

fn fleck(x: u32, y: u32) -> bool {
    let mut h = x.wrapping_mul(0x9e37_79b9) ^ y.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h.is_multiple_of(100)
}

fn lerp(a: [u8; 3], b: [u8; 3], num: u32, den: u32) -> [u8; 3] {
    let mut out = [0u8; 3];
    for c in 0..3 {
        out[c] = ((a[c] as u32 * (den - num) + b[c] as u32 * num) / den) as u8;
    }
    out
}

fn generated_label(s: &Spec, title: &str) -> Vec<u8> {
    let (_, _, lw, lh) = s.label;
    let bg = label_colour(title);
    let mut rgba = Vec::with_capacity((lw * lh * 4) as usize);
    for _ in 0..lw * lh {
        rgba.extend_from_slice(&[bg[0], bg[1], bg[2], 255]);
    }

    if let Some(font) = text::label_font() {
        let (max_px, max_h) = match s.mould {
            Mould::Gba
            | Mould::Ws
            | Mould::Lynx
            | Mould::Gg
            | Mould::Ngp
            | Mould::Tg16
            | Mould::Nds => (lh as f32 / (MAX_LINES as f32 * 1.36), f32::INFINITY),
            Mould::Notched | Mould::Rounded => {
                let room = (lh - 2 * PAD) as f32;
                (room / MAX_LINES as f32, room)
            }
        };
        let layout = text::fit_box(
            font,
            title,
            (lw - 2 * PAD) as f32,
            max_h,
            MAX_LINES,
            max_px,
            MIN_PX,
        );
        text::draw_centred(&mut rgba, lw, lh, &layout, ink(bg));
    }
    rgba
}

fn ink(bg: [u8; 3]) -> [u8; 3] {
    let luma = 0.2126 * bg[0] as f32 + 0.7152 * bg[1] as f32 + 0.0722 * bg[2] as f32;
    if luma > 140.0 {
        [0x1a, 0x18, 0x16]
    } else {
        [0xf4, 0xf1, 0xea]
    }
}

/// How round a mould's label corners are. A Game Gear label has rounded
/// corners that tuck inside the arches of the hood, and a NeoGeo Pocket's
/// are softly rounded too, and a DS card's a little; the others are square.
fn label_corner(mould: Mould) -> f32 {
    match mould {
        Mould::Gg => 14.0,
        Mould::Ngp => 6.0,
        Mould::Nds => 4.0,
        _ => 0.0,
    }
}

/// How much of the label pixel at (x, y) a rounded corner of radius `r` keeps.
fn corner_cover(x: u32, y: u32, w: u32, h: u32, r: f32) -> u32 {
    if r <= 0.0 {
        return 255;
    }
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
    let cx = px.clamp(r, w as f32 - r);
    let cy = py.clamp(r, h as f32 - r);
    let out = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - r;
    ((0.5 - out).clamp(0.0, 1.0) * 255.0).round() as u32
}

fn paste_label(s: &Spec, face: &mut CartFace, label: &[u8]) {
    let (lx, ly, lw, lh) = s.label;
    let r = label_corner(s.mould);
    for y in 0..lh {
        for x in 0..lw {
            let src = ((y * lw + x) * 4) as usize;
            let a = (label[src + 3] as u32 * corner_cover(x, y, lw, lh, r) + 127) / 255;
            if a == 0 {
                continue;
            }
            let d = (((y + ly) * s.w + x + lx) * 4) as usize;
            for c in 0..3 {
                face.rgba[d + c] =
                    ((label[src + c] as u32 * a + face.rgba[d + c] as u32 * (255 - a) + 127) / 255)
                        as u8;
            }
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}
