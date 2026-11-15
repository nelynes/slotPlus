//! slotPlus Shelf Life: a cart shows how it has been kept. A game never played comes in
//! shrink wrap, a cart left on the shelf gathers dust, and a well-played one shows its wear.
//! All of it is drawn into the cart's face when the face is built, from a seed taken from the
//! game's name, so the same cart always looks the same way and nothing is drawn per frame.

use slot_store::Cart;

use crate::cart::{gb_shell_of, CartFace};

/// How a cart has been kept. Dust and wear run from 0 (none) to 3.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Look {
    pub wrap: bool,
    pub dust: u8,
    pub wear: u8,
}

pub const MAX_LEVEL: u8 = 3;

/// Days on the shelf, untouched, before each level of dust.
pub const DUST_DAYS: [u64; 3] = [14, 45, 90];
/// Hours of play since the last clean before each level of wear.
pub const WEAR_HOURS: [u64; 3] = [10, 30, 100];

pub fn dust_level(idle_secs: u64) -> u8 {
    DUST_DAYS
        .iter()
        .filter(|d| idle_secs >= **d * 86_400)
        .count() as u8
}

pub fn wear_level(played_secs: u64) -> u8 {
    WEAR_HOURS
        .iter()
        .filter(|h| played_secs >= **h * 3_600)
        .count() as u8
}

impl Look {
    pub fn plain(&self) -> bool {
        !self.wrap && self.dust == 0 && self.wear == 0
    }

    /// Something a wipe would take away.
    pub fn cleanable(&self) -> bool {
        self.dust > 0 || self.wear > 0
    }
}

pub fn seed_of(cart: &Cart) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in cart.stem.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn hash(seed: u64, x: i64, y: i64) -> u64 {
    let mut h = seed
        ^ (x as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (y as u64).wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
    h ^= h >> 31;
    h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h ^= h >> 29;
    h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^ (h >> 32)
}

fn unit(seed: u64, x: i64, y: i64) -> f32 {
    (hash(seed, x, y) % 1_000_000) as f32 / 1_000_000.0
}

/// Smooth value noise, cells `cell` pixels across, 0..1.
fn smooth(seed: u64, x: f32, y: f32, cell: f32) -> f32 {
    let (fx, fy) = (x / cell, y / cell);
    let (ix, iy) = (fx.floor() as i64, fy.floor() as i64);
    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let a = unit(seed, ix, iy);
    let b = unit(seed, ix + 1, iy);
    let c = unit(seed, ix, iy + 1);
    let d = unit(seed, ix + 1, iy + 1);
    let top = a + (b - a) * sx;
    let bottom = c + (d - c) * sx;
    top + (bottom - top) * sy
}

fn mix(px: &mut [u8], to: [u8; 3], amount: f32) {
    let amount = amount.clamp(0.0, 1.0);
    for k in 0..3 {
        let v = px[k] as f32 + (to[k] as f32 - px[k] as f32) * amount;
        px[k] = v.round().clamp(0.0, 255.0) as u8;
    }
}

/// How far each solid pixel is from the cart's outline, up to `max`.
fn edge_distance(face: &CartFace, max: u32) -> Vec<u8> {
    let (w, h) = (face.w as i64, face.h as i64);
    let solid = |x: i64, y: i64| {
        x >= 0 && y >= 0 && x < w && y < h && face.rgba[((y * w + x) * 4 + 3) as usize] >= 128
    };
    let mut out = vec![u8::MAX; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if !solid(x, y) {
                continue;
            }
            let mut best = max as i64 + 1;
            let r = max as i64;
            'look: for d in 1..=r {
                for (dx, dy) in [
                    (d, 0),
                    (-d, 0),
                    (0, d),
                    (0, -d),
                    (d, d),
                    (-d, d),
                    (d, -d),
                    (-d, -d),
                ] {
                    if !solid(x + dx, y + dy) {
                        best = d;
                        break 'look;
                    }
                }
            }
            out[(y * w + x) as usize] = best.min(255) as u8;
        }
    }
    out
}

/// Draw how the cart has been kept into its face.
pub fn apply_look(face: &mut CartFace, look: &Look, seed: u64) {
    if look.plain() || face.w == 0 || face.h == 0 {
        return;
    }
    if look.wear > 0 {
        wear(face, look.wear.min(MAX_LEVEL), seed);
    }
    if look.dust > 0 {
        dust(face, look.dust.min(MAX_LEVEL), seed ^ 0xd057);
    }
    if look.wrap {
        wrap(face, seed ^ 0x5ea1);
    }
}

const DUST: [u8; 3] = [196, 188, 172];
const SPECK: [u8; 3] = [226, 220, 206];

