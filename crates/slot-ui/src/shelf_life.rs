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

/// Smooth value noise over a face, cells `cell` pixels across, 0..1. The lattice is hashed
/// once, so a pixel costs a few multiplies rather than four hashes.
struct Field {
    cell: f32,
    cols: usize,
    vals: Vec<f32>,
}

impl Field {
    fn new(seed: u64, w: u32, h: u32, cell: f32) -> Field {
        let cols = (w as f32 / cell) as usize + 2;
        let rows = (h as f32 / cell) as usize + 2;
        let mut vals = Vec::with_capacity(cols * rows);
        for y in 0..rows {
            for x in 0..cols {
                vals.push(unit(seed, x as i64, y as i64));
            }
        }
        Field { cell, cols, vals }
    }

    fn at(&self, x: u32, y: u32) -> f32 {
        let (fx, fy) = (x as f32 / self.cell, y as f32 / self.cell);
        let (ix, iy) = (fx as usize, fy as usize);
        let (tx, ty) = (fx - ix as f32, fy - iy as f32);
        let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
        let i = iy * self.cols + ix;
        let (a, b) = (self.vals[i], self.vals[i + 1]);
        let (c, d) = (self.vals[i + self.cols], self.vals[i + self.cols + 1]);
        let top = a + (b - a) * sx;
        let bottom = c + (d - c) * sx;
        top + (bottom - top) * sy
    }
}

/// Smooth value noise at one point, for the few places that need only a handful of samples.
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
    if look.wrap {
        wrap(face, seed ^ 0x5ea1);
    }
    // Dust settles on whatever is outermost, the film included.
    if look.dust > 0 {
        dust(face, look.dust.min(MAX_LEVEL), seed ^ 0xd057);
    }
}

const DUST: [u8; 3] = [196, 188, 172];
const SPECK: [u8; 3] = [226, 220, 206];

fn dust(face: &mut CartFace, level: u8, seed: u64) {
    let (w, h) = (face.w, face.h);
    let haze = [0.0, 0.10, 0.19, 0.30][level as usize];
    let specks = [0.0, 0.006, 0.014, 0.026][level as usize];
    let (broad, fine) = (
        Field::new(seed, w, h, 22.0),
        Field::new(seed ^ 1, w, h, 7.0),
    );
    for y in 0..h {
        // Dust settles thickest along the top, where a standing cart catches it.
        let settle = 1.25 - 0.55 * (y as f32 / h as f32);
        for x in 0..w {
            let at = ((y * w + x) * 4) as usize;
            if face.rgba[at + 3] == 0 {
                continue;
            }
            let cloud = 0.55 + 0.9 * broad.at(x, y) * fine.at(x, y);
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
    let (patches, scuffs) = (
        Field::new(seed, w, h, 22.0),
        Field::new(seed ^ 5, w, h, 30.0),
    );
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
                let patchy = patches.at(x, y);
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
            let scuff = scuffs.at(x, y);
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
            if face.rgba[at + 3] >= 40 {
                mix(&mut face.rgba[at..at + 4], colour, strength * taper);
            }
        }
        x += a.cos() * 0.5;
        y += a.sin() * 0.5;
        a += bend * 0.5;
    }
}

/// The outline of the solid part of the face, as a convex polygon (Andrew's monotone chain
/// over each row's ends): the shape a sheet of film pulled tight around the cart takes.
fn hull(face: &CartFace) -> Vec<(f32, f32)> {
    let (w, h) = (face.w as usize, face.h as usize);
    let mut pts = Vec::new();
    for y in 0..h {
        let row = &face.rgba[y * w * 4..(y + 1) * w * 4];
        let solid = |x: usize| row[x * 4 + 3] >= 128;
        if let Some(l) = (0..w).find(|&x| solid(x)) {
            let r = (0..w).rev().find(|&x| solid(x)).unwrap_or(l);
            pts.push((l as f32, y as f32));
            pts.push((r as f32 + 1.0, y as f32));
            pts.push((l as f32, y as f32 + 1.0));
            pts.push((r as f32 + 1.0, y as f32 + 1.0));
        }
    }
    pts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: (f32, f32), a: (f32, f32), b: (f32, f32)| {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };
    let mut lower: Vec<(f32, f32)> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<(f32, f32)> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// How far inside the convex polygon a point is (negative outside).
fn inside_by(poly: &[(f32, f32)], x: f32, y: f32) -> f32 {
    let mut best = f32::INFINITY;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let len = (ex * ex + ey * ey).sqrt().max(1e-3);
        // The polygon winds counter-clockwise in y-down space; inside is to the left.
        let d = (ex * (y - a.1) - ey * (x - a.0)) / len;
        best = best.min(d);
    }
    best
}

