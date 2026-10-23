use slot_gfx::Canvas;
use slot_store::{Cart, Core};

use crate::bitmaps;
use crate::cart::{
    cart_back_shell, clean_label, CartFace, CART_H, CART_W, LYNX_CART_H, LYNX_CART_W, LYNX_LABEL_W,
    LYNX_LABEL_X,
};
use crate::shelf::foot_y;
use crate::slot_chrome::ease;
use crate::text;

pub const BOARD_W: u32 = CART_W;
pub const BOARD_H: u32 = CART_H;

/// The black ROM chip on `assets/board.png`: x 61..132, y 48..138.
pub const ROM_X: u32 = 61;
pub const ROM_Y: u32 = 48;
pub const ROM_W: u32 = 71;
pub const ROM_H: u32 = 90;

const MARK_LINE_CHARS: usize = 10;
const MARK_LINES: usize = 3;
const MARK_PAD: u32 = 4;
const MARK_PX: f32 = 10.0;
const MARK_MIN_PX: f32 = 6.0;
const MARK_INK: [u8; 3] = [0xbd, 0xbd, 0xbd];

pub fn rom_marking(stem: &str) -> Vec<String> {
    let mut title: Vec<String> = Vec::new();
    for word in clean_label(stem).to_uppercase().split_whitespace() {
        match title.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= MARK_LINE_CHARS => {
                line.push(' ');
                line.push_str(word);
            }
            _ => title.push(word.to_string()),
        }
    }
    if title.len() > MARK_LINES {
        let tail = title.split_off(MARK_LINES - 1).join(" ");
        title.push(tail);
    }
    title
}

pub fn rom_marking_face(stem: &str) -> CartFace {
    let title = rom_marking(stem);
    let mut face = CartFace {
        rgba: vec![0; (ROM_W * ROM_H * 4) as usize],
        w: ROM_W,
        h: ROM_H,
    };
    let Some(font) = text::label_font() else {
        return face;
    };
    let max_w = (ROM_W - 2 * MARK_PAD) as f32;
    let line_h = (MARK_PX * 1.25).ceil() as u32;
    let mut top = ROM_H.saturating_sub(line_h * title.len() as u32) / 2;
    for line in &title {
        let layout = text::fit(font, line, max_w, 1, MARK_PX, MARK_MIN_PX);
        ink_band(&mut face, top, line_h, &layout, MARK_INK, 1.0);
        top += line_h;
    }
    face
}

pub fn board_face(cart: &Cart) -> CartFace {
    let mut face = CartFace {
        rgba: bitmaps::board().to_vec(),
        w: BOARD_W,
        h: BOARD_H,
    };
    over(&mut face, &rom_marking_face(&cart.stem), ROM_X, ROM_Y);
    face
}

fn ink_band(
    face: &mut CartFace,
    top: u32,
    band: u32,
    layout: &text::Layout,
    ink: [u8; 3],
    strength: f32,
) {
    let band = band.min(face.h.saturating_sub(top));
    for (i, a) in text::coverage(face.w, band, layout).into_iter().enumerate() {
        let a = (a as f32 * strength).round() as u8;
        if a == 0 {
            continue;
        }
        let at = ((top * face.w) as usize + i) * 4;
        let (a, d_a) = (a as f32 / 255.0, face.rgba[at + 3] as f32 / 255.0);
        let out_a = a + d_a * (1.0 - a);
        for (k, &c) in ink.iter().enumerate() {
            let d_rgb = face.rgba[at + k] as f32 / 255.0;
            let out_rgb = (c as f32 / 255.0 * a + d_rgb * d_a * (1.0 - a)) / out_a;
            face.rgba[at + k] = (out_rgb * 255.0).round() as u8;
        }
        face.rgba[at + 3] = (out_a * 255.0).round() as u8;
    }
}

