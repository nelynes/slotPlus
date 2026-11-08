//! slotPlus: the games played since the last sync. A game runs with Wi-Fi off, so
//! RetroAchievements hears about it from the shelf afterwards (`startsession`), and it shows
//! as recently played on the player's profile. Kept on the card, so a game played and then
//! switched off still gets reported.
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Played {
    pub game: u32,
    pub hash: String,
}

const FILE: &str = "played.json";

pub(crate) fn queued(dir: &Path) -> Vec<Played> {
    storage::read(&dir.join(FILE)).unwrap_or_default()
}

/// Queue a game. A game already waiting is moved to the end, not queued twice.
pub(crate) fn push(dir: &Path, played: Played) {
    let mut queue = queued(dir);
    queue.retain(|p| p != &played);
    queue.push(played);
    let _ = storage::write(&dir.join(FILE), &queue);
}

/// Drop a game once RetroAchievements has heard about it (or refused it).
pub(crate) fn done(dir: &Path, played: &Played) {
    let mut queue = queued(dir);
    let before = queue.len();
    queue.retain(|p| p != played);
    if queue.len() != before {
        let _ = storage::write(&dir.join(FILE), &queue);
    }
}
