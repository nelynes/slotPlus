//! slotPlus: a game's achievement set printed on the link receipt's paper.
//!
//! The whole list is laid out once as one tall strip of ink. The printer's sheet stays the
//! size it always is, and shows a window onto the strip: scrolling moves the strip through
//! the window. Each window is cheap to make (the paper's texture is worked out once, a
//! sheet tall, and repeats down the strip), so it can be remade as the list scrolls.

use crate::plate::UndoFace;
use crate::quick_menu::print_list_legends;
use crate::receipt::{
    blur, blur_line, kernel, tooth_cut, Rng, CUT, INK, PAPER, PH, PW, STOP_DARK, STOP_LIGHT, STOP_Y,
};
use crate::text;

/// Where the list shows on the sheet: below the top teeth, above the printed legends.
const VIEW_TOP: usize = 10;
const VIEW_BOTTOM: usize = STOP_Y - 10;
pub const VIEW_H: usize = VIEW_BOTTOM - VIEW_TOP;
/// How far the list fades out where it runs past the window.
const FADE: usize = 10;
const SIDE: usize = 12;
const BADGE: usize = 36;
const TEXT_X: usize = SIDE + BADGE + 9;
const TRACK_X: usize = PW - 6;
/// Ink for an achievement not earned yet.
const FAINT: f32 = 0.42;

/// A badge's picture, straight from the card.
pub struct PrintBadge {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

pub struct PrintRow {
    pub title: String,
    pub description: String,
    pub points: u32,
    pub badge: Option<PrintBadge>,
    pub earned: bool,
    /// When it was earned, already written out ("16 NOV 2026"), if the card knows.
    pub date: Option<String>,
}

pub struct PrintList {
    pub game: String,
    pub rows: Vec<PrintRow>,
    /// The day it was printed, written out.
    pub printed: String,
}

/// The paper's texture for one sheet's height, repeated down the strip.
struct Paper {
    press: Vec<f32>,
    grain: Vec<f32>,
    phase: f32,
}

impl Paper {
    fn new() -> Paper {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ 3);
        let head = {
            let row = rng.field(PW);
            let mut out = vec![0.0; PW];
            blur_line(&row, &mut out, 1, PW, &kernel(1.2));
            out
        };
        let phase = rng.next() * 3.0;
        let mottle = blur(&rng.field(PW * PH), PW, PH, 0.8);
        let grain = blur(&rng.field(PW * PH), PW, PH, 1.1);
        let mut press = vec![0.0; PW * PH];
        for y in 0..PH {
            for x in 0..PW {
                let at = y * PW + x;
                let head = 0.95 + 0.05 * head[x] * 2.2;
                let grid = if y % 2 == 0 || x % 2 == 0 { 1.0 } else { 0.0 };
                let mot = 1.0 + mottle[at] * 0.16;
                press[at] = head * mot * (0.90 + 0.10 * grid);
            }
        }
        Paper {
            press,
            grain,
            phase,
        }
    }

    /// One pixel of printed paper: `a` is how much ink, `strip_y` the row down the strip
    /// (for the texture), and `shade` the sheet's own lighting at this spot.
    fn band(&self, strip_y: usize) -> f32 {
        0.97 + 0.03 * (strip_y as f32 / 3.7 + self.phase).sin().signum()
    }

    fn pixel(&self, x: usize, strip_y: usize, band: f32, a: f32, shade: (f32, f32)) -> [u8; 4] {
        let at = (strip_y % PH) * PW + x;
        let press = (self.press[at] * band).clamp(0.0, 1.0);
        let a = a * press;
        let mut out = [0, 0, 0, 255];
        for c in 0..3 {
            let paper = PAPER[c] * (1.0 + self.grain[at] * 0.012);
            let col = paper * (1.0 - a) + INK[c] * a;
            out[c] = ((col * shade.0 + shade.1).clamp(0.0, 1.0) * 255.0) as u8;
        }
        out
    }
}

/// The sheet's lighting down its length: how it curls at its ends.
fn curl(y: usize, h: usize) -> f32 {
    let v = y as f32 / (h - 1) as f32;
    1.0 - 0.09 * (-(v / 0.055).powi(2)).exp() - 0.05 * (-((1.0 - v) / 0.045).powi(2)).exp()
}

