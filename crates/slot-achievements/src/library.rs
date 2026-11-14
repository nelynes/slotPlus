//! Background library preparation. A fingerprint avoids re-reading unchanged ROMs at boot.
//!
//! slotPlus: the index is kept in memory and written to the card in batches (`flush_due`,
//! `flush`) rather than after every ROM. Sets are downloaded only when the player asks, from
//! Settings → RetroAchievements → Download Achievement Sets (`crate::scan`), which records in
//! the index whether each ROM has a set, so the shelf can show a trophy without reading any
//! set itself.
use std::collections::{BTreeMap, HashSet};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use slot_store::Platform;

use crate::storage;

#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    size: u64,
    modified_ns: u128,
    hash: String,
    #[serde(default)]
    checked_at: u64,
    #[serde(default)]
    badge_version: u8,
    /// slotPlus: the last scan found an achievement set for this ROM.
    #[serde(default)]
    has_set: bool,
}

type Index = BTreeMap<PathBuf, Entry>;

/// The index as last read or changed, for the one card it belongs to, and how much of it
/// has not reached the card yet.
struct Cached {
    path: PathBuf,
    index: Index,
    dirty: usize,
    since: Instant,
}

static CACHE: Mutex<Option<Cached>> = Mutex::new(None);

/// Changes written together: after this many, or this long after the first unsaved one.
const FLUSH_AFTER: usize = 16;
const FLUSH_WITHIN: Duration = Duration::from_secs(30);

fn with_index<T>(dir: &Path, f: impl FnOnce(&mut Index, &mut bool) -> T) -> T {
    let path = dir.join("library.json");
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if cache.as_ref().is_none_or(|c| c.path != path) {
        if let Some(old) = cache.take() {
            write_out(&old);
        }
        *cache = Some(Cached {
            index: storage::read(&path).unwrap_or_default(),
            path,
            dirty: 0,
            since: Instant::now(),
        });
    }
    let cached = cache.as_mut().expect("just filled");
    let mut changed = false;
    let out = f(&mut cached.index, &mut changed);
    if changed {
        if cached.dirty == 0 {
            cached.since = Instant::now();
        }
        cached.dirty += 1;
    }
    out
}

fn write_out(cached: &Cached) {
    if cached.dirty > 0 {
        // Fingerprinting is a convenience cache; a failure here must not stop identification.
        let _ = storage::write(&cached.path, &cached.index);
    }
}

/// Write the index if enough has changed, or for long enough. Cheap to call every pass.
pub(crate) fn flush_due() {
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(c) = cache.as_mut() {
        if c.dirty >= FLUSH_AFTER || (c.dirty > 0 && c.since.elapsed() >= FLUSH_WITHIN) {
            write_out(c);
            c.dirty = 0;
        }
    }
}

/// Write whatever has changed now: at power off.
pub(crate) fn flush() {
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(c) = cache.as_mut() {
        write_out(c);
        c.dirty = 0;
    }
}

fn fingerprint(path: &Path) -> Result<(u64, u128), String> {
    let metadata = path.metadata().map_err(|_| "Cannot read ROM metadata")?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    Ok((metadata.len(), modified))
}

pub(crate) fn hash(dir: &Path, path: &Path) -> Result<String, String> {
    let (size, modified_ns) = fingerprint(path)?;
    let known = with_index(dir, |index, _| {
        index
            .get(path)
            .filter(|e| e.size == size && e.modified_ns == modified_ns && modified_ns != 0)
            .map(|e| e.hash.clone())
    });
    if let Some(hash) = known {
        return Ok(hash);
    }
    let hash = content_hash(path)?;
    with_index(dir, |index, changed| {
        index.insert(
            path.into(),
            Entry {
                size,
                modified_ns,
                hash: hash.clone(),
                checked_at: 0,
                badge_version: 0,
                has_set: false,
            },
        );
        *changed = true;
    });
    Ok(hash)
}

/// ROMs whose fingerprinted hash still matches the file on the card. Reads only.
pub(crate) fn indexed(dir: &Path) -> Vec<(PathBuf, String)> {
    let index = with_index(dir, |index, _| index.clone());
    index
        .into_iter()
        .filter(|(path, e)| {
            e.modified_ns != 0 && fingerprint(path).ok() == Some((e.size, e.modified_ns))
        })
        .map(|(path, e)| (path, e.hash))
        .collect()
}

/// The systems slot tracks achievements for, in the order a scan visits them.
pub(crate) const SYSTEMS: [Platform; 8] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Ws,
    Platform::Lynx,
    Platform::Gg,
    Platform::Ngp,
    Platform::Tg16,
];

/// slotPlus: a system's RetroAchievements console ID (rcheevos rc_consoles.h). The
/// WonderSwan and WonderSwan Color share one, as do the NeoGeo Pocket and Pocket Color.
pub(crate) fn console_of(platform: Platform) -> u32 {
    match platform {
        Platform::Gb => 4,
        Platform::Gba | Platform::Gbav => 5,
        Platform::Gbc => 6,
        Platform::Tg16 => 8,
        Platform::Lynx => 13,
        Platform::Ngp => 14,
        Platform::Gg => 15,
        Platform::Nds => 18,
        Platform::Ws => 53,
    }
}

