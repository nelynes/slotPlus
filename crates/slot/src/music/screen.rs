//! slotPlus music cart: drawing the player's screen, a Game Boy Advance's 240x160, in
//! hard pixels like a real cart's.

use std::collections::HashMap;

use slot_retro::{GBA_H, GBA_W};

pub const W: usize = GBA_W as usize;
pub const H: usize = GBA_H as usize;

pub const BACK: [u8; 3] = [0x14, 0x1b, 0x2e];
pub const BAND: [u8; 3] = [0x22, 0x2d, 0x4a];
pub const INK: [u8; 3] = [0xe8, 0xec, 0xf4];
pub const DIM: [u8; 3] = [0x8a, 0x96, 0xb4];
pub const ACCENT: [u8; 3] = [0x5f, 0xd4, 0xc0];
pub const PICK: [u8; 3] = [0x33, 0x45, 0x72];

/// The screen as `xrgb8888`, with a cache of the glyphs it has drawn.
pub struct Screen {
    pub px: Vec<u8>,
    glyphs: HashMap<(char, u32), (fontdue::Metrics, Vec<u8>)>,
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
    fn put(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if x < 0 || y < 0 || x as usize >= W || y as usize >= H {
            return;
        }
        let at = (y as usize * W + x as usize) * 4;
        self.px[at..at + 4].copy_from_slice(&[c[2], c[1], c[0], 0xff]);
    }

    fn blend(&mut self, x: i32, y: i32, c: [u8; 3], a: u8) {
        if x < 0 || y < 0 || x as usize >= W || y as usize >= H {
            return;
        }
        let at = (y as usize * W + x as usize) * 4;
        for (i, ch) in [c[2], c[1], c[0]].into_iter().enumerate() {
            let old = self.px[at + i] as u32;
            self.px[at + i] = ((old * (255 - a as u32) + ch as u32 * a as u32) / 255) as u8;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3]) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }

    pub fn fill(&mut self, c: [u8; 3]) {
        self.rect(0, 0, W as i32, H as i32, c);
    }

    fn glyph(&mut self, ch: char, px: u32) -> Option<(fontdue::Metrics, Vec<u8>)> {
        if let Some(g) = self.glyphs.get(&(ch, px)) {
            return Some(g.clone());
        }
        let font = slot_ui::text::label_font()?;
        let g = font.rasterize(ch, px as f32);
        self.glyphs.insert((ch, px), g.clone());
        Some(g)
    }

    /// How wide `text` is at `px`.
    pub fn width(&mut self, text: &str, px: u32) -> i32 {
        let mut w = 0.0;
        for ch in text.chars() {
            if let Some((m, _)) = self.glyph(ch, px) {
                w += m.advance_width;
            }
        }
        w.round() as i32
    }

    /// One line of text in capitals, its top at `top`, cut off with an ellipsis where it
    /// would pass `right`. The edges are hardened, as a cart's font would be.
    pub fn text(&mut self, text: &str, px: u32, x: i32, top: i32, right: i32, c: [u8; 3]) {
        let Some(font) = slot_ui::text::label_font() else {
            return;
        };
        let Some(vm) = font.horizontal_line_metrics(px as f32) else {
            return;
        };
        let mut line: String = text.to_uppercase();
        if self.width(&line, px) > right - x {
            while !line.is_empty() && self.width(&format!("{line}…"), px) > right - x {
                line.pop();
            }
            line = format!("{}…", line.trim_end());
        }
        let base = top as f32 + vm.ascent;
        let mut pen = x as f32;
        for ch in line.chars() {
            let Some((m, cov)) = self.glyph(ch, px) else {
                continue;
            };
            let gx = (pen + m.xmin as f32).round() as i32;
            let gy = (base - (m.height as f32 + m.ymin as f32)).round() as i32;
            for r in 0..m.height {
                for col in 0..m.width {
                    let a = match cov[r * m.width + col] {
                        v if v >= 150 => 255,
                        v if v >= 70 => 140,
                        _ => continue,
                    };
                    self.blend(gx + col as i32, gy + r as i32, c, a);
                }
            }
            pen += m.advance_width;
        }
    }

    /// A small folder.
    pub fn folder(&mut self, x: i32, y: i32, c: [u8; 3]) {
        self.rect(x, y, 4, 1, c);
        self.rect(x, y + 1, 9, 6, c);
    }

    /// A note.
    pub fn note(&mut self, x: i32, y: i32, c: [u8; 3]) {
        self.rect(x + 5, y, 1, 6, c);
        self.rect(x + 6, y, 2, 1, c);
        self.rect(x + 7, y + 1, 1, 1, c);
        self.rect(x + 2, y + 5, 3, 2, c);
        self.rect(x + 3, y + 4, 2, 1, c);
    }

    /// Play: a triangle pointing right, `s` tall.
    pub fn play(&mut self, x: i32, y: i32, s: i32, c: [u8; 3]) {
        for i in 0..s {
            let half = i.min(s - 1 - i);
            self.rect(x, y + i, half + 1, 1, c);
        }
    }

    pub fn pause(&mut self, x: i32, y: i32, s: i32, c: [u8; 3]) {
        self.rect(x, y, 2, s, c);
        self.rect(x + 4, y, 2, s, c);
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
}
