//! slotPlus Shelf Life: a cart shows how it has been kept. A game never played comes in
//! shrink wrap, a cart left on the shelf gathers dust, and a well-played one shows its wear.
//! All of it is drawn into the cart's face when the face is built, from a seed taken from the
//! game's name, so the same cart always looks the same way and nothing is drawn per frame.

use slot_store::Cart;

use crate::cart::{gb_shell_of, CartFace};

/// How a cart has been kept. Dust runs from 0 (none) to 3, wear from 0 to 4.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Look {
    pub wrap: bool,
    pub dust: u8,
    pub wear: u8,
}

pub const MAX_LEVEL: u8 = 3;
/// Wear goes one further than dust. The last level isn't in the docs: it's left for
/// players to find.
pub const MAX_WEAR: u8 = 4;

/// Days on the shelf, untouched, before each level of dust.
pub const DUST_DAYS: [u64; 3] = [14, 45, 90];
/// Hours of play since the last clean before each level of wear.
pub const WEAR_HOURS: [u64; 4] = [3, 8, 25, 100];

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

/// slotPlus Shelf Life: the money on a wrapped game's price sticker, from where it was sold.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Money {
    /// US dollars, and the answer whenever the release is American, worldwide, mixed or
    /// unknown.
    #[default]
    Dollar,
    /// A Japan-only release.
    Yen,
    /// A release for Europe only.
    Euro,
}

/// Region tags in a No-Intro name that mean Europe. The UK sold in pounds, but it is
/// Europe on these games' boxes, and three currencies are plenty for a sticker.
const EUROPE: [&str; 16] = [
    "Europe",
    "Germany",
    "France",
    "Spain",
    "Italy",
    "Netherlands",
    "Sweden",
    "UK",
    "Denmark",
    "Norway",
    "Finland",
    "Portugal",
    "Austria",
    "Belgium",
    "Switzerland",
    "Scandinavia",
];

impl Money {
    /// From the game's file name: the No-Intro region tag, as in `(Japan)` or
    /// `(Europe) (En,Fr,De)`. A name with no region tag falls back to a GBA header code's last
    /// letter (J Japan, E America, P and the European countries' letters Europe).
    pub fn of(stem: &str, code: &str) -> Money {
        let mut japan = false;
        let mut europe = false;
        let mut other = false;
        let mut tagged = false;
        for group in stem.split('(').skip(1) {
            let group = group.split(')').next().unwrap_or("");
            let regions: Vec<&str> = group.split(',').map(str::trim).collect();
            let is_region = |r: &str| {
                r == "Japan"
                    || EUROPE.contains(&r)
                    || matches!(
                        r,
                        "USA"
                            | "World"
                            | "Korea"
                            | "Australia"
                            | "Canada"
                            | "Brazil"
                            | "Asia"
                            | "China"
                            | "Taiwan"
                            | "Hong Kong"
                    )
            };
            if regions.is_empty() || !regions.iter().all(|r| is_region(r)) {
                continue;
            }
            tagged = true;
            for r in regions {
                match r {
                    "Japan" => japan = true,
                    r if EUROPE.contains(&r) => europe = true,
                    // Australia shares Europe's releases; they're priced as Europe's.
                    "Australia" => europe = true,
                    _ => other = true,
                }
            }
            // The first region group is the release; later ones are something else.
            break;
        }
        if !tagged {
            return match code.chars().nth(3) {
                Some('J') => Money::Yen,
                Some('P' | 'D' | 'F' | 'S' | 'I' | 'X' | 'Y' | 'H' | 'U') => Money::Euro,
                _ => Money::Dollar,
            };
        }
        match (japan, europe, other) {
            (true, false, false) => Money::Yen,
            (false, true, false) => Money::Euro,
            _ => Money::Dollar,
        }
    }

    pub fn of_cart(cart: &Cart) -> Money {
        Money::of(&cart.stem, &cart.code)
    }

    /// What a shop might have charged, in this money, picked by the game's own seed. The
    /// same place in each list is the same sort of price.
    fn prices(self) -> [&'static str; 8] {
        match self {
            Money::Dollar => PRICES,
            Money::Yen => [
                "2,800", "3,800", "4,200", "4,800", "5,200", "5,800", "6,800", "7,800",
            ],
            Money::Euro => [
                "9,99", "14,99", "19,99", "24,99", "29,99", "34,99", "39,99", "49,99",
            ],
        }
    }

    /// A price as it's written: `$34.99`, `¥4,800`, `29,99 €`.
    fn show(self, price: &str) -> String {
        match self {
            Money::Dollar => format!("${price}"),
            Money::Yen => format!("¥{price}"),
            Money::Euro => format!("{price} €"),
        }
    }

