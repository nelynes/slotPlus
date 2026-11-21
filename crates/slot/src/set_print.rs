//! slotPlus: printing a game's achievement set on the Game Boy Printer, from the opened cart.
//!
//! The list is read from the card and laid out on a worker, and kept as a picture under the
//! game's screenshots. The printer scene is the link receipt's; once the sheet is down,
//! Up and Down move the list through it.
//!
//! A screenshot prints the same way, from the screenshot viewer, and is kept beside the
//! screenshot it came from.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use slot_input::Millis;
use slot_store::{Cart, Zone};
use slot_ui::printout::{PrintBadge, PrintList, PrintRow, PrintShot, Printout};
use slot_ui::UndoFace;

use crate::print_scene::{Cue, PrintFrame, PrintScene, PrintTextures};

/// One press moves the list this far.
const STEP: f32 = 48.0;
/// Held, after `HOLD_MS`, it runs at this many pixels a second.
const SPEED: f32 = 420.0;
const HOLD_MS: Millis = 280;
/// How quickly the shown list catches up with where it's going.
const EASE_MS: f32 = 70.0;

pub struct SetPrint {
    built: Option<Receiver<Option<Printout>>>,
    printout: Option<Printout>,
    scene: Option<PrintScene>,
    tex: Option<PrintTextures>,
    /// Where the list sat when the paper was last drawn.
    shown: Option<usize>,
    scroll: f32,
    target: f32,
    held: Option<(f32, Millis)>,
    last: Millis,
    stop_asked: bool,
    owed: Vec<Cue>,
    /// For tests: run the scene without the printer's textures.
    bare: bool,
}

const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

/// "16 NOV 2026", in the card's own time zone.
pub fn day_text(utc: u64, zone: Zone) -> String {
    local_day_text(zone.local(utc as i64))
}

fn local_day_text(local: i64) -> String {
    let (y, m, d) = slot_store::civil_from_days(local.div_euclid(86_400));
    format!("{d} {} {y}", MONTHS[(m - 1).clamp(0, 11) as usize])
}

/// "20 NOV 2026 · 10:45 PM": when a screenshot was taken, from its name, which is
/// already the card's local time.
pub fn taken_text(stamp: &str, clock_24h: bool) -> Option<String> {
    let local = slot_store::parse_stamp(stamp.get(..19)?)?;
    Some(format!(
        "{} · {}",
        local_day_text(local),
        slot_ui::time_text(local, clock_24h)
    ))
}

/// Lay out a screenshot for the printer; `None` when it can't be read.
pub fn build_shot(cart: &Cart, shot: &slot_store::ShotEntry, clock_24h: bool) -> Option<Printout> {
    let (rgba, w, h) = slot_ui::decode_png(&shot.path)?;
    Some(Printout::shot(&PrintShot {
        game: cart.title.clone(),
        taken: taken_text(&shot.stamp, clock_24h).unwrap_or_default(),
        rgba,
        w,
        h,
    }))
}

/// Where a screenshot's print is kept: beside it, under its own name. Not a screenshot's
/// name, so the viewer passes over it.
pub fn shot_keep_path(shot: &Path) -> PathBuf {
    let stem = shot.file_stem().and_then(|s| s.to_str()).unwrap_or("Shot");
    shot.with_file_name(format!("{stem} Print.png"))
}

/// Read a game's set from the card and lay it out; `None` when it has no set there.
pub fn build(root: &Path, cart: &Cart, zone: Zone, now: u64) -> Option<Printout> {
    let set = slot_achievements::set_list(root, &cart.rom)?;
    let rows = set
        .entries
        .iter()
        .map(|e| PrintRow {
            title: e.title.clone(),
            description: e.description.clone(),
            points: e.points,
            badge: e
                .badge
                .as_deref()
                .and_then(slot_achievements::load_badge)
                .map(|b| PrintBadge {
                    w: b.width,
                    h: b.height,
                    rgba: b.rgba,
                }),
            earned: e.earned,
            date: e.earned_at.map(|t| day_text(t, zone)),
        })
        .collect();
    let list = PrintList {
        game: set.game.clone(),
        rows,
        printed: day_text(now, zone),
    };
    Some(Printout::new(&list))
}

