//! slotPlus music cart: what's in the card's `Music/` folder.
//!
//! Folders, playlists (`.m3u`) and songs are listed one folder at a time, when the player
//! opens it, so a big library costs nothing until it's browsed.

use std::path::{Path, PathBuf};

pub use slot_store::music::{has_music, holds_music, is_song, music_dir, MUSIC_DIR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Folder {
        name: String,
        path: PathBuf,
    },
    /// An `.m3u` or `.m3u8` list of songs.
    Playlist {
        name: String,
        path: PathBuf,
    },
    Song {
        name: String,
        path: PathBuf,
    },
}

impl Entry {
    pub fn name(&self) -> &str {
        match self {
            Entry::Folder { name, .. }
            | Entry::Playlist { name, .. }
            | Entry::Song { name, .. } => name,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Entry::Folder { path, .. }
            | Entry::Playlist { path, .. }
            | Entry::Song { path, .. } => path,
        }
    }
}

pub fn is_playlist(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("m3u") || e.eq_ignore_ascii_case("m3u8"))
}

/// The songs a playlist names that are on the card, in its order. Lines starting with `#`
/// are notes; a path is taken from the playlist's own folder unless it's absolute, with
/// Windows' backslashes allowed.
pub fn playlist_songs(playlist: &Path) -> Vec<PathBuf> {
    let Ok(bytes) = std::fs::read(playlist) else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&bytes);
    let dir = playlist.parent().unwrap_or(Path::new(""));
    text.lines()
        .map(|l| l.trim().trim_start_matches('\u{feff}'))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let l = l.replace('\\', "/");
            let p = Path::new(&l);
            match p.is_absolute() {
                true => p.to_path_buf(),
                false => dir.join(p),
            }
        })
        .filter(|p| is_song(p) && p.is_file())
        .collect()
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

/// One folder: its folders with music in them first, then its playlists, then its songs, each
/// in natural order.
pub fn list(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut folders = Vec::new();
    let mut lists = Vec::new();
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
        } else if is_playlist(&path) {
            lists.push(Entry::Playlist { name, path });
        }
    }
    for v in [&mut folders, &mut lists, &mut songs] {
        v.sort_by(|a, b| natural_cmp(a.name(), b.name()));
    }
    folders.extend(lists);
    folders.extend(songs);
    folders
}

/// The songs in one folder, in order: what plays on after the one picked.
pub fn songs_in(dir: &Path) -> Vec<PathBuf> {
    list(dir)
        .into_iter()
        .filter_map(|e| match e {
            Entry::Song { path, .. } => Some(path),
            Entry::Folder { .. } | Entry::Playlist { .. } => None,
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

    #[test]
    fn a_playlist_lists_the_songs_it_names_that_are_on_the_card() {
        let d = tempfile::tempdir().unwrap();
        let m = d.path().join("Music");
        std::fs::create_dir_all(m.join("Album")).unwrap();
        std::fs::write(m.join("Album/01 One.mp3"), b"").unwrap();
        std::fs::write(m.join("Album/02 Two.flac"), b"").unwrap();
        let list = m.join("Mix.M3U");
        std::fs::write(
            &list,
            "\u{feff}#EXTM3U\n#EXTINF:1,Two\nAlbum\\02 Two.flac\n\nAlbum/01 One.mp3\nGone.mp3\ncover.jpg\n",
        )
        .unwrap();
        assert_eq!(
            playlist_songs(&list),
            [m.join("Album/02 Two.flac"), m.join("Album/01 One.mp3")]
        );
        let names: Vec<_> = list_entries(&m);
        assert_eq!(names, ["folder Album", "playlist Mix"]);
    }

    fn list_entries(dir: &Path) -> Vec<String> {
        list(dir)
            .iter()
            .map(|e| match e {
                Entry::Folder { name, .. } => format!("folder {name}"),
                Entry::Playlist { name, .. } => format!("playlist {name}"),
                Entry::Song { name, .. } => format!("song {name}"),
            })
            .collect()
    }
}
