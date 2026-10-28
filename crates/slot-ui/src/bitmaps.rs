use std::sync::OnceLock;

use slot_gfx::OUT_W;

use crate::art::decode_png_bytes;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mould {
    Gba,
    Notched,
    Rounded,
    Ws,
    Lynx,
    Gg,
    Ngp,
    Tg16,
    Nds,
}

impl Mould {
    pub const ALL: [Mould; 9] = [
        Mould::Gba,
        Mould::Notched,
        Mould::Rounded,
        Mould::Ws,
        Mould::Lynx,
        Mould::Gg,
        Mould::Ngp,
        Mould::Tg16,
        Mould::Nds,
    ];

    /// The shapes slotPlus adds to Slot's three, each its own set of art.
    pub const EXTRA: [Mould; 6] = [
        Mould::Ws,
        Mould::Lynx,
        Mould::Gg,
        Mould::Ngp,
        Mould::Tg16,
        Mould::Nds,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    /// The size the mould's art is drawn at.
    pub fn size(self) -> (u32, u32) {
        match self {
            Mould::Gba => (GBA_CART_W, GBA_CART_H),
            Mould::Notched | Mould::Rounded => (GB_CART_W, GB_CART_H),
            Mould::Ws => (WS_CART_W, WS_CART_H),
            Mould::Lynx => (LYNX_CART_W, LYNX_CART_H),
            Mould::Gg => (GG_CART_W, GG_CART_H),
            Mould::Ngp => (NGP_CART_W, NGP_CART_H),
            Mould::Tg16 => (TG16_CART_W, TG16_CART_H),
            Mould::Nds => (NDS_CART_W, NDS_CART_H),
        }
    }
}

pub const GBA_CART_W: u32 = 360;
pub const GBA_CART_H: u32 = 212;
pub const GB_CART_W: u32 = 312;
pub const GB_CART_H: u32 = 337;
/// A WonderSwan cart: a wide 66 x 41 mm shell, as wide as a GBA cart is.
pub const WS_CART_W: u32 = 352;
pub const WS_CART_H: u32 = 211;
/// An Atari Lynx card, nearly square, at the Game Boy pak's width.
pub const LYNX_CART_W: u32 = 312;
pub const LYNX_CART_H: u32 = 328;
/// A Game Gear pak, a little wider than it is tall, at the WonderSwan's width.
pub const GG_CART_W: u32 = 352;
pub const GG_CART_H: u32 = 329;
/// A NeoGeo Pocket pak, a little wider than it is tall, at the Game Boy pak's width.
pub const NGP_CART_W: u32 = 312;
pub const NGP_CART_H: u32 = 268;
/// A TurboGrafx-16 TurboChip: a HuCard the size of a credit card, 86 x 54 mm.
pub const TG16_CART_W: u32 = 214;
pub const TG16_CART_H: u32 = 337;
/// A Nintendo DS game card, 33 x 35 mm, a touch taller than it is wide.
pub const NDS_CART_W: u32 = 288;
pub const NDS_CART_H: u32 = 304;
pub const BOARD_W: u32 = GBA_CART_W;
pub const BOARD_H: u32 = GBA_CART_H;
pub const SOCKET_W: u32 = 64;
pub const SOCKET_H: u32 = 46;
pub const CHIP_W: u32 = 59;
pub const CHIP_H: u32 = 41;
pub const SLOT_W: u32 = OUT_W;
pub const SLOT_H: u32 = 58;
pub const PORT_W: u32 = OUT_W;
pub const PORT_H: u32 = SLOT_H;
pub const PLUG_W: u32 = 92;
pub const PLUG_H: u32 = 244;
pub const ADAPTER_W: u32 = 290;
pub const ADAPTER_H: u32 = 180;
pub const STICKER_W: u32 = 660;
pub const STICKER_H: u32 = 228;

const GBA_CART_PNG: &[u8] = include_bytes!("../../../assets/gba_cart.png");
const GB_CART_PNG: &[u8] = include_bytes!("../../../assets/gb_cart.png");
const GBC_CART_PNG: &[u8] = include_bytes!("../../../assets/gbc_cart.png");
const GBA_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/gba_interior.png");
const GB_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/gb_interior.png");
const GBC_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/gbc_interior.png");
const GB_BOARD_PNG: &[u8] = include_bytes!("../../../assets/gb_board.png");
const BOARD_PNG: &[u8] = include_bytes!("../../../assets/board.png");
const SOCKET_PNG: &[u8] = include_bytes!("../../../assets/socket.png");
const CHIP_PNG: &[u8] = include_bytes!("../../../assets/chip.png");
const SLOT_BACK_DARK_PNG: &[u8] = include_bytes!("../../../assets/slot_back_dark.png");
const SLOT_BACK_LIGHT_PNG: &[u8] = include_bytes!("../../../assets/slot_back_light.png");
const SLOT_FRONT_DARK_PNG: &[u8] = include_bytes!("../../../assets/slot_front_dark.png");
const SLOT_FRONT_LIGHT_PNG: &[u8] = include_bytes!("../../../assets/slot_front_light.png");
const SLOT_PLUGGED_DARK_PNG: &[u8] = include_bytes!("../../../assets/slot_plugged_dark.png");
const SLOT_PLUGGED_LIGHT_PNG: &[u8] = include_bytes!("../../../assets/slot_plugged_light.png");
const SLOT_WS_BACK_PNG: &[u8] = include_bytes!("../../../assets/slot_ws_back.png");
const SLOT_WS_FRONT_PNG: &[u8] = include_bytes!("../../../assets/slot_ws_front.png");
const SLOT_WS_PLUGGED_PNG: &[u8] = include_bytes!("../../../assets/slot_ws_plugged.png");
const SLOT_GG_BACK_PNG: &[u8] = include_bytes!("../../../assets/slot_gg_back.png");
const SLOT_GG_FRONT_PNG: &[u8] = include_bytes!("../../../assets/slot_gg_front.png");
const SLOT_GG_PLUGGED_PNG: &[u8] = include_bytes!("../../../assets/slot_gg_plugged.png");
const SLOT_NGP_BACK_PNG: &[u8] = include_bytes!("../../../assets/slot_ngp_back.png");
const SLOT_NGP_FRONT_PNG: &[u8] = include_bytes!("../../../assets/slot_ngp_front.png");
const SLOT_NGP_PLUGGED_PNG: &[u8] = include_bytes!("../../../assets/slot_ngp_plugged.png");
const SLOT_TG16_BACK_PNG: &[u8] = include_bytes!("../../../assets/slot_tg16_back.png");
const SLOT_TG16_FRONT_PNG: &[u8] = include_bytes!("../../../assets/slot_tg16_front.png");
const SLOT_TG16_PLUGGED_PNG: &[u8] = include_bytes!("../../../assets/slot_tg16_plugged.png");
const SLOT_NDS_BACK_PNG: &[u8] = include_bytes!("../../../assets/slot_nds_back.png");
const SLOT_NDS_FRONT_PNG: &[u8] = include_bytes!("../../../assets/slot_nds_front.png");
const SLOT_NDS_PLUGGED_PNG: &[u8] = include_bytes!("../../../assets/slot_nds_plugged.png");
const SLOT_LYNX_BACK_PNG: &[u8] = include_bytes!("../../../assets/slot_lynx_back.png");
const SLOT_LYNX_FRONT_PNG: &[u8] = include_bytes!("../../../assets/slot_lynx_front.png");
const SLOT_LYNX_PLUGGED_PNG: &[u8] = include_bytes!("../../../assets/slot_lynx_plugged.png");
const PORT_DARK_PNG: &[u8] = include_bytes!("../../../assets/link_port_dark.png");
const PORT_LIGHT_PNG: &[u8] = include_bytes!("../../../assets/link_port_light.png");
const PLUG_HOST_PNG: &[u8] = include_bytes!("../../../assets/link_plug_host.png");
const PLUG_JOIN_PNG: &[u8] = include_bytes!("../../../assets/link_plug_join.png");
const ADAPTER_PNG: &[u8] = include_bytes!("../../../assets/link_adapter.png");
const STICKER_PNG: &[u8] = include_bytes!("../../../assets/sticker.png");
// The slotPlus moulds, drawn by `assets/src/carts.py` in the style of the hand drawn ones.
const WS_CART_PNG: &[u8] = include_bytes!("../../../assets/ws_cart.png");
const WS_SHELL_PNG: &[u8] = include_bytes!("../../../assets/ws_neutral.png");
const WS_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/ws_clear.png");
const WS_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/ws_interior.png");
const WS_BOARD_PNG: &[u8] = include_bytes!("../../../assets/ws_board.png");
const LYNX_CART_PNG: &[u8] = include_bytes!("../../../assets/lynx_cart.png");
const LYNX_SHELL_PNG: &[u8] = include_bytes!("../../../assets/lynx_neutral.png");
const LYNX_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/lynx_clear.png");
const LYNX_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/lynx_interior.png");
const LYNX_BOARD_PNG: &[u8] = include_bytes!("../../../assets/lynx_board.png");
const GG_CART_PNG: &[u8] = include_bytes!("../../../assets/gg_cart.png");
const GG_SHELL_PNG: &[u8] = include_bytes!("../../../assets/gg_neutral.png");
const GG_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/gg_clear.png");
const GG_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/gg_interior.png");
const GG_BOARD_PNG: &[u8] = include_bytes!("../../../assets/gg_board.png");
const NGP_CART_PNG: &[u8] = include_bytes!("../../../assets/ngp_cart.png");
const NGP_SHELL_PNG: &[u8] = include_bytes!("../../../assets/ngp_neutral.png");
const NGP_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/ngp_clear.png");
const NGP_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/ngp_interior.png");
const NGP_BOARD_PNG: &[u8] = include_bytes!("../../../assets/ngp_board.png");
const TG16_CART_PNG: &[u8] = include_bytes!("../../../assets/tg16_cart.png");
const TG16_SHELL_PNG: &[u8] = include_bytes!("../../../assets/tg16_neutral.png");
const TG16_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/tg16_clear.png");
const TG16_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/tg16_interior.png");
const TG16_BOARD_PNG: &[u8] = include_bytes!("../../../assets/tg16_board.png");
const TG16_DECAL_PNG: &[u8] = include_bytes!("../../../assets/tg16_decal.png");
const NDS_CART_PNG: &[u8] = include_bytes!("../../../assets/nds_cart.png");
const NDS_SHELL_PNG: &[u8] = include_bytes!("../../../assets/nds_neutral.png");
const NDS_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/nds_clear.png");
const NDS_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/nds_interior.png");
const NDS_BOARD_PNG: &[u8] = include_bytes!("../../../assets/nds_board.png");
const WS_DECAL_PNG: &[u8] = include_bytes!("../../../assets/ws_decal.png");
const LYNX_DECAL_PNG: &[u8] = include_bytes!("../../../assets/lynx_decal.png");

/// What a slotPlus mould draws over its shell untinted: the gold contacts, and on a
/// WonderSwan the bare board below the shoulder. A Game Gear or NeoGeo Pocket shows nothing; a
/// TurboChip shows its contact strip.
pub fn extra_decal(mould: Mould) -> Option<&'static [u8]> {
    static WS: OnceLock<Vec<u8>> = OnceLock::new();
    static LYNX: OnceLock<Vec<u8>> = OnceLock::new();
    static TG16: OnceLock<Vec<u8>> = OnceLock::new();
    let (bytes, slot) = match mould {
        Mould::Ws => (WS_DECAL_PNG, &WS),
        Mould::Lynx => (LYNX_DECAL_PNG, &LYNX),
        Mould::Tg16 => (TG16_DECAL_PNG, &TG16),
        _ => return None,
    };
    let (w, h) = mould.size();
    Some(load(bytes, w, h, slot))
}

