//! Background library preparation. A fingerprint avoids re-reading unchanged ROMs at boot.
//!
//! slotPlus: the index is kept in memory and written to the card in batches (`flush_due`,
//! `flush`) rather than after every ROM, and a background hash reads gently and pauses
//! whenever slot is busy (`content_hash_gently`), so preparing a library never competes with
//! booting or loading a game for the card.
use std::collections::{BTreeMap, VecDeque};
use std::io::Read;
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

/// Whether slot is busy, so a background read should wait.
pub(crate) trait Gate {
    fn busy(&self) -> bool;
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
    hash_with(dir, path, None)
}

/// A ROM's hash for the library, read gently and paused while `gate` says slot is busy.
pub(crate) fn hash_gently(dir: &Path, path: &Path, gate: &dyn Gate) -> Result<String, String> {
    hash_with(dir, path, Some(gate))
}

fn hash_with(dir: &Path, path: &Path, gate: Option<&dyn Gate>) -> Result<String, String> {
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
    let hash = match gate {
        Some(gate) => content_hash_gently(path, gate)?,
        None => content_hash(path)?,
    };
    with_index(dir, |index, changed| {
        index.insert(
            path.into(),
            Entry {
                size,
                modified_ns,
                hash: hash.clone(),
                checked_at: 0,
                badge_version: 0,
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

pub(crate) fn pending(root: &Path, dir: &Path, now: u64) -> VecDeque<PathBuf> {
    let index = with_index(dir, |index, _| index.clone());
    let mut paths: Vec<_> = Platform::ALL
        .into_iter()
        .flat_map(|platform| {
            std::fs::read_dir(root.join("Games").join(platform.dir_name()))
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(move |path| {
                    path.is_file()
                        && platform.accepts(path)
                        && !path
                            .file_name()
                            .is_some_and(|name| name.to_string_lossy().starts_with('.'))
                })
        })
        .filter(|path| {
            let Some(entry) = index.get(path) else {
                return true;
            };
            // Refresh definitions and server unlocks weekly, and reconsider unknown ROMs.
            fingerprint(path).ok() != Some((entry.size, entry.modified_ns))
                || entry.checked_at == 0
                || entry.badge_version < 2
                || now.saturating_sub(entry.checked_at) >= 7 * 86400
                || !dir.join(format!("{}.json", entry.hash)).exists()
        })
        .collect();
    paths.sort();
    paths.into()
}

pub(crate) fn checked(dir: &Path, path: &Path, now: u64) {
    with_index(dir, |index, changed| {
        if let Some(entry) = index.get_mut(path) {
            entry.checked_at = now;
            entry.badge_version = 2;
            *changed = true;
        }
    });
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
    loop {
        let n = file.read(&mut buffer).map_err(|_| "Cannot read ROM")?;
        if n == 0 {
            break;
        }
        md5.consume(&buffer[..n]);
    }
    Ok(format!("{:x}", md5.compute()))
}

/// Background reads stay under this, so the card has room for whatever slot reads next.
const GENTLE_BYTES_PER_SEC: f64 = 6.0 * 1024.0 * 1024.0;
const GENTLE_CHUNK: usize = 256 * 1024;

/// `content_hash`, read a chunk at a time at no more than `GENTLE_BYTES_PER_SEC`, waiting
/// whenever `gate` says slot is busy.
pub(crate) fn content_hash_gently(path: &Path, gate: &dyn Gate) -> Result<String, String> {
    let size = path
        .metadata()
        .map_err(|_| "Cannot read ROM metadata")?
        .len();
    if size == 0 || size > 64 * 1024 * 1024 {
        return Err("Unsupported ROM size".into());
    }
    let mut file = std::fs::File::open(path).map_err(|_| "Cannot open ROM")?;
    let mut buffer = vec![0; GENTLE_CHUNK];
    let mut md5 = md5::Context::new();
    loop {
        while gate.busy() {
            std::thread::sleep(Duration::from_millis(200));
        }
        let began = Instant::now();
        let n = file.read(&mut buffer).map_err(|_| "Cannot read ROM")?;
        if n == 0 {
            break;
        }
        md5.consume(&buffer[..n]);
        let budget = Duration::from_secs_f64(n as f64 / GENTLE_BYTES_PER_SEC);
        if let Some(rest) = budget.checked_sub(began.elapsed()) {
            std::thread::sleep(rest);
        }
    }
    Ok(format!("{:x}", md5.compute()))
}
