//! A screen recorder for making promo clips on the handheld itself, for a
//! capture build only (branch ds-capture, never merged). With an empty
//! Config/capture on the card, every fifth frame (12 fps) is read back,
//! scaled to 480x320 and written as RGB565 to Captures/capture-<time>.bin,
//! each frame headed by its time in ms (u32 LE). Frames the card can't keep
//! up with are dropped, not queued.

use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::time::Instant;

const EVERY: u64 = 5;
pub const OUT_W: usize = 480;
pub const OUT_H: usize = 320;

pub struct ScreenRecorder {
    tx: SyncSender<(u32, u32, u32, Vec<u8>)>,
    frame: u64,
    began: Instant,
    dropped: u64,
}

impl ScreenRecorder {
    pub fn for_card(root: &Path) -> Option<ScreenRecorder> {
        if !root.join("Config/capture").is_file() {
            return None;
        }
        let dir = root.join("Captures");
        std::fs::create_dir_all(&dir).ok()?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let path = dir.join(format!("capture-{stamp}.bin"));
        let file = std::fs::File::create(&path).ok()?;
        eprintln!("slot: capture: recording to {}", path.display());
        let (tx, rx) = sync_channel::<(u32, u32, u32, Vec<u8>)>(6);
        std::thread::Builder::new()
            .name("capture".into())
            .spawn(move || {
                let mut out = std::io::BufWriter::with_capacity(1 << 20, file);
                for (ms, w, h, rgba) in rx {
                    let frame = scale_565(&rgba, w as usize, h as usize);
                    if out.write_all(&ms.to_le_bytes()).is_err() || out.write_all(&frame).is_err() {
                        eprintln!("slot: capture: write failed, stopping");
                        return;
                    }
                }
                let _ = out.flush();
            })
            .ok()?;
        Some(ScreenRecorder {
            tx,
            frame: 0,
            began: Instant::now(),
            dropped: 0,
        })
    }

    /// Whether this frame is one to keep.
    pub fn wants(&mut self) -> bool {
        self.frame += 1;
        self.frame % EVERY == 0
    }

    pub fn keep(&mut self, w: u32, h: u32, rgba: Vec<u8>) {
        let ms = self.began.elapsed().as_millis() as u32;
        if let Err(TrySendError::Full(_)) = self.tx.try_send((ms, w, h, rgba)) {
            self.dropped += 1;
            if self.dropped.is_power_of_two() {
                eprintln!("slot: capture: {} frames dropped", self.dropped);
            }
        }
    }
}

/// `rgba` (w x h) scaled bilinearly to 480x320, as little-endian RGB565.
pub fn scale_565(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(OUT_W * OUT_H * 2);
    let (sx, sy) = (w as f32 / OUT_W as f32, h as f32 / OUT_H as f32);
    let px = |x: usize, y: usize, c: usize| f32::from(rgba[(y * w + x) * 4 + c]);
    for oy in 0..OUT_H {
        let fy = ((oy as f32 + 0.5) * sy - 0.5).clamp(0.0, (h - 1) as f32);
        let (y0, ty) = (fy as usize, fy.fract());
        let y1 = (y0 + 1).min(h - 1);
        for ox in 0..OUT_W {
            let fx = ((ox as f32 + 0.5) * sx - 0.5).clamp(0.0, (w - 1) as f32);
            let (x0, tx) = (fx as usize, fx.fract());
            let x1 = (x0 + 1).min(w - 1);
            let mut c = [0u16; 3];
            for (i, v) in c.iter_mut().enumerate() {
                let top = px(x0, y0, i) * (1.0 - tx) + px(x1, y0, i) * tx;
                let bottom = px(x0, y1, i) * (1.0 - tx) + px(x1, y1, i) * tx;
                *v = (top * (1.0 - ty) + bottom * ty).round() as u16;
            }
            let v = ((c[0] >> 3) << 11) | ((c[1] >> 2) << 5) | (c[2] >> 3);
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_frame_scales_to_its_own_colour() {
        let rgba: Vec<u8> = [255u8, 0, 0, 255].repeat(720 * 480);
        let out = scale_565(&rgba, 720, 480);
        assert_eq!(out.len(), OUT_W * OUT_H * 2);
        assert!(out
            .chunks(2)
            .all(|p| u16::from_le_bytes([p[0], p[1]]) == 0xf800));
    }
}