fn over(dst: &mut CartFace, src: &CartFace, x: u32, y: u32) {
    for row in 0..src.h {
        for col in 0..src.w {
            let s = ((row * src.w + col) * 4) as usize;
            let a = src.rgba[s + 3] as u32;
            let (dx, dy) = (x + col, y + row);
            if a == 0 || dx >= dst.w || dy >= dst.h {
                continue;
            }
            let d = ((dy * dst.w + dx) * 4) as usize;
            for k in 0..3 {
                dst.rgba[d + k] =
                    ((src.rgba[s + k] as u32 * a + dst.rgba[d + k] as u32 * (255 - a)) / 255) as u8;
            }
            dst.rgba[d + 3] = dst.rgba[d + 3].max(a as u8);
        }
    }
}

fn bitmap_face(rgba: &[u8], w: u32, h: u32) -> CartFace {
    CartFace {
        rgba: rgba.to_vec(),
        w,
        h,
    }
}

pub fn board_x(canvas: Canvas) -> f32 {
    canvas.centred(BOARD_W as f32)
}

pub const BOARD_Y: f32 = 150.0;

pub const TURN_PAD: u32 = 2;

pub const LID_TURN: f32 = -5.0 * std::f32::consts::PI / 180.0;
pub const CHIP_TIP: f32 = 4.0 * std::f32::consts::PI / 180.0;
pub const HOP_LIFT: f32 = 10.0;

pub const SLIDE_SHARE: f32 = 160.0 / 420.0;
pub const SLIDE_UP: f32 = CART_H as f32 / 3.0;

pub fn slide_of(progress: f32) -> f32 {
    ease((progress / SLIDE_SHARE).clamp(0.0, 1.0))
}

pub fn lift_of(progress: f32) -> f32 {
    ease(((progress - SLIDE_SHARE) / (1.0 - SLIDE_SHARE)).clamp(0.0, 1.0))
}

pub const SOCKET_U: [f32; 2] = [99.5, 171.5];
pub const SOCKET_V: f32 = 59.1;
pub const SOCKET_W: u32 = 64;
pub const SOCKET_H: u32 = 46;

pub const CHIP_U: [f32; 2] = [101.0, 173.0];
pub const CHIP_V: f32 = 60.6;
pub const CHIP_W: u32 = 59;
pub const CHIP_H: u32 = 41;

pub const SHADOW_W: u32 = 66;
pub const SHADOW_H: u32 = 12;

const NAME_PX: f32 = 17.0;
const NAME_MIN_PX: f32 = 8.0;
const SOCKET_INK: [u8; 3] = [0xee, 0xf5, 0xe6];
const CHIP_INK: [u8; 3] = [0xf2, 0xf2, 0xf2];

fn lid_rest(canvas: Canvas) -> Placed {
    Placed {
        x: canvas.centred(144.0),
        y: 70.5 - LID_REST_H / 2.0,
        w: 144.0,
        h: LID_REST_H,
    }
}

