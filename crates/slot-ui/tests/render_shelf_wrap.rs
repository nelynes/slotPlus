use slot_gfx::{Canvas, Draw, OUT_H, OUT_W};
use slot_store::{Cart, Platform};
use slot_ui::Shelf;

const SHRINK: usize = 3;
const CELL_W: usize = OUT_W as usize / SHRINK;
const CELL_H: usize = OUT_H as usize / SHRINK;
const COLS: usize = 8;
const DT: f32 = 1.0 / 60.0;
const FRAME_MS: u64 = 1000 / 60;

fn shelf_with(n: usize) -> Shelf {
    Shelf::new(
        (0..n)
            .map(|i| Cart {
                platform: Platform::Gba,
                stem: format!("Game {i}"),
                rom: format!("Games/GBA/Game {i}.gba").into(),
                label: None,
                code: String::new(),
                shell: None,
                title: format!("GAME {i}"),
            })
            .collect(),
    )
}

fn frame(s: &Shelf) -> Vec<u8> {
    let mut px = vec![0u8; CELL_W * CELL_H * 3];
    for p in px.chunks_mut(3) {
        p.copy_from_slice(&[18, 18, 22]);
    }
    let mut list = Vec::new();
    s.draw_row(Canvas::FRAMED, None, 0.0, 0.0, 1.0, &mut list);
    for d in &list {
        let Draw::Rect { x, y, w, h, colour } = *d else {
            continue;
        };
        let (x0, y0) = (x / SHRINK as f32, y / SHRINK as f32);
        let (x1, y1) = ((x + w) / SHRINK as f32, (y + h) / SHRINK as f32);
        for yy in y0.max(0.0) as usize..(y1.max(0.0) as usize).min(CELL_H) {
            for xx in x0.max(0.0) as usize..(x1.max(0.0) as usize).min(CELL_W) {
                let at = (yy * CELL_W + xx) * 3;
                for k in 0..3 {
                    let a = colour[3];
                    px[at + k] = (colour[k] * 255.0 * a + px[at + k] as f32 * (1.0 - a)) as u8;
                }
            }
        }
    }
    for yy in CELL_H - 3..CELL_H {
        for xx in CELL_W / 2 - 1..CELL_W / 2 + 1 {
            px[(yy * CELL_W + xx) * 3..][..3].copy_from_slice(&[255, 255, 255]);
        }
    }
    px
}

fn sheet(frames: &[Vec<u8>], path: &str) {
    let rows = frames.len().div_ceil(COLS);
    let (w, h) = (COLS * (CELL_W + 1) + 1, rows * (CELL_H + 1) + 1);
    let mut px = vec![90u8; w * h * 3];
    for (i, f) in frames.iter().enumerate() {
        let (cx, cy) = ((i % COLS) * (CELL_W + 1) + 1, (i / COLS) * (CELL_H + 1) + 1);
        for row in 0..CELL_H {
            let dst = ((cy + row) * w + cx) * 3;
            px[dst..dst + CELL_W * 3].copy_from_slice(&f[row * CELL_W * 3..][..CELL_W * 3]);
        }
    }
    let file = std::fs::File::create(path).unwrap();
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&px).unwrap();
    println!("wrote {path}: {} frames, {rows} rows", frames.len());
}

struct Run {
    frames: Vec<Vec<u8>>,
    scroll: Vec<f32>,
    gaps: Vec<(f32, f32)>,
}

impl Run {
    fn shot(&mut self, s: &Shelf) {
        self.frames.push(frame(s));
        self.scroll.push(s.scroll);
        self.gaps.push(widest_gap(s));
    }
}

fn run() -> Run {
    Run {
        frames: Vec::new(),
        scroll: Vec::new(),
        gaps: Vec::new(),
    }
}

fn hold(n: usize, start: usize, way: i32, frames: usize) -> Run {
    let mut s = shelf_with(n);
    s.select(start);
    let mut out = run();
    if way > 0 {
        s.hold_right(0);
    } else {
        s.hold_left(0);
    }
    for f in 0..frames {
        let now = f as u64 * FRAME_MS;
        s.tick(now);
        s.update(DT);
        out.shot(&s);
    }
    out
}

fn double_tap(n: usize, start: usize, way: i32, gap: usize, frames: usize) -> Run {
    let mut s = shelf_with(n);
    s.select(start);
    let press = |s: &mut Shelf| {
        if way > 0 {
            s.right()
        } else {
            s.left()
        }
    };
    press(&mut s);
    let mut out = run();
    for f in 0..frames {
        if f == gap {
            press(&mut s);
        }
        s.update(DT);
        out.shot(&s);
    }
    out
}

fn widest_gap(s: &Shelf) -> (f32, f32) {
    let mut list = Vec::new();
    s.draw_row(Canvas::FRAMED, None, 0.0, 0.0, 1.0, &mut list);
    let (mut left, mut right) = (OUT_W as f32, OUT_W as f32);
    for d in &list {
        if let Draw::Rect { x, w, .. } = *d {
            left = left.min(x.max(0.0));
            right = right.min((OUT_W as f32 - (x + w)).max(0.0));
        }
    }
    (left, right)
}

fn verdict(name: &str, way: i32, r: &Run) {
    let back: Vec<usize> = r
        .scroll
        .windows(2)
        .enumerate()
        .filter(|(_, w)| (w[1] - w[0]) * (way as f32) < -1e-4)
        .map(|(i, _)| i + 1)
        .collect();
    let total = r.scroll.last().unwrap() - r.scroll.first().unwrap();
    let left = r.gaps.iter().fold(0.0f32, |a, g| a.max(g.0));
    let right = r.gaps.iter().fold(0.0f32, |a, g| a.max(g.1));
    println!(
        "{name}: travelled {total:+.2} pitches over {} frames, {} of them against the press{}; \
         widest bare edge {left:.0} px left, {right:.0} px right",
        r.scroll.len(),
        back.len(),
        match back.first() {
            Some(f) => format!(", first at frame {f}"),
            None => String::new(),
        }
    );
}

#[test]
fn render_shelf_wrap() {
    let Ok(dir) = std::env::var("SCRATCH_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).unwrap();
    for n in [2usize, 3, 4, 10] {
        for (way, name) in [(1i32, "right"), (-1, "left")] {
            let start = if way > 0 { n - 1 } else { 0 };
            let r = hold(n, start, way, 72);
            verdict(&format!("n={n} hold {name}"), way, &r);
            sheet(&r.frames, &format!("{dir}/hold-{n}-{name}.png"));
            let r = double_tap(n, start, way, 3, 40);
            verdict(&format!("n={n} tap+tap {name}"), way, &r);
            sheet(&r.frames, &format!("{dir}/tap-{n}-{name}.png"));
        }
    }
}