/// A slotPlus mould's silhouette (its alpha is the cart's outline).
pub fn extra_cart(mould: Mould) -> &'static [u8] {
    static WS: OnceLock<Vec<u8>> = OnceLock::new();
    static LYNX: OnceLock<Vec<u8>> = OnceLock::new();
    static GG: OnceLock<Vec<u8>> = OnceLock::new();
    static NGP: OnceLock<Vec<u8>> = OnceLock::new();
    static TG16: OnceLock<Vec<u8>> = OnceLock::new();
    static NDS: OnceLock<Vec<u8>> = OnceLock::new();
    let (bytes, slot) = match mould {
        Mould::Ws => (WS_CART_PNG, &WS),
        Mould::Lynx => (LYNX_CART_PNG, &LYNX),
        Mould::Gg => (GG_CART_PNG, &GG),
        Mould::Ngp => (NGP_CART_PNG, &NGP),
        Mould::Tg16 => (TG16_CART_PNG, &TG16),
        Mould::Nds => (NDS_CART_PNG, &NDS),
        other => panic!("{other:?} is not a slotPlus mould"),
    };
    let (w, h) = mould.size();
    load(bytes, w, h, slot)
}

/// The back plastic of a slotPlus mould, coloured by the game like the other interiors.
pub fn extra_interior(mould: Mould) -> &'static [u8] {
    static WS: OnceLock<Vec<u8>> = OnceLock::new();
    static LYNX: OnceLock<Vec<u8>> = OnceLock::new();
    static GG: OnceLock<Vec<u8>> = OnceLock::new();
    static NGP: OnceLock<Vec<u8>> = OnceLock::new();
    static TG16: OnceLock<Vec<u8>> = OnceLock::new();
    static NDS: OnceLock<Vec<u8>> = OnceLock::new();
    let (bytes, slot) = match mould {
        Mould::Ws => (WS_INTERIOR_PNG, &WS),
        Mould::Lynx => (LYNX_INTERIOR_PNG, &LYNX),
        Mould::Gg => (GG_INTERIOR_PNG, &GG),
        Mould::Ngp => (NGP_INTERIOR_PNG, &NGP),
        Mould::Tg16 => (TG16_INTERIOR_PNG, &TG16),
        Mould::Nds => (NDS_INTERIOR_PNG, &NDS),
        other => panic!("{other:?} is not a slotPlus mould"),
    };
    let (w, h) = mould.size();
    load(bytes, w, h, slot)
}

