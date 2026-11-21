//! slotPlus music cart: drawing the player's screen, a Game Boy Advance's 240x160, in hard
//! pixels like a real cart's. The staircase, its doors and notes, the cart that climbs it,
//! and the band across the top, all drawn here from code: no picture files.

use std::collections::HashMap;
use std::sync::OnceLock;

use slot_retro::{GBA_H, GBA_W};

pub const W: usize = GBA_W as usize;
pub const H: usize = GBA_H as usize;

pub const BACK: [u8; 3] = [0, 0, 0];
pub const LINE: [u8; 3] = [0x58, 0xce, 0xd6];
pub const DIM: [u8; 3] = [0x2e, 0x80, 0x8c];
pub const NOTE: [u8; 3] = [0xf0, 0x6e, 0xc8];
pub const INK: [u8; 3] = [0xf0, 0xf6, 0xfa];
pub const PICK: [u8; 3] = [0x18, 0x34, 0xbe];
pub const PICK_STRIPE: [u8; 3] = [0x28, 0x5a, 0xe6];

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
    glyphs: HashMap<(char, Type), Glyph>,
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            px: vec![0; W * H * 4],
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
        self.put(x + 6, floor - 6, BACK);
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

    /// The EXIT door at the top of a folder: an archway with a list on it, and its word.
    pub fn exit_door(&mut self, x: i32, floor: i32) {
        self.vline(x, floor - 16, floor - 1, LINE);
        self.vline(x + 12, floor - 16, floor - 1, LINE);
        self.hline(x, x + 12, floor - 1, LINE);
        self.hline(x + 3, x + 9, floor - 20, LINE);
        self.put(x + 1, floor - 17, LINE);
        self.put(x + 2, floor - 18, LINE);
        self.put(x + 2, floor - 19, LINE);
        self.put(x + 11, floor - 17, LINE);
        self.put(x + 10, floor - 18, LINE);
        self.put(x + 10, floor - 19, LINE);
        for (k, len) in [2, 4, 6].into_iter().enumerate() {
            self.hline(x + 3, x + 3 + len, floor - 12 + 3 * k as i32, LINE);
        }
        self.text(
            "EXIT",
            Type::Small,
            x - 2,
            floor + 1,
            (0, W as i32),
            0,
            LINE,
        );
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
                    px(self, xx, yy, INK);
                }
            }
        }
        // Its label, with a face.
        for yy in y - 13..=y - 8 {
            for xx in x + 3..=x + 7 {
                px(self, xx, yy, DIM);
            }
        }
        px(self, x + 4, y - 11, INK);
        px(self, x + 6, y - 11, INK);
        for xx in x + 4..=x + 6 {
            px(self, xx, y - 9, INK);
        }
        // The contacts' edge.
        for xx in x + 2..=x + 8 {
            px(self, xx, y - 4, INK);
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
            px(self, x + dx, y + dy, INK);
        }
    }

    /// The B button, round, in the corner.
    pub fn b_button(&mut self, x: i32, y: i32) {
        for yy in 0..11 {
            for xx in 0..11 {
                let (dx, dy) = (xx as f32 - 5.0, yy as f32 - 5.0);
                if dx * dx + dy * dy <= 30.0 {
                    self.put(x + xx, y + yy, LINE);
                }
            }
        }
        self.text("B", Type::Small, x + 3, y, (x, x + 10), 0, BACK);
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