/// Where a printout is kept: beside the game's screenshots, named by when it was printed.
/// The name isn't a screenshot's, so the screenshot viewer passes over it.
pub fn keep_path(root: &Path, cart: &Cart, now: u64, zone: Zone) -> PathBuf {
    let stamp = slot_store::format_stamp(zone.local(now as i64));
    root.join(slot_store::SCREENSHOTS_DIR)
        .join(cart.platform.dir_name())
        .join(&cart.stem)
        .join(format!("{stamp} Achievements.png"))
}

fn keep(path: &Path, face: &UndoFace) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), face.w, face.h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(&face.rgba))
        .map_err(std::io::Error::other)
}

/// Lay a printout out on a worker and keep it at the path it names.
fn on_worker<F>(work: F) -> Receiver<Option<Printout>>
where
    F: FnOnce() -> Option<(Printout, PathBuf)> + Send + 'static,
{
    let (outbox, built) = mpsc::channel();
    let spawned = thread::Builder::new()
        .name("slot-set-print".into())
        .spawn(move || {
            let printout = work().map(|(p, path)| {
                if let Err(e) = keep(&path, &p.whole()) {
                    eprintln!("slot: print: could not keep {}: {e}", path.display());
                }
                p
            });
            let _ = outbox.send(printout);
        });
    if let Err(e) = spawned {
        eprintln!("slot: print: worker thread failed to start: {e}");
    }
    built
}

impl SetPrint {
    pub fn open(root: PathBuf, cart: Cart, zone: Zone, now: Millis) -> SetPrint {
        let built = on_worker(move || {
            let unix = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            let printout = build(&root, &cart, zone, unix)?;
            Some((printout, keep_path(&root, &cart, unix, zone)))
        });
        SetPrint::waiting(built, now)
    }

    /// slotPlus: print a screenshot.
    pub fn shot(cart: Cart, shot: slot_store::ShotEntry, clock_24h: bool, now: Millis) -> SetPrint {
        let built = on_worker(move || {
            let printout = build_shot(&cart, &shot, clock_24h)?;
            Some((printout, shot_keep_path(&shot.path)))
        });
        SetPrint::waiting(built, now)
    }

    fn waiting(built: Receiver<Option<Printout>>, now: Millis) -> SetPrint {
        SetPrint {
            built: Some(built),
            printout: None,
            scene: None,
            tex: None,
            shown: None,
            scroll: 0.0,
            target: 0.0,
            held: None,
            last: now,
            stop_asked: false,
            owed: Vec::new(),
            bare: false,
        }
    }

    /// For tests: a printout already laid out.
    pub fn with(printout: Printout, now: Millis) -> SetPrint {
        let (outbox, built) = mpsc::channel();
        let _ = outbox.send(Some(printout));
        SetPrint {
            bare: true,
            ..SetPrint::waiting(built, now)
        }
    }

    pub fn set_tex(&mut self, tex: PrintTextures) {
        self.tex = Some(tex);
    }