/// The board a clear slotPlus cart's plastic is drawn over.
pub fn extra_board(mould: Mould) -> &'static [u8] {
    static WS: OnceLock<Vec<u8>> = OnceLock::new();
    static LYNX: OnceLock<Vec<u8>> = OnceLock::new();
    static GG: OnceLock<Vec<u8>> = OnceLock::new();
    static NGP: OnceLock<Vec<u8>> = OnceLock::new();
    static TG16: OnceLock<Vec<u8>> = OnceLock::new();
    static NDS: OnceLock<Vec<u8>> = OnceLock::new();
    let (bytes, slot) = match mould {
        Mould::Ws => (WS_BOARD_PNG, &WS),
        Mould::Lynx => (LYNX_BOARD_PNG, &LYNX),
        Mould::Gg => (GG_BOARD_PNG, &GG),
        Mould::Ngp => (NGP_BOARD_PNG, &NGP),
        Mould::Tg16 => (TG16_BOARD_PNG, &TG16),
        Mould::Nds => (NDS_BOARD_PNG, &NDS),
        other => panic!("{other:?} is not a slotPlus mould"),
    };
    let (w, h) = mould.size();
    load(bytes, w, h, slot)
}

fn load(bytes: &'static [u8], w: u32, h: u32, slot: &'static OnceLock<Vec<u8>>) -> &'static [u8] {
    slot.get_or_init(|| {
        let (mut rgba, sw, sh) = decode_png_bytes(bytes).expect("a template png decodes");
        if sh == h && sw > w {
            rgba = crop_centre(&rgba, sw, sh, w);
        } else if (sw, sh) != (w, h) {
            rgba = scale(&rgba, sw, sh, w, h);
        }
        // Transparent pixels are often stored as white. The cart is sampled with a linear
        // filter, which would pull that white in along every edge.
        bleed(&mut rgba, w, h);
        rgba
    })
}