fn dust(face: &mut CartFace, level: u8, seed: u64) {
    let (w, h) = (face.w, face.h);
    let haze = [0.0, 0.10, 0.19, 0.30][level as usize];
    let specks = [0.0, 0.006, 0.014, 0.026][level as usize];
    for y in 0..h {
        // Dust settles thickest along the top, where a standing cart catches it.
        let settle = 1.25 - 0.55 * (y as f32 / h as f32);
        for x in 0..w {
            let at = ((y * w + x) * 4) as usize;
            if face.rgba[at + 3] == 0 {
                continue;
            }
            let (fx, fy) = (x as f32, y as f32);
            let cloud = 0.55 + 0.9 * smooth(seed, fx, fy, 22.0) * smooth(seed ^ 1, fx, fy, 7.0);
            mix(&mut face.rgba[at..at + 4], DUST, haze * settle * cloud);
            let n = unit(seed ^ 2, x as i64, y as i64);
            if n < specks * settle {
                let strength = 0.45 + 0.4 * unit(seed ^ 3, x as i64, y as i64);
                mix(&mut face.rgba[at..at + 4], SPECK, strength);
            }
        }
    }
    // A few bits of fluff: short, faint strands.
    let strands = [0, 2, 5, 9][level as usize];
    for i in 0..strands {
        let x0 = unit(seed ^ 4, i, 0) * w as f32;
        let y0 = unit(seed ^ 4, i, 1) * h as f32 * 0.7;
        let angle = unit(seed ^ 4, i, 2) * std::f32::consts::TAU;
        let len = 4.0 + unit(seed ^ 4, i, 3) * 7.0;
        let bend = (unit(seed ^ 4, i, 4) - 0.5) * 0.25;
        stroke(face, x0, y0, angle, bend, len, SPECK, 0.55);
    }
}

const RUB: [u8; 3] = [232, 228, 218];

fn wear(face: &mut CartFace, level: u8, seed: u64) {
    let (w, h) = (face.w, face.h);
    let edges = edge_distance(face, 3);
    let rub = [0.0, 0.45, 0.65, 0.85][level as usize];
    let reach = [0.0, 0.55, 0.42, 0.30][level as usize];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let at = i * 4;
            if face.rgba[at + 3] < 128 {
                continue;
            }
            let (fx, fy) = (x as f32, y as f32);
            // Corners and edges rub pale where hands hold the cart.
            let d = edges[i];
            if d <= 3 {
                // Long, soft patches, worst at the corners.
                let patchy = smooth(seed, fx, fy, 22.0);
                if patchy > reach {
                    let edge = ((patchy - reach) / (1.0 - reach)).min(0.5) * 2.0;
                    let (cx, cy) = (
                        (fx / w as f32 - 0.5).abs() * 2.0,
                        (fy / h as f32 - 0.5).abs() * 2.0,
                    );
                    let corner = 0.35 + 0.65 * (cx * cy).powf(1.5);
                    let near = (4 - d as u32) as f32 / 3.0;
                    let grain = 0.75 + 0.25 * unit(seed ^ 9, x as i64, y as i64);
                    mix(
                        &mut face.rgba[at..at + 4],
                        RUB,
                        rub * near * grain * edge * corner,
                    );
                }
            }
            // Broad scuffs: patches where the finish has gone dull.
            let scuff = smooth(seed ^ 5, fx, fy, 30.0);
            let cut = [1.0, 0.74, 0.66, 0.58][level as usize];
            if scuff > cut {
                let amount = ((scuff - cut) / (1.0 - cut)) * 0.22;
                let grain = 0.7 + 0.3 * unit(seed ^ 6, x as i64, y as i64);
                mix(&mut face.rgba[at..at + 4], RUB, amount * grain);
            }
        }
    }
    let scratches = [0, 4, 9, 16][level as usize];
    for i in 0..scratches {
        let x0 = unit(seed ^ 7, i, 0) * w as f32;
        let y0 = unit(seed ^ 7, i, 1) * h as f32;
        let angle = (unit(seed ^ 7, i, 2) - 0.5) * 1.2 + if i % 3 == 0 { 1.4 } else { 0.0 };
        let len = 7.0 + unit(seed ^ 7, i, 3) * 20.0;
        let bend = (unit(seed ^ 7, i, 4) - 0.5) * 0.06;
        let strength = 0.30 + 0.30 * unit(seed ^ 7, i, 5);
        stroke(face, x0, y0, angle, bend, len, RUB, strength);
    }
}