const LID_REST_H: f32 = 144.0 * CART_H as f32 / CART_W as f32;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Placed {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn shelf_cart(canvas: Canvas) -> Placed {
    shelf_cart_at(canvas.centred(CART_W as f32), 1.0)
}

pub fn shelf_cart_at(x: f32, scale: f32) -> Placed {
    let (w, h) = (CART_W as f32 * scale, CART_H as f32 * scale);
    Placed {
        x,
        y: foot_y(CART_H as f32) - h,
        w,
        h,
    }
}

pub fn board_at(canvas: Canvas, progress: f32) -> Placed {
    board_from(canvas, shelf_cart(canvas), progress)
}

pub fn board_from(canvas: Canvas, shelf: Placed, progress: f32) -> Placed {
    let rest = Placed {
        x: board_x(canvas),
        y: BOARD_Y,
        w: BOARD_W as f32,
        h: BOARD_H as f32,
    };
    lerp(board_in(shelf), rest, lift_of(progress))
}

/// The board and the back shell fill the cart they sit in, same size as the lid. Letterboxing them
/// to the board bitmap's own aspect made both look shorter the moment the cart opened.
pub fn board_in(cart: Placed) -> Placed {
    cart
}

pub fn lid_at(canvas: Canvas, progress: f32) -> (Placed, f32) {
    lid_from(canvas, shelf_cart(canvas), progress)
}

pub fn lid_from(canvas: Canvas, shelf: Placed, progress: f32) -> (Placed, f32) {
    let slid = Placed {
        y: shelf.y - SLIDE_UP * slide_of(progress),
        ..shelf
    };
    let lift = lift_of(progress);
    (lerp(slid, lid_rest(canvas), lift), LID_TURN * lift)
}

const UNITS_W: f32 = 240.0;
const UNITS_H: f32 = 135.0;

pub fn on_board(board: Placed, u: f32, v: f32) -> (f32, f32) {
    (
        board.x + u * board.w / UNITS_W,
        board.y + v * board.h / UNITS_H,
    )
}

pub fn board_zoom(board: Placed) -> f32 {
    board.w / BOARD_W as f32
}

pub fn grown(p: Placed, by: f32) -> Placed {
    Placed {
        x: p.x - by,
        y: p.y - by,
        w: p.w + 2.0 * by,
        h: p.h + 2.0 * by,
    }
}

fn lerp(a: Placed, b: Placed, t: f32) -> Placed {
    Placed {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        w: a.w + (b.w - a.w) * t,
        h: a.h + (b.h - a.h) * t,
    }
}

pub fn padded(face: &CartFace, pad: u32) -> CartFace {
    let (w, h) = (face.w + 2 * pad, face.h + 2 * pad);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let stride = (face.w * 4) as usize;
    for row in 0..face.h {
        let from = (row * face.w * 4) as usize;
        let to = (((row + pad) * w + pad) * 4) as usize;
        rgba[to..to + stride].copy_from_slice(&face.rgba[from..from + stride]);
    }
    CartFace { rgba, w, h }
}

pub fn socket_face(core: Core) -> CartFace {
    let mut face = bitmap_face(bitmaps::socket(), SOCKET_W, SOCKET_H);
    if let Some(layout) = name_layout(core, (SOCKET_W - 12) as f32) {
        ink_band(&mut face, 4, 37, &layout, SOCKET_INK, 0.5);
    }
    face
}

/// The ways a Lynx can be held, in the order of the sockets on the back of its
/// cart: north, west, east, south. The chip's socket says which way the top of
/// the handheld points while the game is played.
pub const HOLD_NAMES: [&str; 4] = ["Upright", "90CCW", "90CW", "180"];

const HOLD_EDGE: u32 = 10;

/// Where each hold's socket sits on the back of a Lynx cart, as the top-left
/// corner of the socket in the face's own pixels: a compass in the recessed
/// panel the label sits in on the front.
pub const fn hold_socket_at(i: usize) -> (u32, u32) {
    let (x, y, w, h) = (BACK_FLAT_X, BACK_FLAT_Y, BACK_FLAT_W, BACK_FLAT_H);
    let (cx, cy) = (x + w / 2, y + h / 2);
    match i {
        0 => (cx - SOCKET_W / 2, y + HOLD_EDGE),
        1 => (x + HOLD_EDGE, cy - SOCKET_H / 2),
        2 => (x + w - HOLD_EDGE - SOCKET_W, cy - SOCKET_H / 2),
        _ => (cx - SOCKET_W / 2, y + h - HOLD_EDGE - SOCKET_H),
    }
}

/// The lip across the top of a Lynx cart's back: the front's curved top stands
/// proud of the flat back by this much, and casts a short shadow below it.
const BACK_LIP_H: u32 = 32;
const BACK_LIP_TOP: u32 = 22;
const BACK_LIP_SHADOW: u32 = 12;
/// The flat of the back below the lip, where the sockets go.
const BACK_FLAT_X: u32 = 22;
const BACK_FLAT_Y: u32 = BACK_LIP_H + 22;
const BACK_FLAT_W: u32 = LYNX_CART_W - 2 * BACK_FLAT_X;
const BACK_FLAT_H: u32 = LYNX_CART_H - BACK_FLAT_Y - 26;

/// Where the chip sits when it is seated in a hold's socket, its top-left
/// corner in the back face's pixels.
pub fn hold_chip_at(i: usize) -> (f32, f32) {
    let (x, y) = hold_socket_at(i);
    (
        x as f32 + (SOCKET_W - CHIP_W) as f32 / 2.0,
        y as f32 + (SOCKET_H - CHIP_H) as f32 / 2.0,
    )
}

/// A socket marked with a word instead of a core's name.
pub fn named_socket_face(name: &str) -> CartFace {
    let mut face = bitmap_face(bitmaps::socket(), SOCKET_W, SOCKET_H);
    if let Some(layout) = word_layout(name, (SOCKET_W - 12) as f32) {
        ink_band(&mut face, 4, 37, &layout, SOCKET_INK, 0.5);
    }
    face
}

/// The back of a Lynx cart: flat plastic in the shell's colour, with the lip
/// of the curved top standing out across it, and a socket for each rotation
/// with Rotation marked in the middle of them.
pub fn lynx_back_face(cart: &Cart) -> CartFace {
    let mut face = cart_back_shell(cart);
    flat_back(&mut face);
    for (i, name) in HOLD_NAMES.iter().enumerate() {
        let (x, y) = hold_socket_at(i);
        over(&mut face, &named_socket_face(name), x, y);
    }
    if let Some(layout) = word_layout("Rotation", 100.0) {
        let line_h = (NAME_PX * 1.4).ceil() as u32;
        let top = BACK_FLAT_Y + BACK_FLAT_H / 2 - line_h / 2;
        let mut band = CartFace {
            rgba: vec![0; (face.w * line_h * 4) as usize],
            w: face.w,
            h: line_h,
        };
        // Moulded into the plastic: a shade lighter or darker than the shell,
        // whichever reads on it.
        let ink = match luminance(shell_colour(&face)) > 0.45 {
            true => [0x40, 0x40, 0x44],
            false => SOCKET_INK,
        };
        ink_band(&mut band, 0, line_h, &layout, ink, 0.55);
        over(&mut face, &band, 0, top);
    }
    face
}

/// The shell's own colour, from the plain plastic near the top of the face.
fn shell_colour(face: &CartFace) -> [f32; 3] {
    let w = face.w as usize;
    let mut sum = [0f32; 3];
    let mut n = 0f32;
    for y in RECESS_SAMPLE_Y..RECESS_SAMPLE_Y + 12 {
        for x in LYNX_LABEL_X..LYNX_LABEL_X + LYNX_LABEL_W {
            let at = (y as usize * w + x as usize) * 4;
            for (k, total) in sum.iter_mut().enumerate() {
                *total += face.rgba[at + k] as f32;
            }
            n += 1.0;
        }
    }
    sum.map(|c| c / n)
}

fn luminance(c: [f32; 3]) -> f32 {
    (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]) / 255.0
}