const FILM: [u8; 3] = [238, 243, 250];

/// Shrink wrap: a loose sheet of clear film around the cart. The cart sits a little smaller
/// inside it; the film stands off its edges and bridges its notches and rounded corners, so
/// the outline turns soft and glassy; a cloudy layer lies over everything, with two glossy
/// bands, creases at the pinched corners and in from the edges, a crimped heat seal along
/// the top, and the shop's price sticker.
fn wrap(face: &mut CartFace, seed: u64) {
    let (w, h) = (face.w, face.h);
    // The cart sits a little smaller inside the face, so the film has room to stand off it.
    shrink(face, FILM_SLACK);
    let poly = hull(face);
    if poly.len() < 3 {
        return;
    }
    let slack = FILM_SLACK as f32;
    // The sheen: two slanted bands with a brighter core, as a glossy sheet reflects a window.
    let slope = 0.55;
    let span = w as f32 + h as f32 * slope;
    let bands = [
        (0.24 + 0.08 * unit(seed, 0, 0), 0.085, 0.26),
        (0.64 + 0.08 * unit(seed, 1, 0), 0.05, 0.18),
    ];
    let sheen = |fx: f32, fy: f32| -> f32 {
        let along = (fx + fy * slope) / span;
        if !bands
            .iter()
            .any(|(centre, half, _)| (along - centre).abs() < half + 0.02)
        {
            return 0.0;
        }
        let wobble = (smooth(seed ^ 2, fx, fy, 30.0) - 0.5) * 0.03;
        let mut shine = 0.0f32;
        for (centre, half, peak) in bands {
            let d = ((along - centre + wobble) / half).abs();
            if d < 1.0 {
                let core = (-(d / 0.14).powi(2)).exp();
                shine = shine.max(peak * ((1.0 - d * d) + 0.5 * core));
            }
        }
        shine
    };
    // The cloudy lift over the cart, a fixed blend toward the film's colour, in 256ths.
    const CLOUD: u32 = 31;
    for y in 0..h {
        for x in 0..w {
            let at = ((y * w + x) * 4) as usize;
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let a = face.rgba[at + 3];
            if a == 255 {
                // Over the cart, well inside the film.
                let px = &mut face.rgba[at..at + 3];
                for (c, film) in px.iter_mut().zip(FILM) {
                    *c = ((*c as u32 * (256 - CLOUD) + film as u32 * CLOUD) >> 8) as u8;
                }
            } else {
                // The film hangs loose: out past the cart by the slack, its edge a little
                // wavy. Thin film you can see through, brightest along its own edge.
                let wave = (smooth(seed ^ 9, fx, fy, 16.0) - 0.5) * 2.4;
                let d = inside_by(&poly, fx, fy) + slack - 1.0 + wave;
                if d < -0.5 {
                    continue;
                }
                let edge = (1.0 - d / 2.0).clamp(0.0, 1.0);
                let cover = (d + 0.5).clamp(0.0, 1.0);
                let fa = (58.0 + 130.0 * edge) * cover / 255.0;
                let under = a as f32 / 255.0;
                let out_a = fa + under * (1.0 - fa);
                if out_a <= 0.0 {
                    continue;
                }
                for (k, film) in FILM.iter().enumerate() {
                    let c =
                        (*film as f32 * fa + face.rgba[at + k] as f32 * under * (1.0 - fa)) / out_a;
                    face.rgba[at + k] = c.round().clamp(0.0, 255.0) as u8;
                }
                face.rgba[at + 3] = (out_a * 255.0).round() as u8;
            }
            let shine = sheen(fx, fy);
            if shine > 0.0 {
                mix(&mut face.rgba[at..at + 4], [255, 255, 255], shine);
            }
        }
    }
    // Creases: short folds, a bright ridge with a faint shadow beside it, fanning out of
    // the corners where the film was pinched and running in from the edges.
    let corners: Vec<(f32, f32)> = poly
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            let n = poly.len();
            let (a, b) = (poly[(i + n - 1) % n], poly[(i + 1) % n]);
            let (ax, ay, bx, by) = (p.0 - a.0, p.1 - a.1, b.0 - p.0, b.1 - p.1);
            let turn = (ax * by - ay * bx).abs()
                / ((ax * ax + ay * ay).sqrt() * (bx * bx + by * by).sqrt()).max(1e-3);
            turn > 0.5
        })
        .map(|(_, p)| *p)
        .collect();
    let crease = |face: &mut CartFace, x: f32, y: f32, angle: f32, len: f32, k: f32| {
        let (nx, ny) = (-angle.sin(), angle.cos());
        stroke(face, x + nx, y + ny, angle, 0.0, len, [0, 0, 0], k * 0.3);
        stroke(face, x, y, angle, 0.0, len, [255, 255, 255], k);
    };
    let mut n = 0i64;
    for &(cx, cy) in &corners {
        let mid = (h as f32 / 2.0 - cy).atan2(w as f32 / 2.0 - cx);
        for k in 0..3 {
            n += 1;
            let spread = (unit(seed ^ 5, n, 0) - 0.5) * 1.2;
            let len = 5.0 + unit(seed ^ 5, n, 1) * 9.0;
            let start = 1.0 + k as f32;
            let (sx, sy) = (cx + mid.cos() * start, cy + mid.sin() * start);
            crease(face, sx, sy, mid + spread, len, 0.45);
        }
    }
    for _ in 0..26 {
        n += 1;
        let t = unit(seed ^ 6, n, 0);
        let i = ((t * poly.len() as f32) as usize).min(poly.len() - 1);
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let f = unit(seed ^ 6, n, 1);
        let (x0, y0) = (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
        let inward = (b.1 - a.1).atan2(b.0 - a.0) + std::f32::consts::FRAC_PI_2;
        let angle = inward + (unit(seed ^ 6, n, 2) - 0.5) * 1.2;
        let len = 3.0 + unit(seed ^ 6, n, 3) * 6.0;
        crease(face, x0, y0, angle, len, 0.4);
    }
    // The heat seal: a crimped seam along the top of the film.
    let top = poly.iter().map(|p| p.1).fold(f32::INFINITY, f32::min) - slack + 1.0;
    let seam = top + 3.0;
    for x in 0..w {
        let fx = x as f32 + 0.5;
        if inside_by(&poly, fx, seam) + slack < 4.0 {
            continue;
        }
        let wob = (smooth(seed ^ 8, fx, 0.0, 22.0) - 0.5) * 1.6;
        let crimp = if x % 3 == 0 { 0.55 } else { 0.25 };
        for (dy, colour, k) in [(0.0, [255, 255, 255], crimp), (1.0, [0, 0, 0], 0.12)] {
            let y = (seam + wob + dy).round() as i64;
            if y < 0 || y as u32 >= h {
                continue;
            }
            let at = ((y as u32 * w + x) * 4) as usize;
            if face.rgba[at + 3] > 0 {
                mix(&mut face.rgba[at..at + 4], colour, k);
            }
        }
    }
    price_sticker(face, seed);
}