/// A thin line on the solid part of the face, fading at both ends.
#[allow(clippy::too_many_arguments)]
fn stroke(
    face: &mut CartFace,
    x0: f32,
    y0: f32,
    angle: f32,
    bend: f32,
    len: f32,
    colour: [u8; 3],
    strength: f32,
) {
    let steps = (len * 2.0).ceil() as u32;
    let (mut x, mut y, mut a) = (x0, y0, angle);
    for s in 0..steps {
        let t = s as f32 / steps.max(1) as f32;
        let taper = (t * (1.0 - t) * 4.0).min(1.0);
        let (px, py) = (x.round() as i64, y.round() as i64);
        if px >= 0 && py >= 0 && (px as u32) < face.w && (py as u32) < face.h {
            let at = ((py as u32 * face.w + px as u32) * 4) as usize;
            if face.rgba[at + 3] >= 128 {
                mix(&mut face.rgba[at..at + 4], colour, strength * taper);
            }
        }
        x += a.cos() * 0.5;
        y += a.sin() * 0.5;
        a += bend * 0.5;
    }
}

fn wrap(face: &mut CartFace, seed: u64) {
    let (w, h) = (face.w, face.h);
    let edges = edge_distance(face, 4);
    // The film's sheen runs across the cart on a slant, in two soft bands.
    let slope = 0.55;
    let span = w as f32 + h as f32 * slope;
    let bands = [
        (0.22 + 0.08 * unit(seed, 0, 0), 0.075, 0.30),
        (0.62 + 0.08 * unit(seed, 1, 0), 0.045, 0.20),
    ];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let at = i * 4;
            if face.rgba[at + 3] < 128 {
                continue;
            }
            let (fx, fy) = (x as f32, y as f32);
            // The film itself: a faint cool lift over everything.
            mix(&mut face.rgba[at..at + 4], [236, 242, 248], 0.07);
            let along = (fx + fy * slope) / span;
            let mut shine = 0.0f32;
            for (centre, half, peak) in bands {
                let d = ((along - centre) / half).abs();
                if d < 1.0 {
                    let wobble = 0.75 + 0.5 * smooth(seed ^ 2, fx, fy, 14.0);
                    shine = shine.max(peak * (1.0 - d * d) * wobble);
                }
            }
            // Crinkles where the film folds over the edges.
            let d = edges[i];
            if d <= 4 {
                let crease = smooth(seed ^ 3, fx * 1.6, fy * 1.6, 5.0);
                if crease > 0.58 {
                    shine = shine.max((crease - 0.58) * 1.6 * (5 - d as u32) as f32 / 4.0);
                }
            }
            if shine > 0.0 {
                mix(&mut face.rgba[at..at + 4], [255, 255, 255], shine);
            }
        }
    }
    // A seam where the film was sealed, low across the back of the label.
    let seam_y = h as f32 * (0.80 + 0.05 * unit(seed, 2, 0));
    for x in 0..w {
        let wob = (smooth(seed ^ 4, x as f32, 0.0, 18.0) - 0.5) * 3.0;
        let y = (seam_y + wob).round() as i64;
        for (dy, amount) in [(0, 0.28), (1, -0.12)] {
            let yy = y + dy;
            if yy < 0 || yy as u32 >= h {
                continue;
            }
            let at = ((yy as u32 * w + x) * 4) as usize;
            if face.rgba[at + 3] < 128 {
                continue;
            }
            if amount > 0.0 {
                mix(&mut face.rgba[at..at + 4], [255, 255, 255], amount);
            } else {
                mix(&mut face.rgba[at..at + 4], [0, 0, 0], -amount);
            }
        }
    }
}

/// Where a play-time sticker goes on the back of a cart: on the board of a GBA or Game Boy
/// or WonderSwan cart, and low on the back of a Lynx card. Other carts have no back to
/// show yet.
pub fn sticker_at(cart: &Cart, back: &CartFace) -> Option<(u32, u32)> {
    use slot_store::Platform;
    let (sw, sh) = (STICKER_W, STICKER_H);
    let spot = match cart.platform {
        Platform::Lynx => (
            back.w.saturating_sub(sw + 16),
            back.h.saturating_sub(sh + 12),
        ),
        Platform::Gba | Platform::Gbav => (224, 56),
        Platform::Ws => (250, 104),
        Platform::Gb | Platform::Gbc if gb_shell_of(cart).is_some() => (28, 14),
        _ => return None,
    };
    (spot.0 + sw <= back.w && spot.1 + sh <= back.h).then_some(spot)
}

pub const STICKER_W: u32 = 76;
pub const STICKER_H: u32 = 26;
const STICKER_PAPER: [u8; 3] = [244, 240, 226];
const STICKER_INK: [u8; 3] = [38, 44, 80];