    /// The same, spaced out as the scalloped label prints it.
    fn show_spaced(self, price: &str) -> String {
        match self {
            Money::Dollar => format!("$ {price}"),
            Money::Yen => format!("¥ {price}"),
            Money::Euro => format!("{price} €"),
        }
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
    if amount.is_nan() || amount <= 0.0 {
        return;
    }
    let amount = amount.min(1.0);
    for k in 0..3 {
        let v = px[k] as f32 + (to[k] as f32 - px[k] as f32) * amount;
        px[k] = v.round().clamp(0.0, 255.0) as u8;
    }
}

/// How far each solid pixel is from the cart's outline, up to `max` (`u8::MAX` for pixels
/// that aren't solid): a chessboard distance, in two passes over the face.
fn edge_distance(face: &CartFace, max: u32) -> Vec<u8> {
    let (w, h) = (face.w as usize, face.h as usize);
    let cap = (max + 1).min(254) as u8;
    let mut d: Vec<u8> = face
        .rgba
        .chunks_exact(4)
        .map(|p| if p[3] >= 128 { cap } else { 0 })
        .collect();
    // Off the face counts as outside, so the border's solid pixels sit 1 from the edge.
    let at = |d: &Vec<u8>, x: isize, y: isize| -> u8 {
        if x < 0 || y < 0 || x >= w as isize || y >= h as isize {
            0
        } else {
            d[y as usize * w + x as usize]
        }
    };
    for y in 0..h as isize {
        for x in 0..w as isize {
            let i = y as usize * w + x as usize;
            if d[i] == 0 {
                continue;
            }
            let near = at(&d, x - 1, y)
                .min(at(&d, x - 1, y - 1))
                .min(at(&d, x, y - 1))
                .min(at(&d, x + 1, y - 1));
            d[i] = d[i].min(near + 1);
        }
    }
    for y in (0..h as isize).rev() {
        for x in (0..w as isize).rev() {
            let i = y as usize * w + x as usize;
            if d[i] == 0 {
                continue;
            }
            let near = at(&d, x + 1, y)
                .min(at(&d, x + 1, y + 1))
                .min(at(&d, x, y + 1))
                .min(at(&d, x - 1, y + 1));
            d[i] = d[i].min(near + 1);
        }
    }
    for v in &mut d {
        *v = if *v == 0 { u8::MAX } else { (*v).min(cap) };
    }
    d
}

/// Draw how the cart has been kept into its face.
pub fn apply_look(face: &mut CartFace, look: &Look, seed: u64) {
    apply_look_labelled(face, look, seed, None, Money::Dollar);
}

/// The same, knowing where the label sits on the face (`[x, y, w, h]` in the face's pixels),
/// so heavy wear can fray, tear and lift it.
pub fn apply_look_labelled(
    face: &mut CartFace,
    look: &Look,
    seed: u64,
    label: Option<[u32; 4]>,
    money: Money,
) {
    if look.plain() || face.w == 0 || face.h == 0 {
        return;
    }
    if look.wear > 0 {
        let level = look.wear.min(MAX_WEAR);
        let label = label.filter(|l| l[2] > 8 && l[3] > 8);
        wear(face, level, seed);
        if level >= 4 {
            stains(face, seed ^ 0x57a1, label);
        }
        if let Some(label) = label {
            worn_label(face, level, seed, label);
        }
        insert_end(face, level, seed);
    }
    if look.wrap {
        wrap(face, seed ^ WRAP_SALT, money);
    }
    // Dust settles on whatever is outermost, the film included.
    if look.dust > 0 {
        dust(face, look.dust.min(MAX_LEVEL), seed ^ 0xd057);
    }
}

const DUST: [u8; 3] = [196, 188, 172];
const SPECK: [u8; 3] = [226, 220, 206];
const GRIME: [u8; 3] = [92, 86, 76];

fn dust(face: &mut CartFace, level: u8, seed: u64) {
    let (w, h) = (face.w, face.h);
    let haze = [0.0, 0.11, 0.27, 0.44][level as usize];
    let specks = [0.0, 0.007, 0.028, 0.06][level as usize];
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
            if level >= 2 && n > 1.0 - specks * settle * 0.35 {
                // Grime: a few darker flecks among the pale ones.
                mix(&mut face.rgba[at..at + 4], GRIME, 0.55);
            } else if n < specks * settle {
                let strength = 0.45 + 0.4 * unit(seed ^ 3, x as i64, y as i64);
                mix(&mut face.rgba[at..at + 4], SPECK, strength);
            }
        }
    }
    // Deep dust lies thick along the top edges and in the corners, where a cloth never
    // quite reaches: a soft grey rim that fades inward.
    if level >= 2 {
        let edges = edge_distance(face, 6);
        let rim = [0.0, 0.0, 0.30, 0.55][level as usize];
        let clumps = Field::new(seed ^ 11, w, h, 9.0);
        for y in 0..h {
            let up = (1.0 - y as f32 / h as f32).powf(1.6);
            for x in 0..w {
                let i = (y * w + x) as usize;
                let d = edges[i];
                if d > 6 {
                    continue;
                }
                let k = rim * up * (7 - d) as f32 / 6.0 * (0.4 + 0.9 * clumps.at(x, y));
                mix(&mut face.rgba[i * 4..i * 4 + 4], DUST, k);
            }
        }
    }
    // A few bits of fluff: short, faint strands.
    let strands = [0, 2, 9, 18][level as usize];
    for i in 0..strands {
        let x0 = unit(seed ^ 4, i, 0) * w as f32;
        let y0 = unit(seed ^ 4, i, 1) * h as f32 * 0.7;
        let angle = unit(seed ^ 4, i, 2) * std::f32::consts::TAU;
        let len = 4.0 + unit(seed ^ 4, i, 3) * (5.0 + 3.0 * level as f32);
        let bend = (unit(seed ^ 4, i, 4) - 0.5) * 0.25;
        stroke(face, x0, y0, angle, bend, len, SPECK, 0.55);
    }
}

