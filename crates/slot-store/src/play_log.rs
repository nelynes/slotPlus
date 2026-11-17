//! slotPlus: what the card remembers about each game it has seen: how long it has been
//! played, when it was last played, when it first sat on a shelf, and when its cart was last
//! cleaned. One small text file, `Saves/play_log.txt`, written only when a game stops or a
//! cart is cleaned, and read once when the shelf needs it, never at boot.
//!
//! Each line is a game: `<system>/<stem>`, then tab-separated `key=value` fields, so a
//! later version can add fields and an older one keeps what it doesn't know.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{atomic_write, Platform};

pub const PLAY_LOG_FILE: &str = "play_log.txt";

/// One game's record. Times are Unix seconds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayRecord {
    /// Seconds spent in the game, paused time not counted.
    pub seconds: u64,
    /// When the game was last played, if ever.
    pub last: Option<u64>,
    /// When the card first saw the game on a shelf, so dust starts from then.
    pub seen: Option<u64>,
    /// When the cart was last cleaned.
    pub cleaned: Option<u64>,
    /// `seconds` when the cart was last cleaned, so wear builds again only with more play.
    pub worn_from: u64,
    /// Fields this version doesn't know, kept as they were.
    other: Vec<(String, String)>,
}

impl PlayRecord {
    /// The game has been played on this card.
    pub fn played(&self) -> bool {
        self.seconds > 0 || self.last.is_some()
    }

    /// Since when the cart has sat untouched: the last of being played, cleaned or first seen.
    pub fn idle_since(&self) -> Option<u64> {
        [self.last, self.cleaned, self.seen]
            .into_iter()
            .flatten()
            .max()
    }