/// And across it: how it rolls, and its sheen.
fn across() -> Vec<(f32, f32)> {
    (0..PW)
        .map(|x| {
            let u = (x as f32 / (PW - 1) as f32 - 0.5) * 2.0;
            let roll = 0.90 + 0.10 * (u * 1.35).cos();
            let sheen = 0.075 * (-((u + 0.34) / 0.20).powi(2)).exp();
            (roll, sheen)
        })
        .collect()
}

/// Ink laid out on a strip `PW` wide.
struct Strip {
    ink: Vec<f32>,
    h: usize,
}

impl Strip {
    fn new() -> Strip {
        Strip {
            ink: Vec::new(),
            h: 0,
        }
    }

    fn grow(&mut self, to: usize) {
        if to > self.h {
            self.h = to;
            self.ink.resize(PW * to, 0.0);
        }
    }

    fn put(&mut self, x: i32, y: i32, a: f32) {
        if x < 0 || y < 0 || x as usize >= PW {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        self.grow(y + 1);
        let at = y * PW + x;
        self.ink[at] = self.ink[at].max(a);
    }

    fn clear(&mut self, x: i32, y: i32) {
        if x >= 0 && y >= 0 && (x as usize) < PW && (y as usize) < self.h {
            self.ink[y as usize * PW + x as usize] = 0.0;
        }
    }

    /// One line of text with its top at `top`, starting at `x` (or centred when `None`).
    /// Returns the line's height.
    fn line(&mut self, text: &str, px: f32, x: Option<f32>, top: f32, a: f32) -> f32 {
        let Some(font) = text::label_font() else {
            return px;
        };
        let Some(vm) = font.horizontal_line_metrics(px) else {
            return px;
        };
        let base = top + vm.ascent;
        let mut pen = x.unwrap_or((PW as f32 - text::line_width(font, text, px, 0.0)) / 2.0);
        for ch in text.chars() {
            let (m, cov) = font.rasterize(ch, px);
            let gx = (pen + m.xmin as f32).round() as i32;
            let gy = (base - (m.height as f32 + m.ymin as f32)).round() as i32;
            for r in 0..m.height {
                for c in 0..m.width {
                    let v = cov[r * m.width + c] as f32 / 255.0;
                    if v > 0.0 {
                        self.put(gx + c as i32, gy + r as i32, v * a);
                    }
                }
            }
            pen += m.advance_width;
        }
        (vm.ascent - vm.descent + vm.line_gap).ceil()
    }

    /// Text wrapped to `width`, at most `most` lines (the last one cut short with an
    /// ellipsis). Returns where the text ends.
    #[allow(clippy::too_many_arguments)]
    fn wrapped(
        &mut self,
        text: &str,
        px: f32,
        x: Option<f32>,
        width: f32,
        top: f32,
        a: f32,
        most: usize,
    ) -> f32 {
        let mut y = top;
        for line in wrap(text, px, width, most) {
            y += self.line(&line, px, x, y, a);
        }
        y
    }

    fn rule(&mut self, y: usize, dashed: bool) {
        for x in SIDE..PW - SIDE {
            if !dashed || (x / 4) % 2 == 0 {
                self.put(x as i32, y as i32, 0.55);
            }
        }
    }

    fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, a: f32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx as i32, yy as i32, a);
            }
        }
    }

    fn frame(&mut self, x: usize, y: usize, w: usize, h: usize, t: usize, a: f32) {
        self.rect(x, y, w, t, a);
        self.rect(x, y + h - t, w, t, a);
        self.rect(x, y, t, h, a);
        self.rect(x + w - t, y, t, h, a);
    }

    /// A badge in grey, dithered the way a thermal head prints a picture.
    fn badge(&mut self, badge: Option<&PrintBadge>, x: usize, y: usize, a: f32) {
        const BAYER: [[f32; 4]; 4] = [
            [0.0, 8.0, 2.0, 10.0],
            [12.0, 4.0, 14.0, 6.0],
            [3.0, 11.0, 1.0, 9.0],
            [15.0, 7.0, 13.0, 5.0],
        ];
        let Some(b) = badge.filter(|b| b.w > 0 && b.h > 0) else {
            self.frame(x, y, BADGE, BADGE, 1, 0.5 * a);
            return;
        };
        for dy in 0..BADGE {
            for dx in 0..BADGE {
                let sx = (dx * b.w as usize / BADGE).min(b.w as usize - 1);
                let sy = (dy * b.h as usize / BADGE).min(b.h as usize - 1);
                let p = &b.rgba[(sy * b.w as usize + sx) * 4..][..4];
                let lum = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
                let dark = (1.0 - lum) * p[3] as f32 / 255.0;
                // Lift the darks a touch so a dark badge doesn't print as a black square.
                let dark = (dark * 0.9).powf(0.9);
                if dark > (BAYER[dy % 4][dx % 4] + 0.5) / 16.0 {
                    self.put((x + dx) as i32, (y + dy) as i32, a);
                }
            }
        }
        self.frame(x, y, BADGE, BADGE, 1, 0.8 * a);
    }

    /// A small black box with a check mark knocked out of it, on an earned badge's corner.
    fn check(&mut self, x: usize, y: usize) {
        let s = 13;
        self.rect(x, y, s, s, 1.0);
        let (x, y) = (x as i32, y as i32);
        let mark = [
            (3, 7),
            (4, 8),
            (5, 9),
            (6, 8),
            (7, 7),
            (8, 6),
            (9, 5),
            (10, 4),
        ];
        for (mx, my) in mark {
            for t in 0..2 {
                self.clear(x + mx, y + my - t);
            }
        }
    }
}