/// How far the film stands off the cart, in pixels.
const FILM_SLACK: u32 = 5;

/// Scale the face down about its middle so `margin` pixels are left clear on its short
/// sides.
fn shrink(face: &mut CartFace, margin: u32) {
    let (w, h) = (face.w as usize, face.h as usize);
    let short = w.min(h) as f32;
    if short <= 4.0 * margin as f32 {
        return;
    }
    let k = short / (short - 2.0 * margin as f32);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let src = face.rgba.clone();
    // Where each column and row falls in the full-size face: the pixel before it, and how
    // far on toward the next, in 256ths.
    let map = |n: usize, c: f32| -> Vec<(i64, u32)> {
        (0..n)
            .map(|i| {
                let s = (i as f32 + 0.5 - c) * k + c - 0.5;
                let f = s.floor();
                (f as i64, ((s - f) * 256.0) as u32)
            })
            .collect()
    };
    let (xs, ys) = (map(w, cx), map(h, cy));
    let texel = |x: i64, y: i64| -> [u32; 4] {
        if x < 0 || y < 0 || x as usize >= w || y as usize >= h {
            return [0; 4];
        }
        let p = &src[(y as usize * w + x as usize) * 4..][..4];
        [p[0] as u32, p[1] as u32, p[2] as u32, p[3] as u32]
    };
    for (y, &(y0, ty)) in ys.iter().enumerate() {
        for (x, &(x0, tx)) in xs.iter().enumerate() {
            let taps = [
                (texel(x0, y0), (256 - tx) * (256 - ty)),
                (texel(x0 + 1, y0), tx * (256 - ty)),
                (texel(x0, y0 + 1), (256 - tx) * ty),
                (texel(x0 + 1, y0 + 1), tx * ty),
            ];
            let at = (y * w + x) * 4;
            let out = &mut face.rgba[at..at + 4];
            if taps.iter().all(|(t, _)| t[3] == 255) {
                // The common case, well inside the cart: no alpha to weigh.
                for c in 0..3 {
                    let sum: u32 = taps.iter().map(|(t, wgt)| t[c] * wgt).sum();
                    out[c] = ((sum + (1 << 15)) >> 16) as u8;
                }
                out[3] = 255;
                continue;
            }
            // Weighed by alpha, so clear pixels don't darken the edge.
            let alpha: u64 = taps.iter().map(|(t, wgt)| (t[3] * wgt) as u64).sum();
            if alpha < (1 << 15) {
                out.copy_from_slice(&[0, 0, 0, 0]);
                continue;
            }
            for c in 0..3 {
                let sum: u64 = taps
                    .iter()
                    .map(|(t, wgt)| t[c] as u64 * t[3] as u64 * *wgt as u64)
                    .sum();
                out[c] = ((sum + alpha / 2) / alpha).min(255) as u8;
            }
            out[3] = ((alpha + (1 << 15)) >> 16).min(255) as u8;
        }
    }
}

