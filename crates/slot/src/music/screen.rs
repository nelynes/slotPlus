//! slotPlus music cart: drawing the player's screen, a Game Boy Advance's 240x160, in hard
//! pixels like a real cart's. The staircase, its doors and notes, the cart that climbs it,
//! and the band across the top, all drawn here from code: no picture files.

use std::collections::HashMap;
use std::sync::OnceLock;

use slot_retro::{GBA_H, GBA_W};

pub const W: usize = GBA_W as usize;
pub const H: usize = GBA_H as usize;

/// The player's colours. Two themes: the classic cyan and pink on black, and a
/// Monochrome-ish one in greys with red for what matters, like a Game Boy Pocket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub back: [u8; 3],
    /// The stairs, frames and small print.
    pub line: [u8; 3],
    pub dim: [u8; 3],
    /// A step's name, and the name on the picked step.
    pub text: [u8; 3],
    pub text_on: [u8; 3],
    pub note: [u8; 3],
    /// The picked step's band, and its lighter ripple.
    pub pick: [u8; 3],
    pub pick_hi: [u8; 3],
    /// The cart that climbs: his body, and the label with his face.
    pub body: [u8; 3],
    pub face: [u8; 3],
    /// The big play and pause signs.
    pub big: [u8; 3],
    pub pipe: [u8; 3],
    pub door: [u8; 3],
}

pub const CLASSIC: Palette = Palette {
    back: [0, 0, 0],
    line: [0x58, 0xce, 0xd6],
    dim: [0x2e, 0x80, 0x8c],
    text: [0x58, 0xce, 0xd6],
    text_on: [0xf0, 0xf6, 0xfa],
    note: [0xf0, 0x6e, 0xc8],
    pick: [0x18, 0x34, 0xbe],
    pick_hi: [0x3c, 0x6c, 0xf0],
    body: [0xf0, 0xf6, 0xfa],
    face: [0x2e, 0x80, 0x8c],
    big: [0x14, 0x30, 0xe8],
    pipe: [0x4c, 0xc8, 0x64],
    door: [0x58, 0xce, 0xd6],
};

pub const MONOCHROME: Palette = Palette {
    back: [0xe4, 0xe4, 0xe0],
    line: [0x26, 0x26, 0x26],
    dim: [0x9a, 0x9a, 0x98],
    text: [0x9a, 0x9a, 0x98],
    text_on: [0x16, 0x16, 0x16],
    note: [0xd8, 0x18, 0x18],
    pick: [0xcc, 0xcc, 0xc8],
    pick_hi: [0xf4, 0xf4, 0xf2],
    body: [0x1c, 0x1c, 0x1c],
    face: [0xd8, 0x18, 0x18],
    big: [0xd8, 0x18, 0x18],
    pipe: [0x5a, 0x5a, 0x5a],
    door: [0x8a, 0x8a, 0x88],
};

/// The titles' type: DotGothic16, a pixel font with Japanese too, at the size it's drawn for.
const TITLE_TTF: &[u8] = include_bytes!("../../assets/DotGothic16-Regular.ttf");
pub const TITLE_PX: u32 = 16;
/// Small print (times, the play order): Tiny5.
const SMALL_TTF: &[u8] = include_bytes!("../../assets/Tiny5-Regular.ttf");
pub const SMALL_PX: u32 = 10;

fn font(which: Type) -> Option<&'static fontdue::Font> {
    static TITLE: OnceLock<Option<fontdue::Font>> = OnceLock::new();
    static SMALL: OnceLock<Option<fontdue::Font>> = OnceLock::new();
    let (cell, bytes) = match which {
        Type::Title => (&TITLE, TITLE_TTF),
        Type::Small => (&SMALL, SMALL_TTF),
    };
    cell.get_or_init(|| fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).ok())
        .as_ref()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    Title,
    Small,
}

impl Type {
    fn px(self) -> u32 {
        match self {
            Type::Title => TITLE_PX,
            Type::Small => SMALL_PX,
        }
    }
}