const RUB: [u8; 3] = [232, 228, 218];

fn wear(face: &mut CartFace, level: u8, seed: u64) {
    let (w, h) = (face.w, face.h);
    let edges = edge_distance(face, 3);
    let rub = [0.0, 0.45, 0.80, 1.0, 1.0][level as usize];
    let reach = [0.0, 0.55, 0.36, 0.20, 0.10][level as usize];
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
            let cut = [1.0, 0.74, 0.60, 0.50, 0.40][level as usize];
            if scuff > cut {
                let amount =
                    ((scuff - cut) / (1.0 - cut)) * [0.0, 0.22, 0.30, 0.34, 0.42][level as usize];
                let grain = 0.7 + 0.3 * unit(seed ^ 6, x as i64, y as i64);
                mix(&mut face.rgba[at..at + 4], RUB, amount * grain);
            }
        }
    }
    let scratches = [0, 4, 14, 30, 52][level as usize];
    for i in 0..scratches {
        let x0 = unit(seed ^ 7, i, 0) * w as f32;
        let y0 = unit(seed ^ 7, i, 1) * h as f32;
        let angle = (unit(seed ^ 7, i, 2) - 0.5) * 1.2 + if i % 3 == 0 { 1.4 } else { 0.0 };
        let len = 7.0 + unit(seed ^ 7, i, 3) * 20.0;
        let bend = (unit(seed ^ 7, i, 4) - 0.5) * 0.06;
        let strength = 0.30 + [0.0, 0.30, 0.38, 0.45, 0.52][level as usize] * unit(seed ^ 7, i, 5);
        stroke(face, x0, y0, angle, bend, len, RUB, strength);
    }
}

const PAPER_BACK: [u8; 3] = [222, 216, 200];
const TORN_FIBRE: [u8; 3] = [240, 236, 226];
const GLUE: [u8; 3] = [150, 142, 120];