/// Copy an opaque neighbour's colour into each transparent pixel, leaving its alpha at 0.
fn bleed(rgba: &mut [u8], w: u32, h: u32) {
    let src = rgba.to_vec();
    let (w, h) = (w as i32, h as i32);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            if src[i + 3] != 0 {
                continue;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let n = ((ny * w + nx) * 4) as usize;
                if src[n + 3] == 0 {
                    continue;
                }
                rgba[i..i + 3].copy_from_slice(&src[n..n + 3]);
                break;
            }
        }
    }
}

fn crop_centre(src: &[u8], sw: u32, sh: u32, w: u32) -> Vec<u8> {
    let left = ((sw - w) / 2) as usize;
    let mut out = Vec::with_capacity((w * sh * 4) as usize);
    for row in src.chunks_exact(sw as usize * 4) {
        out.extend_from_slice(&row[left * 4..(left + w as usize) * 4]);
    }
    out
}

fn scale(src: &[u8], sw: u32, sh: u32, w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let sy = y * sh / h;
        for x in 0..w {
            let sx = x * sw / w;
            let from = ((sy * sw + sx) * 4) as usize;
            let to = ((y * w + x) * 4) as usize;
            out[to..to + 4].copy_from_slice(&src[from..from + 4]);
        }
    }
    out
}

