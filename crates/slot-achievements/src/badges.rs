//! Badge files are downloaded/validated by Download Achievement Sets and decoded by the UI raster
//! worker. Neither operation touches the emulator or render thread.
use std::collections::{BTreeSet, VecDeque};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use crate::network::{Failure, Transport};
use crate::storage::Game;

pub(crate) const MAX_BYTES: u64 = 128 * 1024;

pub struct BadgeImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub(crate) fn path(dir: &Path, name: &str) -> Option<PathBuf> {
    // API names are identifiers, never URLs or paths. Keep leading zeroes intact.
    (!name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
    .then(|| dir.join("badges").join(format!("{name}.png")))
}

pub fn load_badge(path: &Path) -> Option<BadgeImage> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    decode(&bytes)
}

fn decode(bytes: &[u8]) -> Option<BadgeImage> {
    if bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits { bytes: 1024 * 1024 });
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info();
    if info.width == 0 || info.height == 0 || info.width > 256 || info.height > 256 {
        return None;
    }
    let mut data = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut data).ok()?;
    let mut rgba = Vec::with_capacity((frame.width * frame.height * 4) as usize);
    for pixel in data[..frame.buffer_size()].chunks_exact(frame.color_type.samples()) {
        match frame.color_type {
            png::ColorType::Rgba => rgba.extend_from_slice(pixel),
            png::ColorType::Rgb => rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]),
            png::ColorType::Grayscale => {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], 255])
            }
            png::ColorType::GrayscaleAlpha => {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]])
            }
            png::ColorType::Indexed => return None,
        }
    }
    Some(BadgeImage {
        width: frame.width,
        height: frame.height,
        rgba,
    })
}

/// slotPlus: the badges still to fetch, filled one game at a time by Download Achievement
/// Sets. A badge already on the card (and readable) is never fetched again.
pub(crate) struct Queue {
    dir: PathBuf,
    names: BTreeSet<String>,
    pending: VecDeque<String>,
}

impl Queue {
    pub fn empty(dir: &Path) -> Self {
        Self {
            dir: dir.into(),
            names: BTreeSet::new(),
            pending: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, game: &Game) {
        for achievement in &game.achievements {
            if achievement.flags != 3 {
                continue;
            }
            let Some(path) = path(&self.dir, &achievement.badge) else {
                continue;
            };
            if !self.names.contains(&achievement.badge) && load_badge(&path).is_none() {
                self.names.insert(achievement.badge.clone());
                self.pending.push_back(achievement.badge.clone());
            }
        }
    }

    #[cfg(test)]
    pub fn pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Fetch the next missing badge straight away; `None` once there are none left. A
    /// network failure keeps the badge for next time; anything else (a bad image, a server
    /// refusal) drops it until a later scan.
    pub fn fetch_next(&mut self, http: &mut impl Transport) -> Option<Result<(), Failure>> {
        let name = self.pending.pop_front()?;
        let target = path(&self.dir, &name)?;
        let result = http.badge(&name).and_then(|bytes| {
            decode(&bytes).ok_or(Failure::Invalid)?;
            std::fs::create_dir_all(target.parent().unwrap()).map_err(|_| Failure::Invalid)?;
            slot_store::atomic_write(&target, &bytes).map_err(|_| Failure::Invalid)
        });
        match result {
            Err(Failure::Network) => self.pending.push_front(name),
            _ => {
                self.names.remove(&name);
            }
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "fetches one public RA badge; no account or award request"]
    fn public_ra_badge_endpoint() {
        let bytes = crate::network::Http::new().badge("25000").unwrap();
        let image = decode(&bytes).expect("RA badge must decode");
        assert!(image.width > 0 && image.height > 0);
        if let Some(path) = std::env::var_os("SLOT_RA_PREVIEW_BADGE") {
            std::fs::write(path, bytes).unwrap();
        }
    }
}
