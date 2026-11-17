use std::io::Read;
use std::path::{Path, PathBuf};

use slot_store::{atomic_write, write_slot_state, Core, Platform, SlotState, StateRing};

pub trait Snapshot {
    fn state(&self) -> Option<Vec<u8>>;
    fn save_ram(&self) -> Option<Vec<u8>>;
    fn thumb(&self) -> Option<Vec<u8>>;
    fn load(&self, state: Vec<u8>);

    fn reset(&self) {}

    fn resume_trusted(&self) -> bool {
        true
    }

    fn save_ram_trusted(&self) -> bool {
        true
    }
}

pub fn flush(
    root: &Path,
    platform: Platform,
    core: Core,
    stem: &str,
    state: Option<&[u8]>,
    sav: Option<&[u8]>,
) -> std::io::Result<()> {
    if let Some(state) = state {
        let ring = StateRing::new(root, platform, core, stem);
        if !matches!(ring.read_resume(), Ok(Some(old)) if old == state) {
            ring.write_resume(state)?;
        }
    }
    if let Some(sav) = sav {
        write_sav(root, platform, stem, sav)?;
    }
    Ok(())
}

pub fn eject(
    root: &Path,
    platform: Platform,
    core: Core,
    stem: &str,
    state: Option<&[u8]>,
    sav: Option<&[u8]>,
    slot: &SlotState,
) -> std::io::Result<()> {
    flush(root, platform, core, stem, state, sav)?;
    let mut slot = slot.clone();
    slot.cart = None;
    slot.cart_platform = None;
    write_slot_state(root, &slot)
}

pub fn write_sav(root: &Path, platform: Platform, stem: &str, sav: &[u8]) -> std::io::Result<bool> {
    let path = sav_path(root, platform, stem);
    if let Ok(Some(old)) = read_sav(root, platform, stem) {
        if old == sav {
            return Ok(false);
        }
        // slotPlus: keep the save being replaced, so a bad write or a bad day can be undone.
        if let Err(e) = back_up(root, platform, stem, &old, unix_now()) {
            eprintln!("slot: save backup for {stem}: {e}");
        }
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    atomic_write(&path, sav)?;
    Ok(true)
}

pub fn read_sav(root: &Path, platform: Platform, stem: &str) -> std::io::Result<Option<Vec<u8>>> {
    let srm = crate::root::saves_dir(root)
        .join(platform.dir_name())
        .join(format!("{stem}.srm"));
    for path in [sav_path(root, platform, stem), srm] {
        match std::fs::read(&path) {
            Ok(bytes) if bytes.starts_with(RZIP_MAGIC) => return unrzip(&bytes).map(Some),
            Ok(bytes) => return Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(None)
}

const RZIP_MAGIC: &[u8] = b"#RZIPv\x01#";

fn unrzip(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "bad rzip save");
    let total = bytes.get(12..20).ok_or_else(bad)?;
    let total = u64::from_le_bytes(total.try_into().map_err(|_| bad())?);
    let mut rest = &bytes[20..];
    let mut out = Vec::new();
    while let Some((len, body)) = rest.split_first_chunk::<4>() {
        let len = u32::from_le_bytes(*len) as usize;
        let chunk = body.get(..len).ok_or_else(bad)?;
        flate2::read::ZlibDecoder::new(chunk).read_to_end(&mut out)?;
        rest = &body[len..];
    }
    if out.len() as u64 == total {
        Ok(out)
    } else {
        Err(bad())
    }
}

pub fn read_resume(
    root: &Path,
    platform: Platform,
    core: Core,
    stem: &str,
) -> std::io::Result<Option<Vec<u8>>> {
    StateRing::new(root, platform, core, stem).read_resume()
}

/// slotPlus: the card holds a save for this game, as a `.sav` or a RetroArch `.srm`.
pub fn has_save(root: &Path, platform: Platform, stem: &str) -> bool {
    let dir = crate::root::saves_dir(root).join(platform.dir_name());
    ["sav", "srm"]
        .iter()
        .any(|ext| dir.join(format!("{stem}.{ext}")).exists())
}

fn sav_path(root: &Path, platform: Platform, stem: &str) -> PathBuf {
    crate::root::saves_dir(root)
        .join(platform.dir_name())
        .join(format!("{stem}.sav"))
}

/// slotPlus: copies of a game's earlier saves, kept in `Saves/Backups/<system>/`.
pub const BACKUPS_DIR: &str = "Backups";
/// How many earlier saves are kept for each game.
pub const BACKUPS_KEPT: usize = 3;
/// A save replaced sooner than this after the newest backup isn't backed up again, so a
/// long session writing its save every few minutes doesn't push older backups out.
pub const BACKUP_EVERY_SECS: u64 = 60 * 60;

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn backups_dir(root: &Path, platform: Platform) -> PathBuf {
    crate::root::saves_dir(root)
        .join(BACKUPS_DIR)
        .join(platform.dir_name())
}

/// A game's backups, newest first, with the time each was taken.
pub fn backups(root: &Path, platform: Platform, stem: &str) -> Vec<(u64, PathBuf)> {
    let dir = backups_dir(root, platform);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let prefix = format!("{stem}.");
    let mut found: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let stamp = name.strip_prefix(&prefix)?.strip_suffix(".sav")?;
            if stamp.is_empty() || !stamp.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            Some((stamp.parse().ok()?, e.path()))
        })
        .collect();
    found.sort_by_key(|b| std::cmp::Reverse(b.0));
    found
}

/// Keep `old`, the save about to be replaced, unless the newest backup is recent.
pub fn back_up(
    root: &Path,
    platform: Platform,
    stem: &str,
    old: &[u8],
    now: u64,
) -> std::io::Result<bool> {
    let kept = backups(root, platform, stem);
    if let Some((newest, path)) = kept.first() {
        let recent = now >= *newest && now - newest < BACKUP_EVERY_SECS;
        let same = std::fs::read(path).is_ok_and(|b| b == old);
        if recent || same {
            return Ok(false);
        }
    }
    let dir = backups_dir(root, platform);
    std::fs::create_dir_all(&dir)?;
    atomic_write(&dir.join(format!("{stem}.{now}.sav")), old)?;
    for (_, path) in backups(root, platform, stem).into_iter().skip(BACKUPS_KEPT) {
        let _ = std::fs::remove_file(path);
    }
    Ok(true)
}