/// Worn labels: at level 2 the label's edges fray to bare paper; at level 3 a corner has
/// peeled up and folded back on itself, and a strip has torn away along one edge, leaving
/// the shell and a little glue showing.
fn worn_label(face: &mut CartFace, level: u8, seed: u64, [lx, ly, lw, lh]: [u32; 4]) {
    if level < 2 {
        return;
    }
    let (w, h) = (face.w, face.h);
    let (lx, ly) = (lx.min(w - 1), ly.min(h - 1));
    let (lw, lh) = (lw.min(w - lx), lh.min(h - ly));
    let (x1, y1) = (lx + lw, ly + lh);
    let inside =
        |x: i64, y: i64| x >= lx as i64 && y >= ly as i64 && x < x1 as i64 && y < y1 as i64;
    // The shell beside the label, for what shows where the label has gone.
    let shell = {
        let (sx, sy) = (lx.saturating_sub(3), ly + lh / 2);
        let at = ((sy * w + sx) * 4) as usize;
        [face.rgba[at], face.rgba[at + 1], face.rgba[at + 2]]
    };
    let bare = |face: &mut CartFace, x: u32, y: u32| {
        let at = ((y * w + x) * 4) as usize;
        face.rgba[at..at + 3].copy_from_slice(&shell);
        let glue = 0.35 * unit(seed ^ 31, x as i64 / 2, y as i64 / 2);
        mix(&mut face.rgba[at..at + 4], GLUE, glue);
    };

    if level >= 4 {
        aged_label(face, seed ^ 38, [lx, ly, lw, lh]);
    }

    // Fraying: bare paper fibre showing in spots along the label's edge.
    let fray = [0.0, 0.0, 0.22, 0.36, 0.50][level as usize];
    let patches = Field::new(seed ^ 32, w, h, 16.0);
    let ragged = Field::new(seed ^ 37, w, h, 3.0);
    for y in ly..y1 {
        for x in lx..x1 {
            let d = (x - lx).min(x1 - 1 - x).min(y - ly).min(y1 - 1 - y);
            // Worn back from the edge in soft, uneven nibbles.
            let p = patches.at(x, y);
            let depth = ((p - (1.0 - fray)) / fray * 4.0 + ragged.at(x, y) * 1.5) as u32;
            if p > 1.0 - fray && d <= depth.min(4) {
                let at = ((y * w + x) * 4) as usize;
                let k =
                    (0.85 - d as f32 * 0.12) * (0.7 + 0.3 * unit(seed ^ 33, x as i64, y as i64));
                mix(&mut face.rgba[at..at + 4], TORN_FIBRE, k);
            }
        }
    }
    if level < 3 {
        return;
    }

    // A strip torn from one long edge: a ragged bite with white fibre at its rim.
    let side = hash(seed ^ 34, 0, 0) % 4;
    let along = if side < 2 { lw } else { lh } as f32;
    let start = along * (0.15 + 0.45 * unit(seed ^ 34, 1, 0));
    let more = if level >= 4 { 1.5 } else { 1.0 };
    let len = along * (0.18 + 0.14 * unit(seed ^ 34, 2, 0)) * more;
    let deep = (lw.min(lh) as f32) * (0.06 + 0.05 * unit(seed ^ 34, 3, 0)) * more;
    for y in ly..y1 {
        for x in lx..x1 {
            // Distance into the label from the torn side, and position along it.
            let (into, pos) = match side {
                0 => ((y - ly) as f32, (x - lx) as f32),
                1 => ((y1 - 1 - y) as f32, (x - lx) as f32),
                2 => ((x - lx) as f32, (y - ly) as f32),
                _ => ((x1 - 1 - x) as f32, (y - ly) as f32),
            };
            let t = (pos - start) / len;
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let ragged = 0.55 + 0.45 * smooth(seed ^ 35, pos, 0.0, 3.0);
            let bite = deep * (std::f32::consts::PI * t).sin().powf(0.6) * ragged;
            if into < bite {
                bare(face, x, y);
            } else if into < bite + 1.6 {
                let at = ((y * w + x) * 4) as usize;
                mix(&mut face.rgba[at..at + 4], TORN_FIBRE, 0.85);
            }
        }
    }

    // A dog-eared corner: past a fold line the label has lifted and lies folded back over
    // itself, so the corner shows the shell and the flap shows the label's paper back.
    let first = hash(seed ^ 36, 0, 0) % 4;
    // Past the heaviest level the corner diagonally across has gone too.
    let corners: &[u64] = if level >= 4 {
        &[first, 3 - first]
    } else {
        &[first]
    };
    for (k, &corner) in corners.iter().enumerate() {
        let seed = seed ^ (k as u64 * 0x1d3);
        let (cx, cy, sx, sy) = match corner {
            0 => (lx as f32, ly as f32, 1.0, 1.0),
            1 => (x1 as f32, ly as f32, -1.0, 1.0),
            2 => (lx as f32, y1 as f32, 1.0, -1.0),
            _ => (x1 as f32, y1 as f32, -1.0, -1.0),
        };
        let size = lw.min(lh) as f32 * (0.17 + 0.08 * unit(seed ^ 36, 1, 0));
        // Fold line: points where (u + v) == size, u and v measured in from the corner. A little
        // off 45 degrees, from the seed.
        let skew = 0.75 + 0.5 * unit(seed ^ 36, 2, 0);
        let fold = |u: f32, v: f32| u * skew + v / skew;
        let reach = (size * 2.2) as i64;
        let at_uv = |u: f32, v: f32| ((cx + sx * u) as i64, (cy + sy * v) as i64);
        for v in 0..reach {
            for u in 0..reach {
                let (uf, vf) = (u as f32 + 0.5, v as f32 + 0.5);
                let (x, y) = at_uv(uf, vf);
                if !inside(x, y) {
                    continue;
                }
                let (x, y) = (x as u32, y as u32);
                let f = fold(uf, vf);
                if f < size {
                    bare(face, x, y);
                    continue;
                }
                // The flap: the corner reflected across the fold, as far past it as the corner
                // was before it.
                let past = f - size;
                let n = (skew * skew + 1.0 / (skew * skew)).sqrt();
                let back = past / n;
                if back * n > size {
                    // A soft shadow the flap casts beside it.
                    let shade = (1.0 - (past - size) / 3.0).clamp(0.0, 1.0) * 0.35;
                    if shade > 0.0 {
                        let at = ((y * w + x) * 4) as usize;
                        mix(&mut face.rgba[at..at + 4], [0, 0, 0], shade);
                    }
                    continue;
                }
                let at = ((y * w + x) * 4) as usize;
                // Paper back, brighter near the fold where it curls toward the light.
                let curl = 1.0 - back * n / size;
                face.rgba[at..at + 3].copy_from_slice(&PAPER_BACK);
                mix(&mut face.rgba[at..at + 4], [255, 255, 255], 0.25 * curl);
                mix(
                    &mut face.rgba[at..at + 4],
                    [90, 84, 70],
                    0.18 * (1.0 - curl),
                );
                if (past - 0.0).abs() < 1.0 {
                    mix(&mut face.rgba[at..at + 4], [255, 255, 255], 0.5);
                }
            }
        }
    }
}

const YELLOWED: [u8; 3] = [214, 196, 150];
const SMUDGE: [u8; 3] = [70, 62, 50];