/// What a shop might have charged, picked by the game's own seed.
const PRICES: [&str; 8] = [
    "9.99", "14.99", "19.95", "24.99", "29.95", "34.99", "39.95", "49.99",
];

const STAMP_INK: [u8; 3] = [52, 44, 92];
const TAG_WHITE: [u8; 3] = [232, 230, 224];
const TAG_YELLOW: [u8; 3] = [226, 200, 52];
const TAG_RED: [u8; 3] = [206, 44, 40];

/// A small sheet of sticker paper being printed on.
struct Tag {
    rgba: Vec<u8>,
    w: u32,
    h: u32,
    seed: u64,
}

impl Tag {
    /// `shape(x, y)` says which pixels are paper.
    fn cut(w: u32, h: u32, colour: [u8; 3], seed: u64, shape: impl Fn(f32, f32) -> bool) -> Tag {
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                if !shape(x as f32 + 0.5, y as f32 + 0.5) {
                    continue;
                }
                let at = ((y * w + x) * 4) as usize;
                // Worn paper: grain, and the odd scuff.
                let grain = 0.93 + 0.07 * unit(seed ^ 21, x as i64, y as i64);
                let scuff = 1.0 - 0.10 * smooth(seed ^ 22, x as f32, y as f32, 7.0);
                for (k, c) in colour.iter().enumerate() {
                    rgba[at + k] = (*c as f32 * grain * scuff) as u8;
                }
                rgba[at + 3] = 255;
            }
        }
        Tag { rgba, w, h, seed }
    }

    /// Text in a box, as printed (`stamped` false) or as a price gun leaves it: uneven,
    /// with ink missing here and there.
    #[allow(clippy::too_many_arguments)]
    fn print(
        &mut self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        words: &str,
        px: f32,
        ink: [u8; 3],
        stamped: bool,
    ) {
        let layout = crate::text::Layout {
            lines: vec![words.to_string()],
            px,
            tracking: 0.0,
        };
        let cov = crate::text::coverage(w, h, &layout);
        for by in 0..h {
            for bx in 0..w {
                let a = cov[(by * w + bx) as usize];
                let (tx, ty) = (x + bx, y + by);
                if a == 0 || tx >= self.w || ty >= self.h {
                    continue;
                }
                let at = ((ty * self.w + tx) * 4) as usize;
                if self.rgba[at + 3] == 0 {
                    continue;
                }
                let mut k = a as f32 / 255.0;
                if stamped {
                    let patchy = smooth(self.seed ^ 23, tx as f32, ty as f32, 5.0);
                    let speck = unit(self.seed ^ 24, tx as i64, ty as i64);
                    k *= (0.55 + 0.75 * patchy).min(1.0) * if speck < 0.12 { 0.35 } else { 0.9 };
                }
                mix(&mut self.rgba[at..at + 4], ink, k);
            }
        }
    }

    fn rule(&mut self, x0: u32, x1: u32, y: u32, ink: [u8; 3], k: f32) {
        for x in x0..x1.min(self.w) {
            if y < self.h {
                let at = ((y * self.w + x) * 4) as usize;
                if self.rgba[at + 3] > 0 {
                    mix(&mut self.rgba[at..at + 4], ink, k);
                }
            }
        }
    }
}