    pub fn poll(&mut self, now: Millis) {
        if let Some(built) = &self.built {
            match built.try_recv() {
                Ok(Some(p)) => {
                    self.printout = Some(p);
                    self.built = None;
                }
                Ok(None) | Err(TryRecvError::Disconnected) => {
                    self.built = None;
                    self.stop(now);
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if (self.tex.is_some() || self.bare) && self.scene.is_none() && !self.stop_asked {
            self.scene = Some(PrintScene::new(now));
        }
        let up = self.shown.is_some();
        if let Some(scene) = self.scene.as_mut().filter(|_| !up) {
            scene.hold_print(now);
        }
        self.glide(now);
        if let Some(scene) = &mut self.scene {
            self.owed.extend(scene.cues(now));
        }
    }

    fn glide(&mut self, now: Millis) {
        let dt = now.saturating_sub(self.last) as f32;
        self.last = now;
        let max = self.printout.as_ref().map_or(0, Printout::max_scroll) as f32;
        if let Some((dir, since)) = self.held {
            if now.saturating_sub(since) >= HOLD_MS {
                self.target += dir * SPEED * dt / 1000.0;
            }
        }
        self.target = self.target.clamp(0.0, max);
        let k = (dt / EASE_MS).min(1.0);
        self.scroll += (self.target - self.scroll) * k;
        if (self.target - self.scroll).abs() < 0.5 {
            self.scroll = self.target;
        }
    }

    /// The sheet to put on the paper, when the list has moved since it was last drawn.
    pub fn take_sheet(&mut self) -> Option<UndoFace> {
        let at = self.scroll.round() as usize;
        let p = self.printout.as_ref()?;
        if self.shown == Some(at) {
            return None;
        }
        self.shown = Some(at);
        Some(p.window(at))
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    fn resting(&self, now: Millis) -> bool {
        self.scene.as_ref().is_some_and(|s| s.at_rest(now))
    }

    /// Up or Down pressed (`dir` -1 or 1) or let go (`None`).
    pub fn hold(&mut self, dir: Option<f32>, now: Millis) {
        match dir {
            Some(dir) if self.resting(now) => {
                self.target += dir * STEP;
                self.held = Some((dir, now));
            }
            Some(_) => {}
            None => self.held = None,
        }
    }

    /// A lets the printer get on with it.
    pub fn skip(&mut self, now: Millis) {
        if let Some(scene) = &mut self.scene {
            scene.skip(now);
        }
    }

    /// B puts it away.
    pub fn stop(&mut self, now: Millis) {
        self.stop_asked = true;
        self.held = None;
        if let Some(scene) = &mut self.scene {
            scene.stop(now);
            self.owed.extend(scene.cues(now));
        }
    }

    pub fn take_cues(&mut self) -> Vec<Cue> {
        std::mem::take(&mut self.owed)
    }

    pub fn frame(&self, now: Millis) -> Option<(PrintFrame, PrintTextures)> {
        Some((self.scene.as_ref()?.frame(now), self.tex?))
    }

    pub fn finished(&self, now: Millis) -> bool {
        self.stop_asked && self.scene.as_ref().is_none_or(|s| s.done(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slot_ui::printout::PrintList;

    #[test]
    fn days_read_in_the_card_s_time_zone() {
        // 2026-11-16 03:00 UTC.
        let t = 1_794_798_000;
        assert_eq!(day_text(t, Zone::default()), "16 NOV 2026");
    }

    fn long() -> Printout {
        Printout::new(&PrintList {
            game: "Test".into(),
            printed: "16 NOV 2026".into(),
            rows: (0..30)
                .map(|i| PrintRow {
                    title: format!("Number {i}"),
                    description: "Words".into(),
                    points: 5,
                    badge: None,
                    earned: i % 2 == 0,
                    date: None,
                })
                .collect(),
        })
    }

    /// Run the print forward to `to`, a frame at a time.
    fn run(p: &mut SetPrint, from: Millis, to: Millis) {
        let mut t = from;
        while t < to {
            t += 16;
            p.poll(t);
            p.take_sheet();
        }
    }

    #[test]
    fn the_list_scrolls_only_once_the_sheet_is_down_and_stops_at_its_ends() {
        let mut p = SetPrint::with(long(), 0);
        p.poll(0);
        assert!(
            p.take_sheet().is_some(),
            "the top of the list goes on the paper first"
        );
        assert!(
            p.take_sheet().is_none(),
            "and isn't redrawn while it holds still"
        );
        p.hold(Some(1.0), 100);
        p.hold(None, 120);
        run(&mut p, 0, 600);
        assert_eq!(p.scroll(), 0.0, "still printing: Down does nothing yet");

        run(&mut p, 600, 8_000);
        p.hold(Some(1.0), 8_000);
        p.hold(None, 8_010);
        run(&mut p, 8_000, 8_500);
        assert_eq!(p.scroll(), STEP, "one press, one step");
        assert!(p.take_sheet().is_none(), "already drawn where it stopped");

        p.hold(Some(1.0), 8_500);
        run(&mut p, 8_500, 60_000);
        let max = p.printout.as_ref().unwrap().max_scroll() as f32;
        assert_eq!(p.scroll(), max, "held, it runs to the end and stops");
        p.hold(None, 60_000);
        p.hold(Some(-1.0), 60_000);
        p.hold(None, 60_010);
        run(&mut p, 60_000, 60_500);
        assert_eq!(p.scroll(), max - STEP);
    }

    #[test]
    fn b_puts_it_away() {
        let mut p = SetPrint::with(long(), 0);
        run(&mut p, 0, 8_000);
        assert!(!p.finished(8_000));
        p.stop(8_000);
        run(&mut p, 8_000, 12_000);
        assert!(p.finished(12_000));
    }

    #[test]
    fn a_shot_s_print_is_kept_beside_it_and_the_viewer_passes_over_it() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path().join("Screenshots/GB/Tetris");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("2026-11-20_22-45-01-250.png");
        let face = UndoFace {
            rgba: [40u8, 80, 120, 255].repeat(160 * 144),
            w: 160,
            h: 144,
        };
        keep(&path, &face).unwrap();
        let entry = slot_store::ShotEntry {
            stamp: "2026-11-20_22-45-01-250".into(),
            path: path.clone(),
        };
        let cart = Cart {
            platform: slot_store::Platform::Gb,
            stem: "Tetris".into(),
            rom: d.path().join("Games/GB/Tetris.gb"),
            label: None,
            title: "Tetris".into(),
            code: String::new(),
            shell: None,
            tags: Vec::new(),
        };
        let printout = build_shot(&cart, &entry, false).expect("the shot reads");
        let kept = shot_keep_path(&path);
        assert!(kept.ends_with("Screenshots/GB/Tetris/2026-11-20_22-45-01-250 Print.png"));
        keep(&kept, &printout.whole()).unwrap();
        let shots = slot_store::ShotDir::new(d.path(), cart.platform, &cart.stem);
        assert_eq!(
            shots.list().unwrap(),
            vec![entry.clone()],
            "only the shot itself"
        );
        assert_eq!(
            taken_text("2026-11-20_22-45-01-250", false).as_deref(),
            Some("20 NOV 2026 · 10:45 PM")
        );
        assert_eq!(
            taken_text("2026-11-20_22-45-01-250", true).as_deref(),
            Some("20 NOV 2026 · 22:45")
        );
        assert!(build_shot(
            &cart,
            &slot_store::ShotEntry {
                stamp: entry.stamp.clone(),
                path: dir.join("missing.png"),
            },
            false
        )
        .is_none());
    }

    #[test]
    fn a_kept_printout_sits_with_the_game_s_screenshots_but_is_not_one() {
        let d = tempfile::tempdir().unwrap();
        let cart = Cart {
            platform: slot_store::Platform::Gb,
            stem: "Tetris".into(),
            rom: d.path().join("Games/GB/Tetris.gb"),
            label: None,
            title: "Tetris".into(),
            code: String::new(),
            shell: None,
            tags: Vec::new(),
        };
        let path = keep_path(d.path(), &cart, 1_794_798_000, Zone::default());
        assert!(path.ends_with("Screenshots/GB/Tetris/2026-11-16_03-00-00 Achievements.png"));
        keep(&path, &long().whole()).unwrap();
        let shots = slot_store::ShotDir::new(d.path(), cart.platform, &cart.stem);
        assert!(
            shots.list().unwrap().is_empty(),
            "the viewer passes over it"
        );
    }
}
