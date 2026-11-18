use std::path::Path;

use fontdue::{Font, FontSettings};

const GILL: &str = "/System/Library/Fonts/Supplemental/GillSans.ttc";
const FUTURA: &str = "/System/Library/Fonts/Supplemental/Futura.ttc";
const GILL_ITALIC: u32 = 2;
const FUTURA_MEDIUM: u32 = 0;
const FUTURA_BOLD: u32 = 2;

const OVER: u32 = 4;

struct Run<'a> {
    font: &'a Font,
    text: &'a str,
    px: f32,
    tracking: f32,
    lean: f32,
    bold: f32,
}

impl Run<'_> {
    fn scaled(&self, k: f32) -> Run<'_> {
        Run {
            font: self.font,
            text: self.text,
            px: self.px * k,
            tracking: self.tracking * k,
            lean: self.lean,
            bold: self.bold * k,
        }
    }
}

struct Canvas {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

impl Canvas {
    fn new(w: u32, h: u32) -> Canvas {
        Canvas {
            w: w * OVER,
            h: h * OVER,
            px: vec![0; (w * h * OVER * OVER) as usize],
        }
    }

    fn width(run: &Run) -> f32 {
        run.text
            .chars()
            .map(|c| run.font.metrics(c, run.px).advance_width + run.tracking)
            .sum::<f32>()
            - run.tracking
    }

    fn line(
        &mut self,
        runs: &[Run],
        gaps: &[f32],
        cx: f32,
        baseline: f32,
        max_w: f32,
    ) -> (f32, f32) {
        let width = |k: f32| -> f32 {
            runs.iter()
                .map(|r| Canvas::width(&r.scaled(k)))
                .sum::<f32>()
                + gaps.iter().sum::<f32>() * k
        };
        let mut k = 1.0;
        while k > 0.3 && width(k) > max_w {
            k -= 0.01;
        }
        let mut pen = cx - width(k) / 2.0;
        for (i, run) in runs.iter().enumerate() {
            let run = run.scaled(k);
            self.run(&run, pen, baseline);
            pen += Canvas::width(&run) + gaps.get(i).copied().unwrap_or(0.0) * k;
        }
        (pen, k)
    }

    fn run(&mut self, run: &Run, x: f32, baseline: f32) {
        let o = OVER as f32;
        let mut pen = x * o;
        let base = baseline * o;
        let grow = (run.bold * o).round() as i32;
        for ch in run.text.chars() {
            let (m, cov) = run.font.rasterize(ch, run.px * o);
            let top = base - (m.height as f32 + m.ymin as f32);
            for gy in 0..m.height {
                for gx in 0..m.width {
                    let a = cov[gy * m.width + gx];
                    if a == 0 {
                        continue;
                    }
                    let y = (top + gy as f32).round() as i32;
                    let lean = (base - y as f32) * run.lean;
                    let x0 = (pen + m.xmin as f32 + gx as f32 + lean).round() as i32;
                    for dx in 0..=grow {
                        self.put(x0 + dx, y, a);
                    }
                }
            }
            pen += m.advance_width + run.tracking * o;
        }
    }

    fn put(&mut self, x: i32, y: i32, a: u8) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let i = (y as u32 * self.w + x as u32) as usize;
        self.px[i] = self.px[i].max(a);
    }

    fn write(&self, path: &Path) {
        let (w, h) = (self.w / OVER, self.h / OVER);
        let mut out = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0u32;
                for sy in 0..OVER {
                    for sx in 0..OVER {
                        sum += self.px[((y * OVER + sy) * self.w + x * OVER + sx) as usize] as u32;
                    }
                }
                out.push((sum / (OVER * OVER)) as u8);
            }
        }
        let file = std::fs::File::create(path).expect("create mask");
        let mut enc = png::Encoder::new(file, w, h);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .and_then(|mut wr| wr.write_image_data(&out))
            .expect("write mask");
        println!("wrote {}", path.display());
    }
}

fn face(path: &str, index: u32) -> Font {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}; this runs on a Mac"));
    let settings = FontSettings {
        collection_index: index,
        ..FontSettings::default()
    };
    Font::from_bytes(bytes, settings).expect("font")
}

fn main() {
    let gill = face(GILL, GILL_ITALIC);
    let futura = face(FUTURA, FUTURA_MEDIUM);
    let futura_bold = face(FUTURA, FUTURA_BOLD);
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let (gb_w, gb_h) = slot_ui::seated_box(slot_store::Platform::Gb);

    let game_boy = |px: f32| Run {
        font: &gill,
        text: "GAME BOY",
        px,
        tracking: -0.6,
        lean: 0.0,
        bold: 0.6,
    };

    let mut gb = Canvas::new(gb_w, gb_h);
    let (right, k) = gb.line(
        &[
            Run {
                font: &futura_bold,
                text: "Nintendo",
                px: 13.0,
                tracking: -0.2,
                lean: 0.0,
                bold: 0.0,
            },
            game_boy(24.0),
        ],
        &[5.0],
        116.0,
        47.0,
        138.0,
    );
    gb.run(
        &Run {
            font: &futura,
            text: "TM",
            px: 5.5 * k,
            tracking: 0.0,
            lean: 0.0,
            bold: 0.0,
        },
        right + 1.0,
        47.0 - 12.0 * k,
    );
    gb.write(&assets.join("lettering_gb.png"));

    let mut gbc = Canvas::new(gb_w, gb_h);
    gbc.line(
        &[
            game_boy(22.0),
            Run {
                font: &futura_bold,
                text: "COLOR",
                px: 20.0,
                tracking: 0.4,
                lean: 0.2,
                bold: 0.0,
            },
        ],
        &[6.0],
        120.0,
        43.0,
        140.0,
    );
    gbc.write(&assets.join("lettering_gbc.png"));

    let (gba_w, gba_h) = slot_ui::seated_box(slot_store::Platform::Gba);
    let mut gba = Canvas::new(gba_w, gba_h);
    gba.line(
        &[
            Run {
                font: &futura_bold,
                text: "GAME BOY",
                px: 11.0,
                tracking: 0.0,
                lean: 0.2,
                bold: 0.0,
            },
            Run {
                font: &futura,
                text: "ADVANCE",
                px: 8.5,
                tracking: 1.6,
                lean: 0.0,
                bold: 0.0,
            },
        ],
        &[4.0],
        120.0,
        25.0,
        130.0,
    );
    gba.write(&assets.join("lettering_gba.png"));
}