pub fn gba_cart() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBA_CART_PNG, GBA_CART_W, GBA_CART_H, &P)
}

pub fn gb_cart() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GB_CART_PNG, GB_CART_W, GB_CART_H, &P)
}

pub fn gbc_cart() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBC_CART_PNG, GB_CART_W, GB_CART_H, &P)
}

/// The inside of a GBA cart, from `assets/gba_interior.png`. Same size as the cart and
/// the board, so the carousel can stack the three 1:1. Its colour is set by the game, with the
/// same overlay blend the shells use.
pub fn gba_interior() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBA_INTERIOR_PNG, BOARD_W, BOARD_H, &P)
}

/// The back of a Game Boy pak, from `assets/gb_interior.png`.
pub fn gb_interior() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GB_INTERIOR_PNG, GB_CART_W, GB_CART_H, &P)
}

/// The back of a Game Boy Color pak, from `assets/gbc_interior.png`.
pub fn gbc_interior() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBC_INTERIOR_PNG, GB_CART_W, GB_CART_H, &P)
}

/// The Game Boy PCB, from `assets/gb_board.png`, drawn between the pak's back and front plastic.
pub fn gb_board() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GB_BOARD_PNG, GB_CART_W, GB_CART_H, &P)
}

pub fn board() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(BOARD_PNG, BOARD_W, BOARD_H, &P)
}

pub fn socket() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SOCKET_PNG, SOCKET_W, SOCKET_H, &P)
}

pub fn chip() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(CHIP_PNG, CHIP_W, CHIP_H, &P)
}

/// A system's own cart slot, for Cart Slot set to Bespoke: the back, front and
/// plugged strips, the size of Slot's and with its mouth in the same place, in
/// the handheld's own colours. A system without one keeps Slot's.
///
/// Each is `assets/slot_<system>_{back,front,plugged}.png`, added here as it is
/// drawn.
/// tools/slot_art draws them.
pub fn bespoke_slot(platform: slot_store::Platform) -> Option<[&'static [u8]; 3]> {
    use slot_store::Platform;
    match platform {
        // The pearl white WonderSwan, the WonderSwan Color's too.
        Platform::Ws => {
            static B: OnceLock<Vec<u8>> = OnceLock::new();
            static F: OnceLock<Vec<u8>> = OnceLock::new();
            static P: OnceLock<Vec<u8>> = OnceLock::new();
            Some([
                load(SLOT_WS_BACK_PNG, SLOT_W, SLOT_H, &B),
                load(SLOT_WS_FRONT_PNG, SLOT_W, SLOT_H, &F),
                load(SLOT_WS_PLUGGED_PNG, SLOT_W, SLOT_H, &P),
            ])
        }
        // The grey Game Gear.
        Platform::Gg => {
            static B: OnceLock<Vec<u8>> = OnceLock::new();
            static F: OnceLock<Vec<u8>> = OnceLock::new();
            static P: OnceLock<Vec<u8>> = OnceLock::new();
            Some([
                load(SLOT_GG_BACK_PNG, SLOT_W, SLOT_H, &B),
                load(SLOT_GG_FRONT_PNG, SLOT_W, SLOT_H, &F),
                load(SLOT_GG_PLUGGED_PNG, SLOT_W, SLOT_H, &P),
            ])
        }
        // The carbon Neo Geo Pocket Color.
        Platform::Ngp => {
            static B: OnceLock<Vec<u8>> = OnceLock::new();
            static F: OnceLock<Vec<u8>> = OnceLock::new();
            static P: OnceLock<Vec<u8>> = OnceLock::new();
            Some([
                load(SLOT_NGP_BACK_PNG, SLOT_W, SLOT_H, &B),
                load(SLOT_NGP_FRONT_PNG, SLOT_W, SLOT_H, &F),
                load(SLOT_NGP_PLUGGED_PNG, SLOT_W, SLOT_H, &P),
            ])
        }
        // The TurboExpress, the TurboGrafx-16's handheld.
        Platform::Tg16 => {
            static B: OnceLock<Vec<u8>> = OnceLock::new();
            static F: OnceLock<Vec<u8>> = OnceLock::new();
            static P: OnceLock<Vec<u8>> = OnceLock::new();
            Some([
                load(SLOT_TG16_BACK_PNG, SLOT_W, SLOT_H, &B),
                load(SLOT_TG16_FRONT_PNG, SLOT_W, SLOT_H, &F),
                load(SLOT_TG16_PLUGGED_PNG, SLOT_W, SLOT_H, &P),
            ])
        }
        // The grey Lynx II.
        Platform::Lynx => {
            static B: OnceLock<Vec<u8>> = OnceLock::new();
            static F: OnceLock<Vec<u8>> = OnceLock::new();
            static P: OnceLock<Vec<u8>> = OnceLock::new();
            Some([
                load(SLOT_LYNX_BACK_PNG, SLOT_W, SLOT_H, &B),
                load(SLOT_LYNX_FRONT_PNG, SLOT_W, SLOT_H, &F),
                load(SLOT_LYNX_PLUGGED_PNG, SLOT_W, SLOT_H, &P),
            ])
        }
        // The black DS Lite.
        Platform::Nds => {
            static B: OnceLock<Vec<u8>> = OnceLock::new();
            static F: OnceLock<Vec<u8>> = OnceLock::new();
            static P: OnceLock<Vec<u8>> = OnceLock::new();
            Some([
                load(SLOT_NDS_BACK_PNG, SLOT_W, SLOT_H, &B),
                load(SLOT_NDS_FRONT_PNG, SLOT_W, SLOT_H, &F),
                load(SLOT_NDS_PLUGGED_PNG, SLOT_W, SLOT_H, &P),
            ])
        }
        _ => None,
    }
}