/// A price-gun label's outline: a rectangle with a small notch in the middle of its top
/// and bottom edges.
fn notched(w: f32, h: f32) -> impl Fn(f32, f32) -> bool {
    move |x, y| {
        let notch = |cy: f32| ((x - w / 2.0).powi(2) + (y - cy).powi(2)).sqrt() < 4.0;
        x > 0.5 && y > 0.5 && x < w - 0.5 && y < h - 0.5 && !notch(0.0) && !notch(h)
    }
}

/// The other common label: straight top and bottom, its ends scalloped.
fn scalloped(w: f32, h: f32) -> impl Fn(f32, f32) -> bool {
    move |x, y| {
        let bite = |cx: f32, cy: f32| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() < h * 0.2;
        let e = 9.0;
        let bulge =
            |cx: f32| ((x - cx).powi(2) / (e * e) + (y - h / 2.0).powi(2) / (h * h * 0.09)) <= 1.0;
        let body = x > e && x < w - e && y > 0.5 && y < h - 0.5;
        (body && !bite(e, 0.0) && !bite(e, h) && !bite(w - e, 0.0) && !bite(w - e, h))
            || bulge(e)
            || bulge(w - e)
    }
}

/// slotPlus Shelf Life: the shop's price sticker on a wrapped game. Four kinds, and where
/// one goes, what it says and how straight it was put on all come from the game's seed, so a
/// game always carries the same one.
fn price_sticker(face: &mut CartFace, seed: u64) {
    let pick = |n: i64, of: usize| (hash(seed ^ 0x9a1c, n, 0) % of as u64) as usize;
    let price = PRICES[pick(1, PRICES.len())];
    // Sizes are worked out at a small scale and printed a third larger, to read on the shelf.
    let z = |v: f32| (v * 1.3).round() as u32;
    let zf = |v: f32| v * 1.3;
    let dark = [30, 28, 20];
    let tag = match pick(0, 4) {
        // Marked down: the old price struck through, the new one large.
        0 => {
            let was = 3 + pick(1, PRICES.len() - 3);
            let now = pick(2, was);
            let (w, h) = (z(62.0), z(44.0));
            let mut t = Tag::cut(w, h, TAG_YELLOW, seed, move |x, y| {
                x > 0.5 && y > 0.5 && x < w as f32 - 0.5 && y < h as f32 - 0.5
            });
            t.print(
                z(2.0),
                z(2.0),
                z(26.0),
                z(7.0),
                "ORIGINAL",
                zf(5.5),
                dark,
                false,
            );
            t.print(
                z(2.0),
                z(8.0),
                z(17.0),
                z(7.0),
                "PRICE",
                zf(5.5),
                dark,
                false,
            );
            let old = format!("${}", PRICES[was]);
            t.print(
                z(27.0),
                z(3.0),
                z(34.0),
                z(12.0),
                &old,
                zf(9.0),
                STAMP_INK,
                true,
            );
            t.rule(z(29.0), z(60.0), z(9.0), dark, 0.85);
            t.print(0, z(16.0), w, z(10.0), "DISCOUNTED", zf(8.5), dark, false);
            let new = format!("${}", PRICES[now]);
            t.print(0, z(25.0), w, z(18.0), &new, zf(15.0), STAMP_INK, true);
            t
        }
        // A price gun's label with a red line top and bottom.
        1 => {
            let (w, h) = (z(66.0), z(30.0));
            let mut t = Tag::cut(w, h, TAG_WHITE, seed, notched(w as f32, h as f32));
            t.rule(z(3.0), w - z(3.0), z(5.0), TAG_RED, 0.85);
            t.rule(z(3.0), w - z(3.0), h - z(5.0), TAG_RED, 0.85);
            let words = format!("${price}");
            t.print(0, z(7.0), w, z(17.0), &words, zf(14.0), STAMP_INK, true);
            t
        }
        // The scalloped label: a stock number over the price.
        2 => {
            let code = 10_000_000 + hash(seed ^ 0x51c, 3, 0) % 89_999_999;
            let (w, h) = (z(72.0), z(34.0));
            let mut t = Tag::cut(w, h, TAG_WHITE, seed, scalloped(w as f32, h as f32));
            let stock = format!("B  {code}");
            t.print(
                z(4.0),
                z(3.0),
                w - z(8.0),
                z(10.0),
                &stock,
                zf(7.5),
                STAMP_INK,
                true,
            );
            let words = format!("$ {price}");
            t.print(
                z(4.0),
                z(13.0),
                w - z(8.0),
                z(18.0),
                &words,
                zf(14.0),
                STAMP_INK,
                true,
            );
            t
        }
        // The shop's own label: its name in red, the price fading.
        _ => {
            let (w, h) = (z(64.0), z(30.0));
            let mut t = Tag::cut(w, h, TAG_WHITE, seed, notched(w as f32, h as f32));
            t.print(
                z(14.0),
                z(2.0),
                z(48.0),
                z(10.0),
                "Slot Shop",
                zf(8.0),
                TAG_RED,
                false,
            );
            let words = format!("${price}");
            t.print(
                0,
                z(11.0),
                w,
                z(17.0),
                &words,
                zf(13.0),
                [120, 124, 132],
                true,
            );
            t
        }
    };
    // Up in a corner of the film, put on by hand and not quite straight.
    let (w, h) = (face.w as i64, face.h as i64);
    let inset = FILM_SLACK as i64 + 6 + pick(5, 8) as i64;
    let right = pick(6, 3) != 0;
    let low = pick(7, 4) == 0;
    let x0 = if right {
        w - tag.w as i64 - inset
    } else {
        inset
    };
    let y0 = if low { h - tag.h as i64 - inset } else { inset };
    let tilt = (unit(seed ^ 0x711, 8, 8) - 0.5) * 0.28;
    paste(
        face,
        &tag.rgba,
        tag.w,
        tag.h,
        (x0.max(0) as u32, y0.max(0) as u32),
        tilt,
    );
}