/// A small paper sticker with the time played written on it by hand.
pub fn play_sticker(back: &mut CartFace, cart: &Cart, text: &str) {
    let Some((x0, y0)) = sticker_at(cart, back) else {
        return;
    };
    let seed = seed_of(cart);
    let (sw, sh) = (STICKER_W, STICKER_H);
    // A slight tilt, as a sticker put on by hand.
    let tilt = (unit(seed, 8, 8) - 0.5) * 0.06;
    let mut paper = vec![0u8; (sw * sh * 4) as usize];
    for y in 0..sh {
        for x in 0..sw {
            let at = ((y * sw + x) * 4) as usize;
            let r = 3.0;
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let cx = fx.clamp(r, sw as f32 - r);
            let cy = fy.clamp(r, sh as f32 - r);
            if (fx - cx).powi(2) + (fy - cy).powi(2) > r * r {
                continue;
            }
            let grain = 0.96 + 0.04 * unit(seed ^ 11, x as i64, y as i64);
            for k in 0..3 {
                paper[at + k] = (STICKER_PAPER[k] as f32 * grain) as u8;
            }
            paper[at + 3] = 255;
        }
    }
    if let Some(hand) = crate::hand::hand_font() {
        let lines = [text.to_string()];
        let mut px = 17.0;
        while px > 9.0 && hand.line_width(text, px) > (sw - 8) as f32 {
            px -= 1.0;
        }
        for (i, a) in hand.coverage(sw, sh, &lines, px).into_iter().enumerate() {
            if a == 0 || paper[i * 4 + 3] == 0 {
                continue;
            }
            mix(&mut paper[i * 4..i * 4 + 4], STICKER_INK, a as f32 / 255.0);
        }
    }
    // Place it, turned a touch, with a soft shadow under the lower edge.
    let (cx, cy) = (sw as f32 / 2.0, sh as f32 / 2.0);
    let (sin, cos) = tilt.sin_cos();
    for y in 0..sh + 4 {
        for x in 0..sw + 4 {
            let (dx, dy) = (x as f32 - 2.0 - cx, y as f32 - 2.0 - cy);
            let u = dx * cos + dy * sin + cx;
            let v = -dx * sin + dy * cos + cy;
            let (bx, by) = (x0 as i64 + x as i64 - 2, y0 as i64 + y as i64 - 2);
            if bx < 0 || by < 0 || bx as u32 >= back.w || by as u32 >= back.h {
                continue;
            }
            let at = ((by as u32 * back.w + bx as u32) * 4) as usize;
            if back.rgba[at + 3] == 0 {
                continue;
            }
            let (ui, vi) = (u.floor() as i64, v.floor() as i64);
            let shadow = (ui, vi - 1);
            let on = |(px, py): (i64, i64)| {
                px >= 0
                    && py >= 0
                    && (px as u32) < sw
                    && (py as u32) < sh
                    && paper[((py as u32 * sw + px as u32) * 4 + 3) as usize] > 0
            };
            if on((ui, vi)) {
                let s = ((vi as u32 * sw + ui as u32) * 4) as usize;
                back.rgba[at..at + 3].copy_from_slice(&paper[s..s + 3]);
            } else if on(shadow) {
                mix(&mut back.rgba[at..at + 4], [0, 0, 0], 0.35);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dust_and_wear_build_in_steps() {
        assert_eq!(dust_level(0), 0);
        assert_eq!(dust_level(14 * 86_400), 1);
        assert_eq!(dust_level(60 * 86_400), 2);
        assert_eq!(dust_level(400 * 86_400), 3);
        assert_eq!(wear_level(9 * 3_600), 0);
        assert_eq!(wear_level(31 * 3_600), 2);
        assert_eq!(wear_level(1_000 * 3_600), 3);
    }

    #[test]
    fn a_plain_look_leaves_the_face_alone_and_the_rest_keep_its_outline() {
        let mut face = CartFace {
            rgba: (0..40 * 30)
                .flat_map(|i| {
                    let inside = (i % 40) > 3 && (i % 40) < 36 && (i / 40) > 2 && (i / 40) < 27;
                    [60, 60, 70, if inside { 255 } else { 0 }]
                })
                .collect(),
            w: 40,
            h: 30,
        };
        let before = face.rgba.clone();
        apply_look(&mut face, &Look::default(), 7);
        assert_eq!(face.rgba, before);
        apply_look(
            &mut face,
            &Look {
                wrap: true,
                dust: 3,
                wear: 3,
            },
            7,
        );
        assert_ne!(face.rgba, before);
        for (a, b) in face.rgba.chunks(4).zip(before.chunks(4)) {
            assert_eq!(a[3], b[3]);
            if b[3] == 0 {
                assert_eq!(a, b);
            }
        }
    }
}