/// Whether a system's own cart slot is pale plastic, so the battery and clock
/// printed over it take the light theme's ink whatever the theme.
pub fn bespoke_slot_light(platform: slot_store::Platform) -> bool {
    matches!(platform, slot_store::Platform::Ws)
}

pub fn slot_back() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_BACK_DARK_PNG, SLOT_W, SLOT_H, &P)
}

pub fn slot_back_light() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_BACK_LIGHT_PNG, SLOT_W, SLOT_H, &P)
}

pub fn slot_front() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_FRONT_DARK_PNG, SLOT_W, SLOT_H, &P)
}

pub fn slot_front_light() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_FRONT_LIGHT_PNG, SLOT_W, SLOT_H, &P)
}

pub fn slot_plugged() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_PLUGGED_DARK_PNG, SLOT_W, SLOT_H, &P)
}

pub fn slot_plugged_light() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_PLUGGED_LIGHT_PNG, SLOT_W, SLOT_H, &P)
}

pub fn link_port() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(PORT_DARK_PNG, PORT_W, PORT_H, &P)
}

pub fn link_port_light() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(PORT_LIGHT_PNG, PORT_W, PORT_H, &P)
}

pub fn link_plug_host() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(PLUG_HOST_PNG, PLUG_W, PLUG_H, &P)
}

pub fn link_plug_join() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(PLUG_JOIN_PNG, PLUG_W, PLUG_H, &P)
}

pub fn link_adapter() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(ADAPTER_PNG, ADAPTER_W, ADAPTER_H, &P)
}

pub fn sticker() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(STICKER_PNG, STICKER_W, STICKER_H, &P)
}

// Shells are hand drawn. There is no Game Boy clear file yet, so a clear Game Boy pak uses
// the neutral shell at 80%.
const GBA_SHELL_PNG: &[u8] = include_bytes!("../../../assets/gba_neutral.png");
const GB_SHELL_PNG: &[u8] = include_bytes!("../../../assets/gb_neutral.png");
const GBC_SHELL_PNG: &[u8] = include_bytes!("../../../assets/gbc_neutral.png");
const GBA_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/gba_clear.png");
const GB_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/gb_neutral.png");
const GBC_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/gbc_clear.png");

