//! Which ROMs on the card can earn achievements with no network: their definitions are cached
//! for the configured account, and they hold at least one core achievement.
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::de::IgnoredAny;
use serde::Deserialize;

use crate::{library, storage};

#[derive(Deserialize)]
struct Game {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "ConsoleID")]
    console: u32,
    #[serde(rename = "Achievements")]
    achievements: Vec<Achievement>,
}

#[derive(Deserialize)]
struct Achievement {
    #[serde(rename = "Flags")]
    flags: u32,
}

/// Remembers each cache file's verdict until the file changes, so a rescan after every
/// prepared ROM reads only the file that was just written. Keyed by path, so another
/// account's cache for the same ROM is read afresh.
#[derive(Default)]
pub struct Readiness {
    verdicts: HashMap<PathBuf, (SystemTime, bool)>,
}

impl Readiness {
    /// Call on an I/O worker. Reads the library index and never hashes a ROM: a file the
    /// index has not fingerprinted yet is not ready until the achievement worker reaches it.
    pub fn scan(&mut self, root: &Path) -> HashSet<PathBuf> {
        let Some(dir) = account_dir(root) else {
            return HashSet::new();
        };
        library::indexed(&dir)
            .into_iter()
            .filter(|(_, hash)| self.ready(&dir, hash))
            .map(|(path, _)| path)
            .collect()
    }

    fn ready(&mut self, dir: &Path, hash: &str) -> bool {
        let path = dir.join(format!("{hash}.json"));
        let Some(modified) = path.metadata().and_then(|m| m.modified()).ok() else {
            return false;
        };
        if let Some((seen, ready)) = self.verdicts.get(&path) {
            if *seen == modified {
                return *ready;
            }
        }
        let ready = storage::read::<(Game, IgnoredAny)>(&path).is_ok_and(|(game, _)| {
            game.id != 0
                && matches!(game.console, 4..=6)
                && game.achievements.iter().any(|a| a.flags == 3)
        });
        self.verdicts.insert(path, (modified, ready));
        ready
    }
}

fn account_dir(root: &Path) -> Option<PathBuf> {
    let config = storage::Config::read(&root.join("Config/retroachievements.toml")).ok()?;
    if !config.enabled || config.username.trim().is_empty() {
        return None;
    }
    let account = format!("{:x}", md5::compute(config.username.to_lowercase()));
    Some(root.join("Saves/RetroAchievements").join(account))
}