/// A label that has seen everything: its print sun-faded and gone yellow, worst in soft
/// patches, with grey thumb smudges where it has been held.
fn aged_label(face: &mut CartFace, seed: u64, [lx, ly, lw, lh]: [u32; 4]) {
    let (w, h) = (face.w, face.h);
    let fade = Field::new(seed, w, h, 26.0);
    let grain = Field::new(seed ^ 1, w, h, 4.0);
    for y in ly..ly + lh {
        for x in lx..lx + lw {
            let at = ((y * w + x) * 4) as usize;
            if face.rgba[at + 3] < 128 {
                continue;
            }
            let k = 0.16 + 0.30 * fade.at(x, y) + 0.06 * grain.at(x, y);
            mix(&mut face.rgba[at..at + 4], YELLOWED, k);
        }
    }
    // Thumb smudges: soft ovals near the label's sides, where a thumb rests to pull it.
    let size = lw.min(lh) as f32;
    for i in 0..2 {
        let side = if i == 0 { 0.12 } else { 0.88 };
        let cx = lx as f32 + lw as f32 * (side + (unit(seed ^ 2, i, 0) - 0.5) * 0.12);
        let cy = ly as f32 + lh as f32 * (0.35 + 0.45 * unit(seed ^ 2, i, 1));
        let (rx, ry) = (size * 0.14, size * 0.19);
        let strength = 0.16 + 0.10 * unit(seed ^ 2, i, 2);
        for y in ly..ly + lh {
            for x in lx..lx + lw {
                let (dx, dy) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
                let r = (dx * dx + dy * dy).sqrt() + 0.35 * (grain.at(x, y) - 0.5);
                if r < 1.0 {
                    let at = ((y * w + x) * 4) as usize;
                    mix(
                        &mut face.rgba[at..at + 4],
                        SMUDGE,
                        strength * (1.0 - r).powf(0.7),
                    );
                }
            }
        }
    }
}

const STAIN: [u8; 3] = [118, 88, 46];

/// Old spills dried on the shell: a few irregular tea-brown blots, darkest at the ring
/// where each dried, kept off the label.
fn stains(face: &mut CartFace, seed: u64, label: Option<[u32; 4]>) {
    let (w, h) = (face.w, face.h);
    let on_label = |x: u32, y: u32| {
        label.is_some_and(|[lx, ly, lw, lh]| x >= lx && y >= ly && x < lx + lw && y < ly + lh)
    };
    let edge = Field::new(seed, w, h, 5.0);
    let size = w.min(h) as f32;
    let count = 2 + (hash(seed, 0, 0) % 3) as i64;
    for i in 0..count {
        // Somewhere on the shell itself; a few tries to land off the label.
        let mut centre = None;
        for t in 0..12 {
            let x = (unit(seed ^ 1, i, t) * w as f32) as u32;
            let y = (unit(seed ^ 2, i, t) * h as f32) as u32;
            let at = ((y.min(h - 1) * w + x.min(w - 1)) * 4) as usize;
            if face.rgba[at + 3] >= 128 && !on_label(x, y) {
                centre = Some((x as f32, y as f32));
                break;
            }
        }
        let Some((cx, cy)) = centre else { continue };
        let r = size * (0.05 + 0.07 * unit(seed ^ 3, i, 0));
        let squash = 0.7 + 0.5 * unit(seed ^ 3, i, 1);
        let strength = 0.16 + 0.12 * unit(seed ^ 3, i, 2);
        let reach = (r * 1.6) as i64;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let (x, y) = (cx as i64 + dx, cy as i64 + dy);
                if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
                    continue;
                }
                let (x, y) = (x as u32, y as u32);
                let at = ((y * w + x) * 4) as usize;
                if face.rgba[at + 3] < 128 || on_label(x, y) {
                    continue;
                }
                let d = ((dx as f32 / squash).powi(2) + (dy as f32 * squash).powi(2)).sqrt() / r
                    + 0.45 * (edge.at(x, y) - 0.5);
                if d >= 1.0 {
                    continue;
                }
                // Faint inside, a darker tide line at the rim.
                let ring = (1.0 - (1.0 - d) / 0.12).clamp(0.0, 1.0);
                let k = strength * (0.55 + 0.45 * d) + 0.22 * ring;
                mix(&mut face.rgba[at..at + 4], STAIN, k);
            }
        }
    }
}