/// Stick a small sheet onto a face, turned by `tilt`, with a soft shadow under its lower
/// edge. It only sticks where there is something to stick to.
fn paste(back: &mut CartFace, paper: &[u8], sw: u32, sh: u32, (x0, y0): (u32, u32), tilt: f32) {
    let (cx, cy) = (sw as f32 / 2.0, sh as f32 / 2.0);
    let (sin, cos) = tilt.sin_cos();
    let pad = (sw.max(sh) as f32 * tilt.abs().sin()).ceil() as i64 + 3;
    for y in -pad..sh as i64 + pad {
        for x in -pad..sw as i64 + pad {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let u = dx * cos + dy * sin + cx;
            let v = -dx * sin + dy * cos + cy;
            let (bx, by) = (x0 as i64 + x, y0 as i64 + y);
            if bx < 0 || by < 0 || bx as u32 >= back.w || by as u32 >= back.h {
                continue;
            }
            let at = ((by as u32 * back.w + bx as u32) * 4) as usize;
            if back.rgba[at + 3] == 0 {
                continue;
            }
            let (ui, vi) = (u.floor() as i64, v.floor() as i64);
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
                back.rgba[at + 3] = 255;
            } else if on((ui, vi - 1)) {
                mix(&mut back.rgba[at..at + 4], [0, 0, 0], 0.35);
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
    let Some(at) = sticker_at(cart, back) else {
        return;
    };
    sticker(back, cart, text, at, STICKER_PAPER, STICKER_INK, 0);
}

/// slotPlus: where the gold achievements sticker goes, beside the play-time one.
pub fn gold_sticker_at(cart: &Cart, back: &CartFace) -> Option<(u32, u32)> {
    use slot_store::Platform;
    let (sw, sh) = (STICKER_W, STICKER_H);
    let spot = match cart.platform {
        Platform::Lynx => (16, back.h.saturating_sub(sh + 12)),
        Platform::Gba | Platform::Gbav => (224, 27),
        Platform::Ws => (66, 114),
        Platform::Gb | Platform::Gbc if gb_shell_of(cart).is_some() => (28, 46),
        _ => return None,
    };
    (spot.0 + sw <= back.w && spot.1 + sh <= back.h).then_some(spot)
}

const GOLD_PAPER: [u8; 3] = [232, 192, 84];
const GOLD_INK: [u8; 3] = [74, 48, 6];

/// slotPlus: a gold sticker with how far the game's achievement set has got: "12 / 34", or
/// "Mastered" once every one is earned.
pub fn achievement_sticker(back: &mut CartFace, cart: &Cart, text: &str) {
    let Some(at) = gold_sticker_at(cart, back) else {
        return;
    };
    sticker(back, cart, text, at, GOLD_PAPER, GOLD_INK, 1);
}

fn sticker(
    back: &mut CartFace,
    cart: &Cart,
    text: &str,
    (x0, y0): (u32, u32),
    paper_colour: [u8; 3],
    ink: [u8; 3],
    which: i64,
) {
    let seed = seed_of(cart) ^ (which as u64).wrapping_mul(0x9e37_79b9);
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
                paper[at + k] = (paper_colour[k] as f32 * grain) as u8;
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
            mix(&mut paper[i * 4..i * 4 + 4], ink, a as f32 / 255.0);
        }
    }
    paste(back, &paper, sw, sh, (x0, y0), tilt);
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
                wrap: false,
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

    fn block(w: u32, h: u32) -> CartFace {
        CartFace {
            rgba: (0..w * h).flat_map(|_| [60, 60, 70, 255]).collect(),
            w,
            h,
        }
    }

    #[test]
    fn shrink_wrap_stands_off_the_cart_as_film_you_can_see_through() {
        let mut face = block(120, 80);
        apply_look(
            &mut face,
            &Look {
                wrap: true,
                dust: 0,
                wear: 0,
            },
            7,
        );
        let alpha = |x: u32, y: u32| face.rgba[((y * 120 + x) * 4 + 3) as usize];
        // The cart has drawn in from the face's edge, and film fills the gap.
        let rim = alpha(5, 40);
        assert!(rim > 0 && rim < 255, "the edge is {rim}, not film");
        assert_eq!(alpha(60, 40), 255);
        assert_eq!((face.w, face.h), (120, 80), "the face keeps its size");
    }

    #[test]
    fn a_game_always_carries_the_same_price_sticker_and_games_differ() {
        let wrapped = |seed: u64| {
            let mut face = block(360, 212);
            apply_look(
                &mut face,
                &Look {
                    wrap: true,
                    dust: 0,
                    wear: 0,
                },
                seed,
            );
            face.rgba
        };
        assert_eq!(wrapped(11), wrapped(11));
        let kinds: std::collections::HashSet<Vec<u8>> = (0..12).map(wrapped).collect();
        assert!(
            kinds.len() > 6,
            "only {} looks in twelve games",
            kinds.len()
        );
    }

    #[test]
    fn a_marked_down_price_is_lower_than_the_one_struck_out() {
        for seed in 0..200u64 {
            let pick = |n: i64, of: usize| (hash(seed ^ 0x9a1c, n, 0) % of as u64) as usize;
            let was = 3 + pick(1, PRICES.len() - 3);
            assert!(pick(2, was) < was && was < PRICES.len());
        }
    }

    #[test]
    #[ignore = "timing, run by hand"]
    fn how_long_a_wrap_takes() {
        let t = std::time::Instant::now();
        for seed in 0..20u64 {
            let mut face = block(360, 212);
            apply_look(
                &mut face,
                &Look {
                    wrap: true,
                    dust: 0,
                    wear: 0,
                },
                seed,
            );
        }
        eprintln!("wrap: {:.2} ms each", t.elapsed().as_secs_f64() * 50.0);
    }
}
