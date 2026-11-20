//! slotPlus: what slotPlus reads of PokeDaisy's own settings.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Where the PokeDaisy cores keep their settings: `pokedaisy/settings.ini` in the save
/// directory slotPlus gives them, `Saves/`. The companion's SETTINGS writes it as they change.
pub fn settings_file(root: &Path) -> PathBuf {
    crate::root::saves_dir(root)
        .join("pokedaisy")
        .join("settings.ini")
}

/// Whether the settings set FLIGHT AREA to ANYWHERE: `flightanywhere=1` (or `flyanywhere=1`,
/// as older builds wrote it). PokeDaisy's own apps pause RetroAchievements then, since flying
/// ahead could reach a set's places early.
pub fn flies_anywhere(settings: &str) -> bool {
    settings.lines().any(|line| {
        let Some((key, value)) = line.split_once('=') else {
            return false;
        };
        matches!(key.trim(), "flightanywhere" | "flyanywhere") && value.trim() == "1"
    })
}

/// A check of the settings file that reads it again only when it has changed.
pub fn flying_anywhere(root: &Path) -> impl FnMut() -> bool + Send {
    let file = settings_file(root);
    let mut seen: Option<SystemTime> = None;
    let mut anywhere = false;
    move || {
        let modified = std::fs::metadata(&file).and_then(|m| m.modified()).ok();
        if modified != seen {
            seen = modified;
            anywhere = std::fs::read_to_string(&file).is_ok_and(|text| flies_anywhere(&text));
        }
        anywhere
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flight_area_anywhere_is_read_from_either_key() {
        assert!(flies_anywhere("flyclassic=0\nflightanywhere=1\nflyon=0\n"));
        assert!(flies_anywhere("flyanywhere=1"));
        assert!(!flies_anywhere("flyclassic=0\nflightanywhere=0\nflyon=1\n"));
        assert!(!flies_anywhere(""));
    }

    #[test]
    fn the_check_follows_the_file_as_the_companion_rewrites_it() {
        let d = tempfile::tempdir().unwrap();
        let mut anywhere = flying_anywhere(d.path());
        assert!(!anywhere(), "no settings yet");
        let file = settings_file(d.path());
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "flightanywhere=1\n").unwrap();
        assert!(anywhere());
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&file, "flightanywhere=0\n").unwrap();
        assert!(!anywhere());
    }
}