    /// Seconds of play since the cart was last cleaned.
    pub fn worn_seconds(&self) -> u64 {
        self.seconds.saturating_sub(self.worn_from)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayLog {
    games: HashMap<String, PlayRecord>,
}

pub fn play_log_path(root: &Path) -> PathBuf {
    root.join("Saves").join(PLAY_LOG_FILE)
}

fn key(platform: Platform, stem: &str) -> String {
    format!("{}/{stem}", platform.hardware().dir_name())
}

impl PlayLog {
    pub fn get(&self, platform: Platform, stem: &str) -> Option<&PlayRecord> {
        self.games.get(&key(platform, stem))
    }

    pub fn record(&self, platform: Platform, stem: &str) -> PlayRecord {
        self.get(platform, stem).cloned().unwrap_or_default()
    }

    fn entry(&mut self, platform: Platform, stem: &str) -> &mut PlayRecord {
        self.games.entry(key(platform, stem)).or_default()
    }

    /// Add a stretch of play that ended at `now`.
    pub fn add_play(&mut self, platform: Platform, stem: &str, seconds: u64, now: u64) {
        let r = self.entry(platform, stem);
        r.seconds += seconds;
        r.last = Some(now);
        r.seen.get_or_insert(now);
    }

    /// Note games the card hasn't seen before. True when any was new.
    pub fn see<'a>(
        &mut self,
        games: impl IntoIterator<Item = (Platform, &'a str)>,
        now: u64,
    ) -> bool {
        let mut new = false;
        for (platform, stem) in games {
            let r = self.entry(platform, stem);
            if r.seen.is_none() {
                r.seen = Some(now);
                new = true;
            }
        }
        new
    }

    /// Wipe a cart: its dust goes, and so does the wear of the play so far.
    pub fn clean(&mut self, platform: Platform, stem: &str, now: u64) {
        self.clean_only(platform, stem, now, true, true);
    }

    /// slotPlus: wipe a cart of its `dust`, its `wear`, or both: the dust starts again from
    /// `now`, the wear from the play so far.
    pub fn clean_only(&mut self, platform: Platform, stem: &str, now: u64, dust: bool, wear: bool) {
        let r = self.entry(platform, stem);
        if dust {
            r.cleaned = Some(now);
            r.seen.get_or_insert(now);
        }
        if wear {
            r.worn_from = r.seconds;
        }
    }

    pub fn parse(text: &str) -> PlayLog {
        let mut games = HashMap::new();
        for line in text.lines() {
            let mut fields = line.split('\t');
            let Some(name) = fields.next().filter(|n| n.contains('/')) else {
                continue;
            };
            let mut r = PlayRecord::default();
            for field in fields {
                let Some((k, v)) = field.split_once('=') else {
                    continue;
                };
                let n = v.parse::<u64>().ok();
                match (k, n) {
                    ("seconds", Some(n)) => r.seconds = n,
                    ("last", Some(n)) => r.last = Some(n),
                    ("seen", Some(n)) => r.seen = Some(n),
                    ("cleaned", Some(n)) => r.cleaned = Some(n),
                    ("worn_from", Some(n)) => r.worn_from = n,
                    _ => r.other.push((k.to_string(), v.to_string())),
                }
            }
            games.insert(name.to_string(), r);
        }
        PlayLog { games }
    }

    pub fn text(&self) -> String {
        let mut names: Vec<&String> = self.games.keys().collect();
        names.sort();
        let mut out = String::new();
        for name in names {
            let r = &self.games[name];
            out.push_str(name);
            out.push_str(&format!("\tseconds={}", r.seconds));
            for (k, v) in [("last", r.last), ("seen", r.seen), ("cleaned", r.cleaned)] {
                if let Some(v) = v {
                    out.push_str(&format!("\t{k}={v}"));
                }
            }
            if r.worn_from > 0 {
                out.push_str(&format!("\tworn_from={}", r.worn_from));
            }
            for (k, v) in &r.other {
                out.push_str(&format!("\t{k}={v}"));
            }
            out.push('\n');
        }
        out
    }
}

/// The card's play log; empty when there is none yet or it can't be read.
pub fn read_play_log(root: &Path) -> PlayLog {
    std::fs::read_to_string(play_log_path(root))
        .map(|t| PlayLog::parse(&t))
        .unwrap_or_default()
}

pub fn write_play_log(root: &Path, log: &PlayLog) -> std::io::Result<()> {
    let path = play_log_path(root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    atomic_write(&path, log.text().as_bytes())
}

/// Read the log, change it and write it back.
pub fn update_play_log(root: &Path, change: impl FnOnce(&mut PlayLog)) -> std::io::Result<PlayLog> {
    let mut log = read_play_log(root);
    change(&mut log);
    write_play_log(root, &log)?;
    Ok(log)
}

/// "12 h 05 m", "45 m", or "Under a minute".
pub fn play_time_text(seconds: u64) -> String {
    let minutes = seconds / 60;
    match (minutes / 60, minutes % 60) {
        (0, 0) => "Under a minute".to_string(),
        (0, m) => format!("{m} m"),
        (h, m) => format!("{h} h {m:02} m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_log_reads_back_what_it_wrote_and_keeps_fields_it_doesnt_know() {
        let mut log = PlayLog::parse("GBA/Golden Sun\tseconds=60\tlast=5\tsparkle=yes\n");
        log.add_play(Platform::Gba, "Golden Sun", 30, 9);
        log.see([(Platform::Ws, "Gunpey")], 7);
        let again = PlayLog::parse(&log.text());
        assert_eq!(again, log);
        let gs = again.record(Platform::Gba, "Golden Sun");
        assert_eq!((gs.seconds, gs.last, gs.seen), (90, Some(9), Some(9)));
        assert!(log.text().contains("sparkle=yes"));
    }

    #[test]
    fn gba_video_shares_the_gba_record() {
        let mut log = PlayLog::default();
        log.add_play(Platform::Gbav, "Shrek", 10, 1);
        assert_eq!(log.record(Platform::Gba, "Shrek").seconds, 10);
    }

    #[test]
    fn cleaning_starts_dust_and_wear_again_from_now() {
        let mut log = PlayLog::default();
        log.see([(Platform::Gb, "Tetris")], 100);
        log.add_play(Platform::Gb, "Tetris", 3600, 200);
        assert_eq!(log.record(Platform::Gb, "Tetris").worn_seconds(), 3600);
        log.clean(Platform::Gb, "Tetris", 300);
        let r = log.record(Platform::Gb, "Tetris");
        assert_eq!((r.idle_since(), r.worn_seconds()), (Some(300), 0));
        log.add_play(Platform::Gb, "Tetris", 60, 400);
        assert_eq!(log.record(Platform::Gb, "Tetris").worn_seconds(), 60);
    }

    #[test]
    fn seeing_a_game_twice_keeps_the_first_time() {
        let mut log = PlayLog::default();
        assert!(log.see([(Platform::Lynx, "Klax")], 10));
        assert!(!log.see([(Platform::Lynx, "Klax")], 20));
        assert_eq!(log.record(Platform::Lynx, "Klax").seen, Some(10));
    }

    #[test]
    fn play_time_reads_as_hours_and_minutes() {
        assert_eq!(play_time_text(20), "Under a minute");
        assert_eq!(play_time_text(45 * 60), "45 m");
        assert_eq!(play_time_text(12 * 3600 + 5 * 60), "12 h 05 m");
    }
}
