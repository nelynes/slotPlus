//! slotPlus music cart: what's in the card's `Music/` folder.
//!
//! Folders and songs are listed one folder at a time, when the player opens it, so a big
//! library costs nothing until it's browsed.

use std::path::{Path, PathBuf};

pub use slot_store::music::{has_music, holds_music, is_song, music_dir, MUSIC_DIR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Folder { name: String, path: PathBuf },
    Song { name: String, path: PathBuf },
}

impl Entry {
    pub fn name(&self) -> &str {
        match self {
            Entry::Folder { name, .. } | Entry::Song { name, .. } => name,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Entry::Folder { path, .. } | Entry::Song { path, .. } => path,
        }
    }
}

fn hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_none_or(|n| n.starts_with('.'))
}

/// Sorts the way people number things: "2 Song" before "10 Song", case ignored.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let n = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut s = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        s.push(c);
                        it.next();
                    }
                    s
                };
                let (na, nb) = (n(&mut a), n(&mut b));
                let (ta, tb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                let ord = x.to_lowercase().cmp(y.to_lowercase());
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
                a.next();
                b.next();
            }
        }
    }
}

/// One folder: its folders with music in them first, then its songs, each in natural order.
pub fn list(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut folders = Vec::new();
    let mut songs = Vec::new();
    for entry in read.flatten() {
        let path = entry.path();
        if hidden(&path) {
            continue;
        }
        let name = match path.is_dir() {
            true => path.file_name(),
            false => path.file_stem(),
        }
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
        if path.is_dir() {
            if holds_music(&path, 3) {
                folders.push(Entry::Folder { name, path });
            }
        } else if is_song(&path) {
            songs.push(Entry::Song { name, path });
        }
    }
    folders.sort_by(|a, b| natural_cmp(a.name(), b.name()));
    songs.sort_by(|a, b| natural_cmp(a.name(), b.name()));
    folders.extend(songs);
    folders
}

/// The songs in one folder, in order: what plays on after the one picked.
pub fn songs_in(dir: &Path) -> Vec<PathBuf> {
    list(dir)
        .into_iter()
        .filter_map(|e| match e {
            Entry::Song { path, .. } => Some(path),
            Entry::Folder { .. } => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_sort_as_numbers() {
        let mut v = vec!["10 Ten", "2 Two", "1 One", "b", "A"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(v, ["1 One", "2 Two", "10 Ten", "A", "b"]);
    }

    #[test]
    fn folders_with_music_come_first_then_songs() {
        let d = tempfile::tempdir().unwrap();
        let m = d.path().join("Music");
        std::fs::create_dir_all(m.join("Album/Disc 1")).unwrap();
        std::fs::create_dir_all(m.join("Empty")).unwrap();
        std::fs::write(m.join("Album/Disc 1/01 Song.mp3"), b"").unwrap();
        std::fs::write(m.join("10 Later.flac"), b"").unwrap();
        std::fs::write(m.join("9 Sooner.OGG"), b"").unwrap();
        std::fs::write(m.join("cover.jpg"), b"").unwrap();
        std::fs::write(m.join(".hidden.mp3"), b"").unwrap();
        let names: Vec<_> = list(&m).iter().map(|e| e.name().to_string()).collect();
        assert_eq!(names, ["Album", "9 Sooner", "10 Later"]);
        assert!(has_music(d.path()));
        assert_eq!(songs_in(&m).len(), 2);
        assert!(!has_music(&d.path().join("Elsewhere")));
    }
}