/// Repaint the face as the back of the cart. The front's curved line, label
/// recess and contact strip are not on the back: it is one flat face, lit
/// from above, with a lip across the top. The plastic's grain is kept as a
/// faint texture, and the outline keeps a soft rounded edge.
fn flat_back(face: &mut CartFace) {
    let shell = shell_colour(face);
    let (w, h) = (face.w as usize, face.h as usize);
    let alpha: Vec<u8> = face.rgba.chunks_exact(4).map(|px| px[3]).collect();
    let inside = |x: i32, y: i32| {
        x >= 0
            && y >= 0
            && (x as usize) < w
            && (y as usize) < h
            && alpha[y as usize * w + x as usize] > 128
    };
    for y in 0..h {
        for x in 0..w {
            let at = (y * w + x) * 4;
            if alpha[y * w + x] == 0 {
                continue;
            }
            let yy = y as u32;
            let light = if yy < BACK_LIP_TOP {
                // The top of the lip faces the light.
                1.12
            } else if yy < BACK_LIP_H {
                // Its front turns down towards the flat, darkening as it goes.
                let t = (yy - BACK_LIP_TOP) as f32 / (BACK_LIP_H - BACK_LIP_TOP) as f32;
                1.02 - 0.22 * t
            } else if yy < BACK_LIP_H + BACK_LIP_SHADOW {
                // The shadow the lip throws on the flat, fading out.
                let t = (yy - BACK_LIP_H) as f32 / BACK_LIP_SHADOW as f32;
                0.62 + 0.34 * t * t.sqrt()
            } else {
                0.96
            };
            // A rounded edge all the way round: darker the nearer the outline.
            let mut near = 0;
            for d in 1..=EDGE_BEVEL as i32 {
                let (x, y) = (x as i32, y as i32);
                if !(inside(x - d, y) && inside(x + d, y) && inside(x, y - d) && inside(x, y + d)) {
                    near = EDGE_BEVEL as i32 + 1 - d;
                    break;
                }
            }
            let bevel = 1.0 - 0.07 * near as f32;
            let grain = 1.0 + (hash_noise(x as u32, y as u32) - 0.5) * 0.05;
            for (k, c) in shell.iter().enumerate() {
                face.rgba[at + k] = (c * light * bevel * grain).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    // A bright line along the lip's edge, where its top turns down.
    for y in [BACK_LIP_TOP as usize - 1, BACK_LIP_TOP as usize] {
        for x in 0..w {
            let at = (y * w + x) * 4;
            if alpha[y * w + x] > 0 {
                for (k, c) in shell.iter().enumerate() {
                    face.rgba[at + k] = (c * 1.3).round().min(255.0) as u8;
                }
            }
        }
    }
}

const EDGE_BEVEL: u32 = 4;

/// A fixed fine noise, for the moulding's grain.
fn hash_noise(x: u32, y: u32) -> f32 {
    let mut n = x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    (n ^ (n >> 16)) as f32 / u32::MAX as f32
}

/// A row of plain shell above the panel, where the colour is taken from.
const RECESS_SAMPLE_Y: u32 = 40;

pub fn chip_face(core: Option<Core>) -> CartFace {
    let mut face = bitmap_face(bitmaps::chip(), CHIP_W, CHIP_H);
    if let Some(layout) = core.and_then(|c| name_layout(c, (CHIP_W - 8) as f32)) {
        ink_band(&mut face, 4, 34, &layout, CHIP_INK, 1.0);
    }
    padded(&face, TURN_PAD)
}

pub fn chip_shadow_face() -> CartFace {
    let (w, h) = (SHADOW_W, SHADOW_H);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 + 0.5 - w as f32 / 2.0) / (w as f32 / 2.0);
            let dy = (y as f32 + 0.5 - h as f32 / 2.0) / (h as f32 / 2.0);
            let fall = (1.0 - (dx * dx + dy * dy)).max(0.0);
            rgba.extend_from_slice(&[0, 0, 0, (fall * fall * 255.0) as u8]);
        }
    }
    CartFace { rgba, w, h }
}

fn name_layout(core: Core, max_w: f32) -> Option<text::Layout> {
    word_layout(core.text(), max_w)
}

fn word_layout(word: &str, max_w: f32) -> Option<text::Layout> {
    let font = text::label_font()?;
    let tracking = |px: f32| (px * 0.10).round();
    let mut px = NAME_PX;
    while px > NAME_MIN_PX && text::line_width(font, word, px, tracking(px)) > max_w {
        px -= 1.0;
    }
    Some(text::Layout {
        lines: vec![word.to_string()],
        px,
        tracking: tracking(px),
    })
}