/// How the cart that climbs the stairs is standing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pose {
    Stand,
    /// Mid-hop: legs tucked, arms up.
    Hop,
    DanceLeft,
    DanceRight,
}

type Glyph = (fontdue::Metrics, Vec<u8>);

/// The screen as `xrgb8888`, with a cache of the glyphs it has drawn.
pub struct Screen {
    pub px: Vec<u8>,
    pub pal: Palette,
    glyphs: HashMap<(char, Type), Glyph>,
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            px: vec![0; W * H * 4],
            pal: CLASSIC,
            glyphs: HashMap::new(),
        }
    }
}

impl Screen {
    pub fn put(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if x < 0 || y < 0 || x as usize >= W || y as usize >= H {
            return;
        }
        let at = (y as usize * W + x as usize) * 4;
        self.px[at..at + 4].copy_from_slice(&[c[2], c[1], c[0], 0xff]);
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3]) {
        for yy in y.max(0)..(y + h).min(H as i32) {
            for xx in x.max(0)..(x + w).min(W as i32) {
                self.put(xx, yy, c);
            }
        }
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: [u8; 3]) {
        self.rect(x0, y, x1 - x0 + 1, 1, c);
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: [u8; 3]) {
        self.rect(x, y0, 1, y1 - y0 + 1, c);
    }

    pub fn fill(&mut self, c: [u8; 3]) {
        self.rect(0, 0, W as i32, H as i32, c);
    }

    fn glyph(&mut self, ch: char, t: Type) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&(ch, t)) {
            return Some(g.clone());
        }
        let g = font(t)?.rasterize(ch, t.px() as f32);
        self.glyphs.insert((ch, t), g.clone());
        Some(g)
    }

    /// How wide `text` is.
    pub fn width(&mut self, text: &str, t: Type) -> i32 {
        let mut w = 0.0;
        for ch in text.chars() {
            if let Some((m, _)) = self.glyph(ch, t) {
                w += m.advance_width;
            }
        }
        w.round() as i32
    }

    /// One line of text, its top at `top`, starting `skip` pixels in (for scrolling it) and
    /// cut off at `left` and `right`. Pixels are on or off, as on a cart's screen.
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        text: &str,
        t: Type,
        x: i32,
        top: i32,
        (left, right): (i32, i32),
        skip: i32,
        c: [u8; 3],
    ) {
        let Some(vm) = font(t).and_then(|f| f.horizontal_line_metrics(t.px() as f32)) else {
            return;
        };
        let base = top as f32 + vm.ascent;
        let mut pen = (x - skip) as f32;
        for ch in text.chars() {
            if pen > right as f32 {
                break;
            }
            let Some((m, cov)) = self.glyph(ch, t) else {
                continue;
            };
            let gx = (pen + m.xmin as f32).round() as i32;
            let gy = (base - (m.height as f32 + m.ymin as f32)).round() as i32;
            for r in 0..m.height {
                for col in 0..m.width {
                    let x = gx + col as i32;
                    if x < left || x > right || cov[r * m.width + col] < 110 {
                        continue;
                    }
                    self.put(x, gy + r as i32, c);
                }
            }
            pen += m.advance_width;
        }
    }

    /// A closed door standing on `floor`, 9 wide and 13 tall.
    pub fn door(&mut self, x: i32, floor: i32, c: [u8; 3]) {
        self.rect(x, floor - 11, 9, 11, c);
        self.rect(x + 2, floor - 13, 5, 2, c);
        self.put(x + 1, floor - 12, c);
        self.put(x + 7, floor - 12, c);
        self.put(x + 6, floor - 6, self.pal.back);
    }

    /// The same door open: its frame, with `shut` of its 9 columns closed from the left.
    pub fn doorway(&mut self, x: i32, floor: i32, shut: i32, c: [u8; 3]) {
        self.vline(x, floor - 11, floor - 1, c);
        self.vline(x + 8, floor - 11, floor - 1, c);
        self.hline(x + 2, x + 6, floor - 13, c);
        self.put(x + 1, floor - 12, c);
        self.put(x + 7, floor - 12, c);
        if shut > 0 {
            self.rect(x, floor - 11, shut.min(9), 11, c);
            self.rect(x + 2, floor - 13, (shut - 2).clamp(0, 5), 2, c);
        }
    }

    /// The EXIT door: an archway with a list on it and its word, flying on a propeller
    /// whose blades turn with `frame`.
    pub fn exit_door(&mut self, x: i32, floor: i32, frame: u64) {
        let c = self.pal.door;
        self.vline(x, floor - 16, floor - 1, c);
        self.vline(x + 12, floor - 16, floor - 1, c);
        self.hline(x, x + 12, floor - 1, c);
        self.hline(x + 3, x + 9, floor - 20, c);
        for (dx, dy) in [(1, 17), (2, 18), (2, 19), (11, 17), (10, 18), (10, 19)] {
            self.put(x + dx, floor - dy, c);
        }
        for (k, len) in [2, 4, 6].into_iter().enumerate() {
            self.hline(x + 3, x + 3 + len, floor - 12 + 3 * k as i32, c);
        }
        // The propeller: a shaft, and blades that turn.
        self.vline(x + 6, floor - 23, floor - 21, c);
        let reach = [8, 5, 1, 5][((frame / 3) % 4) as usize];
        self.hline(x + 6 - reach, x + 6 + reach, floor - 24, c);
        self.text("EXIT", Type::Small, x - 2, floor + 1, (0, W as i32), 0, c);
    }

    /// A warp pipe standing on `floor`, 13 across: back to the top of the music.
    pub fn pipe(&mut self, x: i32, floor: i32) {
        let (c, b) = (self.pal.pipe, self.pal.back);
        self.rect(x + 1, floor - 8, 11, 8, c);
        self.rect(x, floor - 12, 13, 4, c);
        self.hline(x + 1, x + 11, floor - 9, b);
        self.vline(x + 3, floor - 7, floor - 1, b);
        self.vline(x + 2, floor - 11, floor - 10, b);
    }

    /// The big play sign, `h` tall.
    pub fn big_play(&mut self, x: i32, y: i32, h: i32) {
        let c = self.pal.big;
        for r in 0..h {
            let half = r.min(h - 1 - r);
            self.hline(x, x + half, y + r, c);
        }
    }

    pub fn big_pause(&mut self, x: i32, y: i32, h: i32) {
        let c = self.pal.big;
        self.rect(x, y, 7, h, c);
        self.rect(x + 12, y, 7, h, c);
    }

    /// The band on a step: plain for the picked step, rippling out from its left end while
    /// its song plays (`phase` turns), and fading away to the right.
    pub fn band(&mut self, x0: i32, top: i32, h: i32, phase: Option<f32>, picked: bool) {
        let (lo, hi) = (self.pal.pick, self.pal.pick_hi);
        let w = W as i32 - x0;
        for dx in 0..w {
            let fade = 1.0 - dx as f32 / w as f32;
            let wave = match phase {
                Some(p) => (0.5 + 0.5 * ((dx as f32) * 0.55 - p).sin()) * fade,
                None => match dx % 12 == 0 {
                    true => 0.6 * fade,
                    false => 0.0,
                },
            };
            // Three hard steps of colour, as a cart's screen would have.
            let (c, on) = match (picked, wave) {
                (true, w) if w > 0.55 => (hi, true),
                (true, _) => (lo, true),
                (false, w) if w > 0.62 => (lo, true),
                (false, _) => (lo, false),
            };
            if on && (picked || fade > 0.25) {
                self.vline(x0 + dx, top, top + h - 1, c);
            }
        }
    }
    /// A note, its foot at `foot`.
    pub fn note(&mut self, x: i32, foot: i32, c: [u8; 3]) {
        self.rect(x, foot - 4, 3, 3, c);
        self.vline(x + 2, foot - 11, foot - 3, c);
        self.put(x + 3, foot - 11, c);
        self.put(x + 4, foot - 10, c);
        self.vline(x + 5, foot - 9, foot - 8, c);
    }

    /// A playlist: two notes.
    pub fn notes(&mut self, x: i32, foot: i32, c: [u8; 3]) {
        self.note(x, foot, c);
        self.note(x + 5, foot - 2, c);
    }

    /// slotPlus's guide: a little cart with arms and legs and a face on its label, its feet
    /// on `floor`. `shown` rows of it, from the top, are drawn (for going through a door).
    pub fn guide(&mut self, x: i32, floor: i32, pose: Pose, shown: i32) {
        let top = floor - 16;
        let (body, face) = (self.pal.body, self.pal.face);
        let px = |s: &mut Screen, gx: i32, gy: i32, c: [u8; 3]| {
            if gy - top < shown {
                s.put(gx, gy, c);
            }
        };
        let y = floor;
        // The body: a cart, its top corners cut.
        for yy in y - 15..=y - 5 {
            for xx in x + 1..=x + 9 {
                let corner = yy == y - 15 && (xx == x + 1 || xx == x + 9);
                if !corner {
                    px(self, xx, yy, body);
                }
            }
        }
        // Its label, with a face.
        for yy in y - 13..=y - 8 {
            for xx in x + 3..=x + 7 {
                px(self, xx, yy, face);
            }
        }
        px(self, x + 4, y - 11, body);
        px(self, x + 6, y - 11, body);
        for xx in x + 4..=x + 6 {
            px(self, xx, y - 9, body);
        }
        // The contacts' edge.
        for xx in x + 2..=x + 8 {
            px(self, xx, y - 4, body);
        }
        let limbs: &[(i32, i32)] = match pose {
            Pose::Stand => &[
                (3, -3),
                (3, -2),
                (3, -1),
                (2, -1),
                (7, -3),
                (7, -2),
                (7, -1),
                (8, -1),
                (0, -10),
                (0, -9),
                (0, -8),
                (10, -10),
                (10, -9),
                (10, -8),
            ],
            Pose::Hop => &[
                (2, -3),
                (1, -2),
                (8, -3),
                (9, -2),
                (0, -11),
                (-1, -12),
                (-1, -13),
                (10, -11),
                (11, -12),
                (11, -13),
            ],
            Pose::DanceLeft => &[
                (3, -3),
                (2, -2),
                (1, -1),
                (7, -3),
                (7, -2),
                (7, -1),
                (0, -11),
                (-1, -12),
                (-2, -13),
                (10, -9),
                (11, -8),
                (12, -8),
            ],
            Pose::DanceRight => &[
                (3, -3),
                (3, -2),
                (3, -1),
                (7, -3),
                (8, -2),
                (9, -1),
                (0, -9),
                (-1, -8),
                (-2, -8),
                (10, -11),
                (11, -12),
                (12, -13),
            ],
        };
        for (dx, dy) in limbs {
            px(self, x + dx, y + dy, body);
        }
    }

    /// The B button, round, in the corner.
    pub fn b_button(&mut self, x: i32, y: i32) {
        for yy in 0..11 {
            for xx in 0..11 {
                let (dx, dy) = (xx as f32 - 5.0, yy as f32 - 5.0);
                if dx * dx + dy * dy <= 30.0 {
                    self.put(x + xx, y + yy, self.pal.line);
                }
            }
        }
        let back = self.pal.back;
        self.text("B", Type::Small, x + 3, y, (x, x + 10), 0, back);
    }
}

/// "3:07", or "1:02:44" past an hour.
pub fn clock(ms: u64) -> String {
    let s = ms / 1000;
    match s >= 3600 {
        true => format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60),
        false => format!("{}:{:02}", s / 60, s % 60),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_read_like_a_player_s() {
        assert_eq!(clock(0), "0:00");
        assert_eq!(clock(187_400), "3:07");
        assert_eq!(clock(3_764_000), "1:02:44");
    }

    #[test]
    fn the_fonts_load_and_draw_japanese_too() {
        let mut s = Screen::default();
        assert!(s.width("Coastline", Type::Title) > 40);
        assert!(s.width("光の街", Type::Title) > 30);
        assert!(s.width("1:23", Type::Small) > 8);
    }
}
