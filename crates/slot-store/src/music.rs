//! slotPlus music cart: the one cart on the Game Boy Advance shelf that isn't a game. It's
//! there whenever the card's `Music/` folder has songs in it, and its "ROM" is that folder.

use std::path::{Path, PathBuf};

use crate::platform::Platform;
use crate::scan::Cart;

pub const MUSIC_DIR: &str = "Music";
/// The cart's name on the card: its save, play time and label go by it.
pub const MUSIC_STEM: &str = "slotPlus Music";
pub const MUSIC_TITLE: &str = "Music";

/// The kinds of file the cart plays.
pub const MUSIC_EXTENSIONS: [&str; 6] = ["mp3", "ogg", "oga", "flac", "wav", "m4a"];

pub fn music_dir(root: &Path) -> PathBuf {
    root.join(MUSIC_DIR)
}

pub fn is_song(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| MUSIC_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

fn hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_none_or(|n| n.starts_with('.'))
}

/// Whether a folder holds a song anywhere inside it, looking `depth` folders down. It stops
/// at the first song it finds.
pub fn holds_music(dir: &Path, depth: usize) -> bool {
    let Ok(read) = std::fs::read_dir(dir) else {
        return false;
    };
    let mut folders = Vec::new();
    for entry in read.flatten() {
        let path = entry.path();
        if hidden(&path) {
            continue;
        }
        if path.is_dir() {
            folders.push(path);
        } else if is_song(&path) {
            return true;
        }
    }
    depth > 0 && folders.iter().any(|f| holds_music(f, depth - 1))
}

/// Whether the card has music for the cart to play.
pub fn has_music(root: &Path) -> bool {
    holds_music(&music_dir(root), 3)
}

/// The music cart, with its label if the card has one in `Labels/GBA/`.
pub fn music_cart(root: &Path) -> Cart {
    let label = root
        .join("Labels")
        .join(Platform::Gba.dir_name())
        .join(format!("{MUSIC_STEM}.png"));
    Cart {
        platform: Platform::Gba,
        stem: MUSIC_STEM.to_string(),
        rom: music_dir(root),
        label: label.is_file().then_some(label),
        title: MUSIC_TITLE.to_string(),
        code: String::new(),
        shell: None,
        tags: Vec::new(),
    }
}

pub fn is_music(cart: &Cart) -> bool {
    cart.platform == Platform::Gba && cart.stem == MUSIC_STEM && cart.rom.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cart_is_there_only_with_songs_on_the_card() {
        let d = tempfile::tempdir().unwrap();
        assert!(!has_music(d.path()));
        let deep = d.path().join("Music/Artist/Album/Disc 1");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("cover.jpg"), b"").unwrap();
        assert!(!has_music(d.path()), "a picture isn't music");
        std::fs::write(deep.join("01.FLAC"), b"").unwrap();
        assert!(has_music(d.path()));
        let cart = music_cart(d.path());
        assert!(is_music(&cart));
        assert_eq!(cart.title, "Music");
    }
}