/// Words broken into lines no wider than `width`; a word too long for a line is cut.
fn wrap(text: &str, px: f32, width: f32, most: usize) -> Vec<String> {
    let Some(font) = text::label_font() else {
        return vec![text.to_string()];
    };
    let fits = |s: &str| text::line_width(font, s, px, 0.0) <= width;
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let tried = match line.is_empty() {
            true => word.to_string(),
            false => format!("{line} {word}"),
        };
        if fits(&tried) {
            line = tried;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        line = word.to_string();
        while !fits(&line) && line.chars().count() > 1 {
            line.pop();
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.len() > most {
        lines.truncate(most);
        let last = lines.last_mut().expect("at least one line");
        while !fits(&format!("{last}…")) && !last.is_empty() {
            last.pop();
        }
        last.push('…');
    }
    lines
}

/// The list laid out and ready to show a window at a time.
pub struct Printout {
    strip: Strip,
    paper: Paper,
    legends: Vec<(UndoFace, usize)>,
}

impl Printout {
    pub fn new(list: &PrintList) -> Printout {
        let mut s = Strip::new();
        let earned = list.rows.iter().filter(|r| r.earned).count();
        let total = list.rows.len();
        let points: u32 = list.rows.iter().map(|r| r.points).sum();
        let got: u32 = list
            .rows
            .iter()
            .filter(|r| r.earned)
            .map(|r| r.points)
            .sum();
        let wide = (PW - 2 * SIDE) as f32;

        let mut y = 14.0;
        y += s.line("ACHIEVEMENTS", 13.0, None, y, 0.8) + 6.0;
        y = s.wrapped(&list.game.to_uppercase(), 19.0, None, wide, y, 1.0, 3) + 4.0;
        y += s.line(&format!("{earned} / {total}"), 30.0, None, y, 1.0);
        // How far along, as a bar.
        let bar_y = y.round() as usize + 4;
        s.frame(SIDE + 8, bar_y, PW - 2 * SIDE - 16, 10, 1, 0.9);
        let fill = (PW - 2 * SIDE - 20) * earned / total.max(1);
        s.rect(SIDE + 10, bar_y + 2, fill, 6, 0.9);
        y = (bar_y + 18) as f32;
        if total > 0 && earned == total {
            let (w, h) = (132, 36);
            let x = (PW - w) / 2;
            s.frame(x, y as usize, w, h, 2, 1.0);
            s.frame(x + 4, y as usize + 4, w - 8, h - 8, 1, 1.0);
            s.line("MASTERED", 16.0, None, y + 8.0, 1.0);
            y += h as f32 + 10.0;
        }
        s.rule(y as usize, true);
        y += 10.0;

        let text_w = (PW - TEXT_X - SIDE) as f32;
        for row in &list.rows {
            let a = if row.earned { 1.0 } else { FAINT };
            let top = y;
            s.badge(row.badge.as_ref(), SIDE, top as usize, a);
            if row.earned {
                s.check(SIDE + BADGE - 9, top as usize + BADGE - 9);
            }
            let mut ty = s.wrapped(
                &row.title.to_uppercase(),
                12.0,
                Some(TEXT_X as f32),
                text_w,
                top,
                a,
                2,
            );
            ty = s.wrapped(
                &row.description,
                11.0,
                Some(TEXT_X as f32),
                text_w,
                ty + 1.0,
                a * 0.9,
                5,
            );
            let pts = match row.points {
                1 => "1 PT".to_string(),
                n => format!("{n} PTS"),
            };
            let meta = match (row.earned, &row.date) {
                (true, Some(date)) => format!("{pts}  ·  {date}"),
                (true, None) => format!("{pts}  ·  EARNED"),
                (false, _) => pts,
            };
            ty += 2.0 + s.line(&meta, 10.0, Some(TEXT_X as f32), ty + 2.0, a * 0.85);
            y = ty.max(top + BADGE as f32) + 8.0;
            s.rule(y as usize, true);
            y += 9.0;
        }

        y += 2.0;
        y += s.line(&format!("POINTS  {got} / {points}"), 13.0, None, y, 1.0) + 4.0;
        y += s.line(&format!("PRINTED {}", list.printed), 11.0, None, y, 0.8) + 8.0;
        y += s.line("SLOTPLUS", 10.0, None, y, 0.6);
        s.grow(y as usize + 18);

        let legends = print_list_legends([0, 0, 0]);
        let scrolls = s.h > VIEW_H;
        let mut placed = Vec::new();
        let gap = 14;
        let [scroll, done] = legends;
        let row: Vec<UndoFace> = match scrolls {
            true => vec![scroll, done],
            false => vec![done],
        };
        let width: usize = row.iter().map(|l| l.w as usize).sum::<usize>() + gap * (row.len() - 1);
        let mut x = PW.saturating_sub(width) / 2;
        for legend in row {
            let w = legend.w as usize;
            placed.push((legend, x));
            x += w + gap;
        }
        Printout {
            strip: s,
            paper: Paper::new(),
            legends: placed,
        }
    }

    /// How tall the whole list is.
    pub fn height(&self) -> usize {
        self.strip.h
    }

    /// How far the list can scroll.
    pub fn max_scroll(&self) -> usize {
        self.strip.h.saturating_sub(VIEW_H)
    }

    /// The printer's sheet with the list `scroll` pixels down showing through it.
    pub fn window(&self, scroll: usize) -> UndoFace {
        let scroll = scroll.min(self.max_scroll());
        let mut ink = vec![0.0f32; PW * PH];
        for wy in VIEW_TOP..VIEW_BOTTOM {
            let sy = scroll + wy - VIEW_TOP;
            if sy >= self.strip.h {
                break;
            }
            // Fade where the list runs on past the window.
            let from_top = wy - VIEW_TOP;
            let from_bottom = VIEW_BOTTOM - 1 - wy;
            let mut fade = 1.0f32;
            if scroll > 0 && from_top < FADE {
                fade = fade.min(from_top as f32 / FADE as f32);
            }
            if scroll < self.max_scroll() && from_bottom < FADE {
                fade = fade.min(from_bottom as f32 / FADE as f32);
            }
            let row = &self.strip.ink[sy * PW..(sy + 1) * PW];
            for (x, v) in row.iter().enumerate() {
                ink[wy * PW + x] = v * fade;
            }
        }
        if self.max_scroll() > 0 {
            // Where the window is in the list: a thin track and a heavier thumb.
            let track = VIEW_BOTTOM - VIEW_TOP - 8;
            let thumb = (track * VIEW_H / self.strip.h).max(14);
            let at = VIEW_TOP + 4 + (track - thumb) * scroll / self.max_scroll();
            for y in VIEW_TOP + 4..VIEW_TOP + 4 + track {
                ink[y * PW + TRACK_X] = 0.25;
            }
            for y in at..at + thumb {
                for x in TRACK_X - 1..=TRACK_X + 1 {
                    ink[y * PW + x] = 0.85;
                }
            }
        }
        for (legend, x) in &self.legends {
            stamp(&mut ink, legend, *x, STOP_Y);
        }
        let mut rgba = vec![0u8; PW * PH * 4];
        let across = across();
        for y in 0..PH {
            let (curl, band) = (curl(y, PH), self.paper.band(scroll + y));
            for (x, (roll, sheen)) in across.iter().enumerate() {
                if tooth_cut(x, y) {
                    continue;
                }
                let at = y * PW + x;
                let px = self
                    .paper
                    .pixel(x, scroll + y, band, ink[at], (roll * curl, *sheen));
                rgba[at * 4..at * 4 + 4].copy_from_slice(&px);
            }
        }
        UndoFace {
            rgba,
            w: PW as u32,
            h: PH as u32,
        }
    }

    /// The whole list on one long strip of paper, to keep.
    pub fn whole(&self) -> UndoFace {
        let h = self.strip.h;
        let mut rgba = vec![0u8; PW * h * 4];
        let across = across();
        for y in 0..h {
            let (curl, band) = (curl(y, h.max(2)), self.paper.band(y));
            for (x, (roll, sheen)) in across.iter().enumerate() {
                let at = y * PW + x;
                let ink = self.strip.ink[at];
                let px = self.paper.pixel(x, y, band, ink, (roll * curl, *sheen));
                rgba[at * 4..at * 4 + 4].copy_from_slice(&px);
            }
        }
        UndoFace {
            rgba,
            w: PW as u32,
            h: h as u32,
        }
    }
}

/// A legend printed onto the paper as ink, the way the link receipt prints its own.
fn stamp(ink: &mut [f32], legend: &UndoFace, ox: usize, oy: usize) {
    let (w, h) = (legend.w as usize, legend.h as usize);
    for y in 0..h {
        for x in 0..w {
            let (px_x, px_y) = (ox + x, oy + y);
            if px_x >= PW || px_y >= CUT {
                continue;
            }
            let px = &legend.rgba[(y * w + x) * 4..][..4];
            let lum = (px[0] as f32 + px[1] as f32 + px[2] as f32) / 765.0;
            let dark = ((STOP_LIGHT - lum) / (STOP_LIGHT - STOP_DARK)).clamp(0.0, 1.0);
            let at = px_y * PW + px_x;
            ink[at] = ink[at].max(dark * px[3] as f32 / 255.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(n: usize, earned: usize) -> PrintList {
        PrintList {
            game: "Test Game".into(),
            rows: (0..n)
                .map(|i| PrintRow {
                    title: format!("Achievement {i}"),
                    description: "Do a thing that takes a few words to say".into(),
                    points: 5,
                    badge: None,
                    earned: i < earned,
                    date: (i == 0).then(|| "16 NOV 2026".into()),
                })
                .collect(),
            printed: "16 NOV 2026".into(),
        }
    }

    #[test]
    fn a_short_set_fits_the_sheet_and_does_not_scroll() {
        let p = Printout::new(&list(1, 1));
        assert_eq!(p.max_scroll(), 0);
        let w = p.window(50);
        assert_eq!((w.w as usize, w.h as usize), (PW, PH));
        assert_eq!(w.rgba, p.window(0).rgba, "nothing to scroll to");
    }

    #[test]
    fn a_long_set_scrolls_through_the_same_sheet() {
        let p = Printout::new(&list(40, 12));
        assert!(p.height() > 6 * PH, "forty rows run long: {}", p.height());
        assert!(p.max_scroll() > 0);
        let (top, lower) = (p.window(0), p.window(400));
        assert_eq!((lower.w as usize, lower.h as usize), (PW, PH));
        assert_ne!(top.rgba, lower.rgba);
        assert_eq!(p.window(usize::MAX).rgba, p.window(p.max_scroll()).rgba);
        let whole = p.whole();
        assert_eq!(whole.h as usize, p.height());
    }

    #[test]
    fn long_words_and_lines_are_wrapped_and_cut() {
        let lines = wrap(
            "one two three four five six seven eight nine ten eleven twelve",
            11.0,
            60.0,
            2,
        );
        assert_eq!(lines.len(), 2);
        assert!(lines[1].ends_with('…'));
    }
}