/// Each tracked system that has games on the card, with its ROMs in name order. Hidden
/// files and anything the shelf wouldn't show are left out.
pub(crate) fn roms(root: &Path) -> Vec<(Platform, Vec<PathBuf>)> {
    SYSTEMS
        .into_iter()
        .filter_map(|platform| {
            let mut paths: Vec<PathBuf> =
                std::fs::read_dir(root.join("Games").join(platform.dir_name()))
                    .into_iter()
                    .flatten()
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|path| {
                        path.is_file()
                            && platform.accepts(path)
                            && !path
                                .file_name()
                                .is_some_and(|name| name.to_string_lossy().starts_with('.'))
                    })
                    .collect();
            paths.sort();
            (!paths.is_empty()).then_some((platform, paths))
        })
        .collect()
}

/// Record what a scan found for a ROM already hashed into the index.
pub(crate) fn checked(dir: &Path, path: &Path, now: u64, has_set: bool) {
    with_index(dir, |index, changed| {
        if let Some(entry) = index.get_mut(path) {
            entry.checked_at = now;
            entry.badge_version = SET_RULES;
            entry.has_set = has_set;
            *changed = true;
        }
    });
}

/// slotPlus: which rules an entry's `has_set` was judged by. 3: RetroAchievements' notices
/// and stand-in games don't count.
const SET_RULES: u8 = 3;

/// ROMs whose last scan found a set, still unchanged on the card, with the set still cached.
/// A verdict from older rules is judged again from the cached set, once.
pub(crate) fn with_sets(dir: &Path) -> HashSet<PathBuf> {
    let (index, rejudged) = with_index(dir, |index, changed| {
        let mut rejudged = false;
        for entry in index
            .values_mut()
            .filter(|e| e.has_set && e.badge_version < SET_RULES)
        {
            entry.has_set = crate::storage::read::<(
                crate::storage::Game,
                std::collections::BTreeSet<u32>,
            )>(&dir.join(format!("{}.json", entry.hash)))
            .is_ok_and(|(game, _)| crate::scan::has_set(&game));
            entry.badge_version = SET_RULES;
            *changed = true;
            rejudged = true;
        }
        (index.clone(), rejudged)
    });
    if rejudged {
        flush();
    }
    index
        .into_iter()
        .filter(|(path, e)| {
            e.has_set
                && e.modified_ns != 0
                && fingerprint(path).ok() == Some((e.size, e.modified_ns))
                && dir.join(format!("{}.json", e.hash)).exists()
        })
        .map(|(path, _)| path)
        .collect()
}

/// slotPlus: a PC Engine HuCard with a 512-byte copier's header, told by its size as
/// rcheevos does: `size & 512`.
fn pce_header(path: &Path, size: u64) -> bool {
    size & 512 != 0
        && path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pce"))
}

pub(crate) fn content_hash(path: &Path) -> Result<String, String> {
    let size = path
        .metadata()
        .map_err(|_| "Cannot read ROM metadata")?
        .len();
    if size == 0 || size > 64 * 1024 * 1024 {
        return Err("Unsupported ROM size".into());
    }
    let mut file = std::fs::File::open(path).map_err(|_| "Cannot open ROM")?;
    let mut buffer = [0; 64 * 1024];
    let mut md5 = md5::Context::new();
    // slotPlus: RetroAchievements hashes a Lynx game without its 64-byte "LYNX" header
    // (rcheevos rc_hash), which .lnx files carry and .lyx ones don't.
    let mut head = [0; 64];
    let read = file.read(&mut head).map_err(|_| "Cannot read ROM")?;
    if pce_header(path, size) {
        // A HuCard dump 512 bytes past a multiple of 8 KB carries a copier's header, which
        // rcheevos rc_hash_pce leaves out (as the core does).
        file.seek(std::io::SeekFrom::Start(512)).map_err(|_| "Cannot read ROM")?;
    } else if !(read == 64 && head.starts_with(b"LYNX\0")) {
        md5.consume(&head[..read]);
    }
    loop {
        let n = file.read(&mut buffer).map_err(|_| "Cannot read ROM")?;
        if n == 0 {
            break;
        }
        md5.consume(&buffer[..n]);
    }
    Ok(format!("{:x}", md5.compute()))
}

/// Test hooks: an entry's trophy verdict and the rules it was judged by.
#[cfg(test)]
pub(crate) fn set_verdict(dir: &Path, path: &Path, has_set: bool, rules: u8) {
    with_index(dir, |index, changed| {
        let entry = index.get_mut(path).expect("indexed");
        entry.has_set = has_set;
        entry.badge_version = rules;
        *changed = true;
    });
}

#[cfg(test)]
pub(crate) fn verdict(dir: &Path, path: &Path) -> Option<(bool, u8)> {
    with_index(dir, |index, _| index.get(path).map(|e| (e.has_set, e.badge_version)))
}
