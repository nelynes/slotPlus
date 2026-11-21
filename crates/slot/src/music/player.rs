//! slotPlus music cart: the player, run as a core of slotPlus's own, so the cart goes in,
//! comes out, pauses and saves like any other.
//!
//! Up and Down pick, A opens a folder or plays a song (and the rest of its folder after
//! it), B goes back up, START pauses, L and R skip back and on, Left and Right wind back and
//! on ten seconds, and SELECT changes the order: in order, repeat all, repeat one, shuffle.
//!
//! What it was playing, and where, is kept as the cart's save, so it picks up there.

use std::path::{Path, PathBuf};

use slot_retro::{AvInfo, ButtonMask, CoreError, RetroCore};

use super::decode::{Song, OUT_RATE};
use super::library::{self, Entry};
use super::screen::{self, clock, Screen, ACCENT, BACK, BAND, DIM, INK, PICK};

const FPS: f64 = 60.0;
/// Rows of the list on screen.
const ROWS: usize = 8;
const ROW_H: i32 = 12;
const LIST_TOP: i32 = 18;
const PANEL_TOP: i32 = LIST_TOP + ROWS as i32 * ROW_H + 4;
/// Held, a button repeats after this many frames, then every `REPEAT_EVERY`.
const REPEAT_AFTER: u32 = 18;
const REPEAT_EVERY: u32 = 4;
const WIND_MS: u64 = 10_000;
/// Back on L this early in a song goes to the one before; later, to its start.
const RESTART_AFTER_MS: u64 = 3_000;
const SAVE_MAGIC: &str = "slotplus-music 1";

/// The music folder whose cart is playing a song right now (not paused, not stopped), for
/// the lid: shut while a song plays, only the screen goes off.
static PLAYING: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// Whether the music cart for the card at `root` is playing a song.
pub fn playing(root: &Path) -> bool {
    let music = library::music_dir(root);
    PLAYING
        .lock()
        .map(|p| p.as_deref() == Some(music.as_path()))
        .unwrap_or(false)
}