/// The neutral shell asset for a mould, opaque or the separate clear one. The game's colour is
/// blended onto it by the caller.
pub fn shell_asset(mould: Mould, clear: bool) -> &'static [u8] {
    static GBA: OnceLock<Vec<u8>> = OnceLock::new();
    static GB: OnceLock<Vec<u8>> = OnceLock::new();
    static GBC: OnceLock<Vec<u8>> = OnceLock::new();
    static GBA_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static GB_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static GBC_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static WS: OnceLock<Vec<u8>> = OnceLock::new();
    static WS_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static LYNX: OnceLock<Vec<u8>> = OnceLock::new();
    static LYNX_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static GG: OnceLock<Vec<u8>> = OnceLock::new();
    static GG_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static NGP: OnceLock<Vec<u8>> = OnceLock::new();
    static NGP_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static TG16: OnceLock<Vec<u8>> = OnceLock::new();
    static TG16_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static NDS: OnceLock<Vec<u8>> = OnceLock::new();
    static NDS_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    let (bytes, w, h, slot) = match (mould, clear) {
        (Mould::Ws, false) => (WS_SHELL_PNG, WS_CART_W, WS_CART_H, &WS),
        (Mould::Ws, true) => (WS_CLEAR_PNG, WS_CART_W, WS_CART_H, &WS_CLEAR),
        (Mould::Lynx, false) => (LYNX_SHELL_PNG, LYNX_CART_W, LYNX_CART_H, &LYNX),
        (Mould::Lynx, true) => (LYNX_CLEAR_PNG, LYNX_CART_W, LYNX_CART_H, &LYNX_CLEAR),
        (Mould::Gg, false) => (GG_SHELL_PNG, GG_CART_W, GG_CART_H, &GG),
        (Mould::Gg, true) => (GG_CLEAR_PNG, GG_CART_W, GG_CART_H, &GG_CLEAR),
        (Mould::Ngp, false) => (NGP_SHELL_PNG, NGP_CART_W, NGP_CART_H, &NGP),
        (Mould::Ngp, true) => (NGP_CLEAR_PNG, NGP_CART_W, NGP_CART_H, &NGP_CLEAR),
        (Mould::Tg16, false) => (TG16_SHELL_PNG, TG16_CART_W, TG16_CART_H, &TG16),
        (Mould::Tg16, true) => (TG16_CLEAR_PNG, TG16_CART_W, TG16_CART_H, &TG16_CLEAR),
        (Mould::Nds, false) => (NDS_SHELL_PNG, NDS_CART_W, NDS_CART_H, &NDS),
        (Mould::Nds, true) => (NDS_CLEAR_PNG, NDS_CART_W, NDS_CART_H, &NDS_CLEAR),
        (Mould::Gba, false) => (GBA_SHELL_PNG, GBA_CART_W, GBA_CART_H, &GBA),
        (Mould::Notched, false) => (GB_SHELL_PNG, GB_CART_W, GB_CART_H, &GB),
        (Mould::Rounded, false) => (GBC_SHELL_PNG, GB_CART_W, GB_CART_H, &GBC),
        (Mould::Gba, true) => (GBA_CLEAR_PNG, GBA_CART_W, GBA_CART_H, &GBA_CLEAR),
        (Mould::Notched, true) => (GB_CLEAR_PNG, GB_CART_W, GB_CART_H, &GB_CLEAR),
        (Mould::Rounded, true) => (GBC_CLEAR_PNG, GB_CART_W, GB_CART_H, &GBC_CLEAR),
    };
    load(bytes, w, h, slot)
}

/// Footer buttons. Each file is already the footer height.
#[derive(Copy, Clone)]
pub(crate) enum ButtonLegend {
    A,
    B,
    Left,
    Right,
    Select,
    X,
    Y,
}

const BUTTON_LEGEND_PNG: [&[u8]; 7] = [
    include_bytes!("../../../assets/buttonlegend_round_a.png"),
    include_bytes!("../../../assets/buttonlegend_round_b.png"),
    include_bytes!("../../../assets/buttonlegend_dpad_left.png"),
    include_bytes!("../../../assets/buttonlegend_dpad_right.png"),
    include_bytes!("../../../assets/buttonlegend_long_select.png"),
    include_bytes!("../../../assets/buttonlegend_round_x.png"),
    include_bytes!("../../../assets/buttonlegend_round_y.png"),
];

pub(crate) fn button_legend(which: ButtonLegend) -> (&'static [u8], u32, u32) {
    static ALL: OnceLock<Vec<(Vec<u8>, u32, u32)>> = OnceLock::new();
    let all = ALL.get_or_init(|| {
        BUTTON_LEGEND_PNG
            .iter()
            .map(|bytes| {
                let (mut rgba, w, h) = decode_png_bytes(bytes).expect("a button legend decodes");
                bleed(&mut rgba, w, h);
                (rgba, w, h)
            })
            .collect()
    });
    let (rgba, w, h) = &all[which as usize];
    (rgba, *w, *h)
}