/// The end that goes into the slot: rubbed pale and scored by countless insertions, worst
/// right at the edge.
fn insert_end(face: &mut CartFace, level: u8, seed: u64) {
    if level < 2 {
        return;
    }
    let (w, h) = (face.w, face.h);
    let bottom = (0..h)
        .rev()
        .find(|&y| (0..w).any(|x| face.rgba[((y * w + x) * 4 + 3) as usize] >= 128))
        .unwrap_or(h - 1);
    let band = h as f32 * [0.0, 0.0, 0.10, 0.16, 0.21][level as usize];
    let strength = [0.0, 0.0, 0.40, 0.70, 0.85][level as usize];
    let haze = Field::new(seed ^ 41, w, h, 10.0);
    let top = (bottom as f32 - band).max(0.0) as u32;
    for y in top..=bottom {
        let near = (1.0 - (bottom - y) as f32 / band.max(1.0)).clamp(0.0, 1.0);
        for x in 0..w {
            let at = ((y * w + x) * 4) as usize;
            if face.rgba[at + 3] < 128 {
                continue;
            }
            let grain = 0.6 + 0.4 * unit(seed ^ 42, x as i64, y as i64);
            let k = strength * near.powf(1.4) * (0.45 + 0.75 * haze.at(x, y)) * grain;
            mix(&mut face.rgba[at..at + 4], RUB, k);
        }
    }
    // Scoring: short scratches running up from the edge, as the slot's springs drag on it.
    let marks = [0, 0, 14, 34, 56][level as usize];
    for i in 0..marks {
        let x0 = unit(seed ^ 43, i, 0) * w as f32;
        let y0 = bottom as f32 - unit(seed ^ 43, i, 1) * band * 0.7;
        let angle = -std::f32::consts::FRAC_PI_2 + (unit(seed ^ 43, i, 2) - 0.5) * 0.35;
        let len = 3.0 + unit(seed ^ 43, i, 3) * band * 0.9;
        let k = 0.35 + 0.35 * unit(seed ^ 43, i, 4);
        stroke(face, x0, y0, angle, 0.0, len, RUB, k);
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
fn wrap(face: &mut CartFace, seed: u64, money: Money) {
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
    price_sticker(face, seed, money);
}

/// The shrink wrap alone, as a layer to draw over the bare cart while it is torn off: what
/// `wrap` adds to a face. Only the cart's outline matters, so any face of the cart will do.
///
/// The film is found by wrapping the outline twice, once black and once white: where the
/// two differ the cart shows through, and what they share is the film.
pub fn film_of(face: &CartFace, seed: u64, money: Money) -> CartFace {
    let painted = |v: u8| {
        let mut f = CartFace {
            w: face.w,
            h: face.h,
            rgba: face.rgba.clone(),
        };
        for px in f.rgba.chunks_exact_mut(4) {
            px[..3].fill(v);
        }
        wrap(&mut f, seed ^ WRAP_SALT, money);
        f
    };
    let (dark, light) = (painted(0), painted(255));
    let mut film = dark;
    for (px, lit) in film
        .rgba
        .chunks_exact_mut(4)
        .zip(light.rgba.chunks_exact(4))
    {
        let all = px[3] as u32;
        if all == 0 {
            continue;
        }
        // Premultiplied, the white cart adds 255 x (the cart's share) to the black one.
        let through = (lit[1] as u32 - (px[1] as u32).min(lit[1] as u32)) * all / 255;
        let alpha = all.saturating_sub(through);
        if alpha == 0 {
            px.fill(0);
            continue;
        }
        for c in &mut px[..3] {
            *c = (*c as u32 * all / alpha).min(255) as u8;
        }
        px[3] = alpha as u8;
    }
    film
}

/// How far the film stands off the cart, in pixels.
const FILM_SLACK: u32 = 5;
/// What a game's seed is mixed with for its film, so `film_of` matches the wrapped face.
const WRAP_SALT: u64 = 0x5ea1;

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
fn price_sticker(face: &mut CartFace, seed: u64, money: Money) {
    let pick = |n: i64, of: usize| (hash(seed ^ 0x9a1c, n, 0) % of as u64) as usize;
    let prices = money.prices();
    let price = prices[pick(1, prices.len())];
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
            let old = money.show(prices[was]);
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
            let new = money.show(prices[now]);
            t.print(0, z(25.0), w, z(18.0), &new, zf(15.0), STAMP_INK, true);
            t
        }
        // A price gun's label with a red line top and bottom.
        1 => {
            let (w, h) = (z(66.0), z(30.0));
            let mut t = Tag::cut(w, h, TAG_WHITE, seed, notched(w as f32, h as f32));
            t.rule(z(3.0), w - z(3.0), z(5.0), TAG_RED, 0.85);
            t.rule(z(3.0), w - z(3.0), h - z(5.0), TAG_RED, 0.85);
            let words = money.show(price);
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
            let words = money.show_spaced(price);
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
            let words = money.show(price);
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
/// or WonderSwan or Pokémon mini cart, and low on the back of a Lynx card. Other carts have no back to
/// show yet.
pub fn sticker_at(cart: &Cart, back: &CartFace) -> Option<(u32, u32)> {
    use slot_store::Platform;
    let (sw, sh) = (STICKER_W, STICKER_H);
    let spot = match cart.platform.hardware() {
        Platform::Lynx => (
            back.w.saturating_sub(sw + 16),
            back.h.saturating_sub(sh + 12),
        ),
        Platform::Gba | Platform::Gbav => (224, 56),
        Platform::Ws => (250, 104),
        // Low on the Pokémon mini board, either side of its mark.
        Platform::Pm => (38, 126),
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
    let spot = match cart.platform.hardware() {
        Platform::Lynx => (16, back.h.saturating_sub(sh + 12)),
        Platform::Gba | Platform::Gbav => (224, 27),
        Platform::Ws => (66, 114),
        Platform::Pm => (186, 126),
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
    // slotPlus: the writing keeps clear of the little icon in the top right corner.
    let room = sw - ICON_ROOM;
    if let Some(hand) = crate::hand::hand_font() {
        let lines = [text.to_string()];
        let mut px = 17.0;
        while px > 9.0 && hand.line_width(text, px) > (room - 6) as f32 {
            px -= 1.0;
        }
        for (i, a) in hand.coverage(room, sh, &lines, px).into_iter().enumerate() {
            let at = ((i as u32 / room * sw + i as u32 % room) * 4) as usize;
            if a == 0 || paper[at + 3] == 0 {
                continue;
            }
            mix(&mut paper[at..at + 4], ink, a as f32 / 255.0);
        }
    }
    let icon = match which {
        0 => Icon::Stopwatch,
        _ => Icon::Trophy,
    };
    draw_icon(&mut paper, sw, icon, ink);
    paste(back, &paper, sw, sh, (x0, y0), tilt);
}

/// slotPlus: how much of a sticker's width, at the right, is kept for its icon.
const ICON_ROOM: u32 = 12;

/// slotPlus: the little printed icon in a sticker's top right corner: a stopwatch on the
/// play-time sticker, a trophy on the achievements one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Stopwatch,
    Trophy,
}

/// Whether a point, in the icon's own 10 x 10 box, is inked.
fn icon_inked(icon: Icon, x: f32, y: f32) -> bool {
    let in_box = |x0: f32, y0: f32, x1: f32, y1: f32| x >= x0 && x <= x1 && y >= y0 && y <= y1;
    match icon {
        Icon::Stopwatch => {
            // A ring for the face, a crown and button on top, and one hand.
            let (cx, cy) = (5.0, 5.9);
            let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            let ring = (3.0..=4.0).contains(&d);
            let crown = in_box(4.1, 0.3, 5.9, 1.9);
            let stem = in_box(4.6, 1.4, 5.4, 2.2);
            let button = in_box(7.6, 1.6, 8.6, 2.6);
            let hand = in_box(4.6, 3.4, 5.4, 6.2) || in_box(4.6, 5.4, 6.8, 6.2);
            ring || crown || stem || button || hand
        }
        Icon::Trophy => {
            // A cup with a handle each side, a stem and a base.
            let cup = in_box(2.6, 0.6, 7.4, 2.0)
                || (y >= 2.0 && y <= 5.4 && (x - 5.0).abs() <= 2.4 - (y - 2.0) * 0.38);
            let handle = |side: f32| {
                let (hx, hy) = (5.0 + side * 2.9, 2.6);
                let d = ((x - hx).powi(2) + (y - hy).powi(2)).sqrt();
                (0.9..=1.7).contains(&d) && (x - 5.0) * side > 2.4
            };
            let stem = in_box(4.4, 5.2, 5.6, 7.4);
            let base = in_box(2.8, 7.6, 7.2, 9.2);
            cup || handle(-1.0) || handle(1.0) || stem || base
        }
    }
}

/// Draw `icon` into a sticker's paper, in its top right corner, in the sticker's ink.
fn draw_icon(paper: &mut [u8], sw: u32, icon: Icon, ink: [u8; 3]) {
    const SIZE: u32 = 10;
    const SAMPLES: u32 = 4;
    let (x0, y0) = (sw - SIZE - 3, 3);
    for py in 0..SIZE {
        for px in 0..SIZE {
            let mut hits = 0;
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let x = px as f32 + (sx as f32 + 0.5) / SAMPLES as f32;
                    let y = py as f32 + (sy as f32 + 0.5) / SAMPLES as f32;
                    hits += icon_inked(icon, x, y) as u32;
                }
            }
            let at = (((y0 + py) * sw + x0 + px) * 4) as usize;
            if hits == 0 || paper[at + 3] == 0 {
                continue;
            }
            let cover = hits as f32 / (SAMPLES * SAMPLES) as f32;
            mix(&mut paper[at..at + 4], ink, cover * 0.9);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_play_sticker_has_a_stopwatch_and_a_gold_one_a_trophy_in_the_corner() {
        let (sw, sh) = (STICKER_W, STICKER_H);
        let paper = |icon| {
            let mut p = vec![255u8; (sw * sh * 4) as usize];
            draw_icon(&mut p, sw, icon, [0, 0, 0]);
            p
        };
        let inked = |p: &[u8], x0: u32, x1: u32| {
            (0..sh)
                .flat_map(|y| (x0..x1).map(move |x| (x, y)))
                .filter(|&(x, y)| p[((y * sw + x) * 4) as usize] < 200)
                .count()
        };
        let (watch, cup) = (paper(Icon::Stopwatch), paper(Icon::Trophy));
        for p in [&watch, &cup] {
            assert!(
                inked(p, sw - ICON_ROOM - 1, sw) > 15,
                "an icon in the top right"
            );
            assert_eq!(
                inked(p, 0, sw - ICON_ROOM - 1),
                0,
                "and nothing under the writing"
            );
        }
        assert_ne!(watch, cup, "two different icons");
    }

    #[test]
    fn dust_and_wear_build_in_steps() {
        assert_eq!(dust_level(0), 0);
        assert_eq!(dust_level(14 * 86_400), 1);
        assert_eq!(dust_level(60 * 86_400), 2);
        assert_eq!(dust_level(400 * 86_400), 3);
        assert_eq!(wear_level(2 * 3_600), 0);
        assert_eq!(wear_level(3 * 3_600), 1);
        assert_eq!(wear_level(9 * 3_600), 2);
        assert_eq!(wear_level(25 * 3_600), 3);
        assert_eq!(wear_level(99 * 3_600), 3);
        assert_eq!(wear_level(1_000 * 3_600), 4);
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

    #[test]
    fn the_film_alone_is_clear_over_the_cart_and_solid_at_its_sticker() {
        let mut face = CartFace {
            w: 120,
            h: 80,
            rgba: vec![0; 120 * 80 * 4],
        };
        for y in 10..70 {
            for x in 10..110 {
                face.rgba[(y * 120 + x) * 4..][..4].copy_from_slice(&[90, 40, 40, 255]);
            }
        }
        let film = film_of(&face, 7, Money::Dollar);
        let alpha = |x: usize, y: usize| film.rgba[(y * 120 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0, "nothing far outside the cart");
        let thin = film
            .rgba
            .chunks_exact(4)
            .filter(|p| (1..120).contains(&p[3]))
            .count();
        let solid = film.rgba.chunks_exact(4).filter(|p| p[3] == 255).count();
        assert!(thin > 2000, "most of the film is see-through: {thin}");
        assert!(solid > 100, "the price sticker is not: {solid}");
    }

    #[test]
    fn heavy_wear_lifts_a_label_corner_and_scuffs_the_insert_end() {
        let blank = || {
            let mut face = CartFace {
                w: 120,
                h: 140,
                rgba: vec![0; 120 * 140 * 4],
            };
            for y in 0..140 {
                for x in 0..120 {
                    let c = if (20..100).contains(&x) && (20..100).contains(&y) {
                        [200, 40, 40, 255]
                    } else {
                        [120, 120, 124, 255]
                    };
                    face.rgba[(y * 120 + x) * 4..][..4].copy_from_slice(&c);
                }
            }
            face
        };
        let label = Some([20, 20, 80, 80]);
        let worn = |level| {
            let mut f = blank();
            let look = Look {
                wrap: false,
                dust: 0,
                wear: level,
            };
            apply_look_labelled(&mut f, &look, 99, label, Money::Dollar);
            f
        };
        let red_left = |f: &CartFace| {
            f.rgba
                .chunks_exact(4)
                .filter(|p| p[0] > 150 && p[1] < 90)
                .count()
        };
        let (fresh, light, heavy) = (red_left(&blank()), red_left(&worn(1)), red_left(&worn(3)));
        assert!(
            light > fresh * 9 / 10,
            "light wear leaves the label: {light}/{fresh}"
        );
        assert!(
            heavy < fresh * 9 / 10,
            "heavy wear tears and lifts it: {heavy}/{fresh}"
        );
        let pale = |f: &CartFace, y: usize| {
            (0..120)
                .map(|x| f.rgba[(y * 120 + x) * 4] as u32)
                .sum::<u32>()
                / 120
        };
        assert!(
            pale(&worn(3), 138) > pale(&blank(), 138) + 20,
            "the insert end rubs pale"
        );
        let worst = worn(MAX_WEAR);
        assert!(
            red_left(&worst) < heavy,
            "the last level takes more of the label"
        );
        let stained = |f: &CartFace| {
            f.rgba
                .chunks_exact(4)
                .enumerate()
                .filter(|(i, p)| {
                    let (x, y) = (i % 120, i / 120);
                    let off_label = !((20..100).contains(&x) && (20..100).contains(&y));
                    off_label && p[0] > p[2] + 24
                })
                .count()
        };
        assert_eq!(stained(&worn(3)), 0, "no stains before the last level");
        assert!(stained(&worst) > 20, "the shell is stained off the label");
    }

    #[test]
    fn the_price_is_in_the_money_of_where_the_game_was_sold() {
        let of = |stem: &str| Money::of(stem, "");
        assert_eq!(of("Mother 3 (Japan)"), Money::Yen);
        assert_eq!(
            of("Mother 3 (Japan) (En) (v1.3)"),
            Money::Yen,
            "a translation of a Japanese cart"
        );
        assert_eq!(of("Golden Sun (Europe) (En,Fr,De)"), Money::Euro);
        assert_eq!(of("Kuru Kuru Kururin (Europe, Australia)"), Money::Euro);
        assert_eq!(of("Tetris (World) (Rev 1)"), Money::Dollar);
        assert_eq!(of("Pokemon Pinball (USA, Australia)"), Money::Dollar);
        assert_eq!(
            of("Super Mario Land (Japan, USA)"),
            Money::Dollar,
            "a conflict is dollars"
        );
        assert_eq!(of("Advance Wars (USA) (Beta)"), Money::Dollar);
        assert_eq!(of("My Homebrew"), Money::Dollar, "no tag, no code: dollars");
        assert_eq!(
            Money::of("Untagged", "AXVJ"),
            Money::Yen,
            "a GBA code's region letter"
        );
        assert_eq!(Money::of("Untagged", "BPEP"), Money::Euro);
        assert_eq!(Money::of("Untagged", "BPEE"), Money::Dollar);
        assert_eq!(Money::Yen.show("4,800"), "¥4,800");
        assert_eq!(Money::Euro.show("29,99"), "29,99 €");
    }

    #[test]
    fn a_yen_sticker_is_not_a_dollar_one() {
        let face = || {
            let mut f = CartFace {
                w: 160,
                h: 120,
                rgba: vec![0; 160 * 120 * 4],
            };
            for px in f.rgba.chunks_exact_mut(4) {
                px.copy_from_slice(&[90, 90, 96, 255]);
            }
            f
        };
        let (mut a, mut b) = (face(), face());
        price_sticker(&mut a, 5, Money::Dollar);
        price_sticker(&mut b, 5, Money::Yen);
        assert_ne!(a.rgba, b.rgba);
    }
}