fn set_playing(top: &Path, on: bool) {
    if let Ok(mut p) = PLAYING.lock() {
        match on {
            true => *p = Some(top.to_path_buf()),
            false if p.as_deref() == Some(top) => *p = None,
            false => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Order {
    #[default]
    InOrder,
    RepeatAll,
    RepeatOne,
    Shuffle,
}

impl Order {
    fn next(self) -> Order {
        match self {
            Order::InOrder => Order::RepeatAll,
            Order::RepeatAll => Order::RepeatOne,
            Order::RepeatOne => Order::Shuffle,
            Order::Shuffle => Order::InOrder,
        }
    }

    fn text(self) -> &'static str {
        match self {
            Order::InOrder => "In Order",
            Order::RepeatAll => "Repeat All",
            Order::RepeatOne => "Repeat One",
            Order::Shuffle => "Shuffle",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Order::InOrder => "inorder",
            Order::RepeatAll => "repeatall",
            Order::RepeatOne => "repeatone",
            Order::Shuffle => "shuffle",
        }
    }

    fn parse(s: &str) -> Order {
        [
            Order::InOrder,
            Order::RepeatAll,
            Order::RepeatOne,
            Order::Shuffle,
        ]
        .into_iter()
        .find(|o| o.word() == s)
        .unwrap_or_default()
    }
}

pub struct MusicPlayer {
    top: PathBuf,
    dir: PathBuf,
    entries: Vec<Entry>,
    cursor: usize,
    scroll: usize,
    queue: Vec<PathBuf>,
    playing: Option<usize>,
    song: Option<Song>,
    paused: bool,
    order: Order,
    /// A song that wouldn't play, to say so.
    trouble: Option<String>,
    held: u16,
    held_for: u32,
    screen: Screen,
    dirty: bool,
    shown_second: u64,
    audio: Vec<i16>,
    frame: u64,
    rng: u64,
    /// What the cart keeps as its save: changed only when a song starts, pauses or ends, so
    /// the card isn't written every second.
    kept: Vec<u8>,
}

impl Default for MusicPlayer {
    fn default() -> Self {
        MusicPlayer::new()
    }
}

fn samples_at(frame: u64) -> u64 {
    (frame as f64 * OUT_RATE as f64 / FPS) as u64
}

impl MusicPlayer {
    pub fn new() -> MusicPlayer {
        MusicPlayer {
            top: PathBuf::new(),
            dir: PathBuf::new(),
            entries: Vec::new(),
            cursor: 0,
            scroll: 0,
            queue: Vec::new(),
            playing: None,
            song: None,
            paused: false,
            order: Order::InOrder,
            trouble: None,
            held: 0,
            held_for: 0,
            screen: Screen::default(),
            dirty: true,
            shown_second: u64::MAX,
            audio: Vec::new(),
            frame: 0,
            rng: 0x2545_f491_4f6c_dd1d,
            kept: Vec::new(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn now_playing(&self) -> Option<&Path> {
        self.queue.get(self.playing?).map(PathBuf::as_path)
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn order(&self) -> Order {
        self.order
    }

    pub fn position_ms(&self) -> u64 {
        self.song.as_ref().map_or(0, Song::position_ms)
    }

    fn open_dir(&mut self, dir: PathBuf, select: Option<&Path>) {
        self.entries = library::list(&dir);
        self.dir = dir;
        self.cursor = select
            .and_then(|s| self.entries.iter().position(|e| e.path() == s))
            .unwrap_or(0);
        self.scroll = 0;
        self.keep_cursor_shown();
        self.dirty = true;
    }

    fn keep_cursor_shown(&mut self) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + ROWS {
            self.scroll = self.cursor + 1 - ROWS;
        }
    }

    fn move_cursor(&mut self, by: i32) {
        let n = self.entries.len();
        if n == 0 {
            return;
        }
        self.cursor = (self.cursor as i64 + by as i64).rem_euclid(n as i64) as usize;
        self.keep_cursor_shown();
        self.dirty = true;
    }

    fn pick(&mut self) {
        match self.entries.get(self.cursor).cloned() {
            Some(Entry::Folder { path, .. }) => self.open_dir(path, None),
            Some(Entry::Song { path, .. }) => {
                self.queue = library::songs_in(&self.dir);
                let at = self.queue.iter().position(|p| *p == path).unwrap_or(0);
                self.start(at, 0);
            }
            None => {}
        }
    }

    fn back(&mut self) {
        if self.dir == self.top {
            return;
        }
        let Some(up) = self.dir.parent().map(Path::to_path_buf) else {
            return;
        };
        let was = self.dir.clone();
        self.open_dir(up, Some(&was));
    }

    /// Play the queue's song `at`, from `from_ms` in.
    fn start(&mut self, at: usize, from_ms: u64) {
        let Some(path) = self.queue.get(at).cloned() else {
            return self.stop();
        };
        self.playing = Some(at);
        self.paused = false;
        self.trouble = None;
        match Song::open(&path) {
            Ok(mut song) => {
                if from_ms > 0 {
                    song.seek_ms(from_ms);
                }
                self.song = Some(song);
            }
            Err(e) => {
                eprintln!("slot: music: {}: {e}", path.display());
                self.song = None;
                self.trouble = path.file_stem().map(|s| s.to_string_lossy().into_owned());
            }
        }
        self.dirty = true;
        self.keep();
    }

    fn stop(&mut self) {
        self.song = None;
        self.playing = None;
        self.paused = false;
        self.dirty = true;
        self.keep();
    }

    fn random(&mut self, n: usize) -> usize {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng % n.max(1) as u64) as usize
    }

    /// The song after this one, or `None` at the end of the queue.
    fn following(&mut self, by_hand: bool) -> Option<usize> {
        let (at, n) = (self.playing?, self.queue.len());
        match self.order {
            Order::RepeatOne if !by_hand => Some(at),
            Order::Shuffle if n > 1 => {
                let pick = self.random(n - 1);
                Some(if pick >= at { pick + 1 } else { pick })
            }
            Order::InOrder if at + 1 >= n && !by_hand => None,
            _ => Some((at + 1) % n),
        }
    }

    fn skip_on(&mut self) {
        match self.following(true) {
            Some(at) => self.start(at, 0),
            None => self.stop(),
        }
    }

    fn skip_back(&mut self) {
        let Some(at) = self.playing else {
            return;
        };
        if self.position_ms() > RESTART_AFTER_MS || self.queue.len() < 2 {
            return self.start(at, 0);
        }
        let n = self.queue.len();
        self.start((at + n - 1) % n, 0);
    }

    fn wind(&mut self, back: bool) {
        let Some(song) = &mut self.song else {
            return;
        };
        let at = song.position_ms();
        let to = match back {
            true => at.saturating_sub(WIND_MS),
            false => at + WIND_MS,
        };
        if song.length_ms.is_some_and(|len| to >= len) {
            return self.skip_on();
        }
        song.seek_ms(to);
        self.dirty = true;
    }

    fn toggle_pause(&mut self) {
        match self.playing {
            Some(_) if self.song.is_some() => {
                self.paused = !self.paused;
                self.keep();
            }
            // Nothing playing yet: START plays whatever's picked.
            _ => self.pick(),
        }
        self.dirty = true;
    }

    fn press(&mut self, b: u16) {
        match b {
            ButtonMask::UP => self.move_cursor(-1),
            ButtonMask::DOWN => self.move_cursor(1),
            ButtonMask::A => self.pick(),
            ButtonMask::B => self.back(),
            ButtonMask::START => self.toggle_pause(),
            ButtonMask::L => self.skip_back(),
            ButtonMask::R => self.skip_on(),
            ButtonMask::LEFT => self.wind(true),
            ButtonMask::RIGHT => self.wind(false),
            ButtonMask::SELECT => {
                self.order = self.order.next();
                self.dirty = true;
                self.keep();
            }
            _ => {}
        }
    }

    fn input(&mut self, mask: u16) {
        const BUTTONS: [u16; 10] = [
            ButtonMask::UP,
            ButtonMask::DOWN,
            ButtonMask::LEFT,
            ButtonMask::RIGHT,
            ButtonMask::A,
            ButtonMask::B,
            ButtonMask::START,
            ButtonMask::SELECT,
            ButtonMask::L,
            ButtonMask::R,
        ];
        const REPEATS: u16 =
            ButtonMask::UP | ButtonMask::DOWN | ButtonMask::LEFT | ButtonMask::RIGHT;
        let fresh = mask & !self.held;
        let repeating = mask & self.held & REPEATS;
        self.held_for = match repeating {
            0 => 0,
            _ => self.held_for + 1,
        };
        for b in BUTTONS {
            let repeats = repeating & b != 0
                && self.held_for >= REPEAT_AFTER
                && (self.held_for - REPEAT_AFTER).is_multiple_of(REPEAT_EVERY);
            if fresh & b != 0 || repeats {
                self.press(b);
            }
        }
        self.held = mask;
    }

    fn play(&mut self, frames: usize) {
        self.audio.clear();
        self.audio.resize(frames * 2, 0);
        if self.paused {
            return;
        }
        let Some(song) = &mut self.song else {
            return;
        };
        let made = song.fill(&mut self.audio);
        if song.ended() {
            let left = frames - made;
            match self.following(false) {
                Some(at) => {
                    self.start(at, 0);
                    if let (Some(next), true) = (&mut self.song, left > 0) {
                        next.fill(&mut self.audio[made * 2..]);
                    }
                }
                None => self.stop(),
            }
        }
    }

    /// What the cart keeps: the song (from the music folder), where it was, and the order.
    fn keep(&mut self) {
        let (song, at) = match self.now_playing() {
            Some(p) => (
                p.strip_prefix(&self.top)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .into_owned(),
                self.position_ms(),
            ),
            None => (String::new(), 0),
        };
        self.kept = format!("{SAVE_MAGIC}\n{song}\n{at}\n{}\n", self.order.word()).into_bytes();
    }

    /// Pick up from a kept record: the song's folder open on it, paused where it was.
    fn resume(&mut self, data: &[u8]) -> bool {
        let text = String::from_utf8_lossy(data);
        let mut lines = text.lines();
        if lines.next() != Some(SAVE_MAGIC) {
            return false;
        }
        let song = lines.next().unwrap_or_default();
        let at: u64 = lines
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        self.order = Order::parse(lines.next().unwrap_or_default().trim());
        let path = self.top.join(song);
        if song.is_empty() || !path.is_file() {
            self.keep();
            return true;
        }
        if let Some(dir) = path.parent().map(Path::to_path_buf) {
            self.open_dir(dir.clone(), Some(&path));
            self.queue = library::songs_in(&dir);
            if let Some(i) = self.queue.iter().position(|p| *p == path) {
                self.start(i, at);
                self.paused = true;
                self.keep();
            }
        }
        true
    }

    fn draw(&mut self) {
        let second = self.position_ms() / 1000;
        if !self.dirty && second == self.shown_second {
            return;
        }
        self.dirty = false;
        self.shown_second = second;
        let s = &mut self.screen;
        s.fill(BACK);
        // The band along the top: where you are.
        s.rect(0, 0, screen::W as i32, 15, BAND);
        let place = match self.dir.strip_prefix(&self.top) {
            Ok(rel) if rel.as_os_str().is_empty() => "Music".to_string(),
            Ok(rel) => rel
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            Err(_) => "Music".to_string(),
        };
        s.note(5, 4, ACCENT);
        let order = self.order.text();
        let order_w = s.width(&order.to_uppercase(), 9);
        s.text(&place, 10, 16, 2, screen::W as i32 - order_w - 12, INK);
        s.text(
            order,
            9,
            screen::W as i32 - order_w - 5,
            3,
            screen::W as i32,
            DIM,
        );

        // The list.
        let playing = self.queue.get(self.playing.unwrap_or(usize::MAX)).cloned();
        if self.entries.is_empty() {
            s.text(
                "No music here",
                10,
                12,
                LIST_TOP + 30,
                screen::W as i32 - 12,
                DIM,
            );
        }
        for (row, i) in (self.scroll..self.entries.len()).take(ROWS).enumerate() {
            let y = LIST_TOP + row as i32 * ROW_H;
            let e = &self.entries[i];
            if i == self.cursor {
                s.rect(2, y - 1, screen::W as i32 - 4, ROW_H, PICK);
            }
            let ink = match playing.as_deref() == Some(e.path()) {
                true => ACCENT,
                false => INK,
            };
            match e {
                Entry::Folder { .. } => s.folder(6, y + 2, DIM),
                Entry::Song { .. } if playing.as_deref() == Some(e.path()) => {
                    s.play(8, y + 1, 8, ACCENT)
                }
                Entry::Song { .. } => {}
            }
            s.text(e.name(), 10, 20, y, screen::W as i32 - 8, ink);
        }
        // Where the list is, when it's longer than the screen.
        if self.entries.len() > ROWS {
            let track = ROWS as i32 * ROW_H;
            let thumb = (track * ROWS as i32 / self.entries.len() as i32).max(6);
            let at =
                (track - thumb) * self.scroll as i32 / (self.entries.len() - ROWS).max(1) as i32;
            s.rect(screen::W as i32 - 3, LIST_TOP - 1 + at, 2, thumb, DIM);
        }

        // What's playing.
        s.rect(0, PANEL_TOP - 2, screen::W as i32, 1, BAND);
        match (&self.song, &playing) {
            (Some(song), Some(path)) => {
                let title = song.tags.title.clone().unwrap_or_else(|| {
                    path.file_stem()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
                let by = [song.tags.artist.clone(), song.tags.album.clone()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · ");
                let (at, len) = (song.position_ms(), song.length_ms);
                match self.paused {
                    true => s.pause(6, PANEL_TOP + 3, 9, ACCENT),
                    false => s.play(7, PANEL_TOP + 3, 9, ACCENT),
                }
                s.text(&title, 11, 20, PANEL_TOP, screen::W as i32 - 6, INK);
                s.text(&by, 9, 20, PANEL_TOP + 13, screen::W as i32 - 6, DIM);
                let times = match len {
                    Some(len) => format!("{} / {}", clock(at), clock(len)),
                    None => clock(at),
                };
                let tw = s.width(&times, 9);
                let bar_y = PANEL_TOP + 29;
                let bar_w = screen::W as i32 - 20 - tw - 10;
                s.rect(8, bar_y, bar_w, 3, BAND);
                if let Some(len) = len.filter(|l| *l > 0) {
                    let fill = (bar_w as u64 * at.min(len) / len) as i32;
                    s.rect(8, bar_y, fill, 3, ACCENT);
                }
                s.text(
                    &times,
                    9,
                    screen::W as i32 - tw - 6,
                    bar_y - 4,
                    screen::W as i32,
                    DIM,
                );
            }
            _ => {
                let say = match &self.trouble {
                    Some(name) => format!("Can't play {name}"),
                    None => "A to play  ·  START to pause".to_string(),
                };
                s.text(&say, 9, 8, PANEL_TOP + 12, screen::W as i32 - 8, DIM);
            }
        }
    }
}

impl Drop for MusicPlayer {
    fn drop(&mut self) {
        set_playing(&self.top, false);
    }
}

impl RetroCore for MusicPlayer {
    fn load(&mut self, rom: &Path) -> Result<(), CoreError> {
        // The cart's "ROM" is the music folder itself.
        self.top = rom.to_path_buf();
        self.open_dir(self.top.clone(), None);
        self.keep();
        self.draw();
        Ok(())
    }

    fn run_frame(&mut self, input: ButtonMask) {
        self.input(input.0);
        let start = samples_at(self.frame);
        self.frame += 1;
        let frames = (samples_at(self.frame) - start) as usize;
        self.play(frames);
        set_playing(&self.top, self.song.is_some() && !self.paused);
        self.draw();
    }

    fn reset(&mut self) {
        self.stop();
        self.open_dir(self.top.clone(), None);
    }

    fn video_xrgb8888(&self) -> &[u8] {
        &self.screen.px
    }

    fn take_audio(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.audio)
    }

    fn serialize(&mut self) -> Result<Vec<u8>, CoreError> {
        self.keep();
        Ok(self.kept.clone())
    }

    fn unserialize(&mut self, data: &[u8]) -> Result<(), CoreError> {
        let paused = self.paused;
        if !self.resume(data) {
            return Err(CoreError::Load("not a music cart state".into()));
        }
        self.paused = paused && self.song.is_some();
        Ok(())
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        Some(self.kept.clone())
    }

    fn load_save_ram(&mut self, data: &[u8]) -> Result<(), CoreError> {
        self.resume(data);
        Ok(())
    }

    fn av_info(&self) -> AvInfo {
        AvInfo {
            fps: FPS,
            sample_rate: OUT_RATE as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::decode::tests::wav;

    fn card() -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let m = d.path().join("Music");
        std::fs::create_dir_all(m.join("Album")).unwrap();
        wav(&m.join("Album/01 First.wav"), 44_100, 2, 0.5);
        wav(&m.join("Album/02 Second.wav"), 44_100, 2, 0.5);
        wav(&m.join("Loose.wav"), 22_050, 1, 0.5);
        (d, m)
    }

    fn tap(p: &mut MusicPlayer, b: u16) {
        p.run_frame(ButtonMask(b));
        p.run_frame(ButtonMask(0));
    }

    fn run(p: &mut MusicPlayer, frames: usize) -> Vec<i16> {
        let mut out = Vec::new();
        for _ in 0..frames {
            p.run_frame(ButtonMask(0));
            out.extend(p.take_audio());
        }
        out
    }

    #[test]
    fn it_browses_folders_and_plays_a_song_then_the_rest_of_its_folder() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        let names: Vec<_> = p.entries().iter().map(Entry::name).collect();
        assert_eq!(names, ["Album", "Loose"]);
        tap(&mut p, ButtonMask::A);
        assert_eq!(p.dir(), m.join("Album"));
        tap(&mut p, ButtonMask::A);
        assert_eq!(
            p.now_playing(),
            Some(m.join("Album/01 First.wav").as_path())
        );
        let audio = run(&mut p, 10);
        assert_eq!(
            audio.len(),
            10 * 735 * 2,
            "a frame's worth of sound each frame"
        );
        assert!(audio.iter().any(|v| v.abs() > 8_000));
        run(&mut p, 40);
        assert_eq!(
            p.now_playing(),
            Some(m.join("Album/02 Second.wav").as_path()),
            "on to the next"
        );
        run(&mut p, 40);
        assert_eq!(p.now_playing(), None, "in order, it stops at the end");
        tap(&mut p, ButtonMask::B);
        assert_eq!(p.dir(), m);
        assert_eq!(p.cursor(), 0, "back on the folder it came out of");
        tap(&mut p, ButtonMask::B);
        assert_eq!(p.dir(), m, "never above the music folder");
    }

    #[test]
    fn start_pauses_and_the_save_picks_up_where_it_was() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        tap(&mut p, ButtonMask::A);
        tap(&mut p, ButtonMask::DOWN);
        tap(&mut p, ButtonMask::A);
        run(&mut p, 12);
        tap(&mut p, ButtonMask::START);
        assert!(p.paused());
        let quiet = run(&mut p, 5);
        assert!(quiet.iter().all(|v| *v == 0), "paused is silent");
        tap(&mut p, ButtonMask::SELECT);
        let save = p.save_ram().unwrap();

        let mut again = MusicPlayer::new();
        again.load(&m).unwrap();
        again.load_save_ram(&save).unwrap();
        assert_eq!(
            again.now_playing(),
            Some(m.join("Album/02 Second.wav").as_path())
        );
        assert!(again.paused(), "it waits for START");
        assert_eq!(again.dir(), m.join("Album"));
        assert_eq!(again.cursor(), 1);
        assert_eq!(again.order(), Order::RepeatAll);
        assert!(again.position_ms() > 0);
    }

    #[test]
    fn repeat_one_plays_the_same_song_again_and_skipping_still_moves_on() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        tap(&mut p, ButtonMask::A);
        tap(&mut p, ButtonMask::A);
        tap(&mut p, ButtonMask::SELECT);
        tap(&mut p, ButtonMask::SELECT);
        assert_eq!(p.order(), Order::RepeatOne);
        run(&mut p, 45);
        assert_eq!(
            p.now_playing(),
            Some(m.join("Album/01 First.wav").as_path())
        );
        tap(&mut p, ButtonMask::R);
        assert_eq!(
            p.now_playing(),
            Some(m.join("Album/02 Second.wav").as_path())
        );
        tap(&mut p, ButtonMask::L);
        assert_eq!(
            p.now_playing(),
            Some(m.join("Album/01 First.wav").as_path())
        );
    }

    #[test]
    fn a_song_that_will_not_play_is_named_and_nothing_breaks() {
        let (_d, m) = card();
        std::fs::write(m.join("Broken.mp3"), b"nope").unwrap();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        tap(&mut p, ButtonMask::DOWN);
        tap(&mut p, ButtonMask::A);
        assert!(p.trouble.as_deref() == Some("Broken"));
        assert_eq!(run(&mut p, 3).len(), 3 * 735 * 2);
    }

    #[test]
    fn a_save_from_something_else_is_left_alone() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        p.load_save_ram(&[0u8; 32]).unwrap();
        assert_eq!(p.now_playing(), None);
        assert!(p.unserialize(&[1, 2, 3]).is_err());
    }
}
