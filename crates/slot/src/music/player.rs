//! slotPlus music cart: the player, run as a core of slotPlus's own, so the cart goes in,
//! comes out, pauses and saves like any other.
//!
//! In the spirit of the music carts sold for the GBA: each file is a step on a staircase,
//! folders are doors and songs are notes (playlists two), and a little cart climbs it,
//! picking the step he stands on. At the top of a folder's stairs a warp pipe takes him
//! back to the top of the music, and the EXIT door flies along beside him on a propeller.
//! Left and Right hop him down and up a step (faster the longer they're held), Up and Down
//! a few at once, A plays a song (and the rest of its folder or playlist after it) or takes
//! him through a door or down the pipe, and B pauses, then stops, then takes him out of the
//! EXIT door. START plays and pauses, L and R skip back and on (held, they wind ten seconds
//! at a time), SELECT changes the order (in order, repeat all, repeat one, shuffle), and
//! SELECT held switches the theme: classic, or Monochrome-ish. While a song plays he dances
//! on its step, the notes dance with him, and Up and Down are the handheld's volume
//! (slotPlus takes them before they reach the player).
//!
//! What it was playing, and where, is kept as the cart's save, so it picks up there.

use std::path::{Path, PathBuf};

use slot_retro::{AvInfo, ButtonMask, CoreError, RetroCore};

use super::decode::{Song, OUT_RATE};
use super::library::{self, Entry};
use super::screen::{self, clock, Palette, Pose, Screen, Type, CLASSIC, MONOCHROME};

const FPS: f64 = 60.0;

/// The staircase: each step `STEP` across and `ROW` up from the one before, the first with
/// its tread at `BASE`. A file's name sits to the right of its step's riser, and its door or
/// note, and he, stand on the tread above it. `VISIBLE` names fit on screen.
const ROW: i32 = 17;
const STEP: i32 = 17;
const X0: i32 = 2;
const BASE: i32 = 145;
const VISIBLE: usize = 7;
/// The line along the bottom that shows how far into the song it is.
const BAR_Y: i32 = 150;
/// Where the EXIT door waits while a song plays, and where it starts in a new folder.
const DOOR_CORNER: (f32, f32) = (4.0, 30.0);
const DOOR_START: (f32, f32) = (40.0, 44.0);
/// How far from him the door flies, and how quickly it catches up.
const DOOR_OFFSET: (f32, f32) = (-30.0, -40.0);
const DOOR_EASE: f32 = 0.12;
/// How tall the warp pipe is: he stands on top of it.
const PIPE_H: f32 = 12.0;
/// SELECT held this long switches the theme.
const THEME_HOLD: u32 = 40;

/// A held Left or Right hops again after `REPEAT_AFTER` frames, then every `REPEAT_FIRST`
/// frames, one frame sooner each hop down to `REPEAT_FASTEST`.
const REPEAT_AFTER: u32 = 18;
const REPEAT_FIRST: u32 = 12;
const REPEAT_FASTEST: u32 = 3;
/// Up and Down hop this many steps.
const PAGE: i32 = 5;
const HOP_FRAMES: u32 = 8;
const DOOR_FRAMES: u32 = 14;
const DROP_FRAMES: u32 = 24;
const DANCE_FRAMES: u64 = 10;
/// L or R held this long winds instead of skipping, and winds again this often.
const WIND_AFTER: u32 = 18;
const WIND_EVERY: u32 = 8;
const WIND_MS: u64 = 10_000;
/// Back on L this early in a song goes to the one before; later, to its start.
const RESTART_AFTER_MS: u64 = 3_000;
const SAVE_MAGIC: &str = "slotplus-music 2";
const SAVE_MAGIC_1: &str = "slotplus-music 1";

/// The music folder whose cart is playing a song right now (not paused, not stopped), for
/// the lid and the volume: shut while a song plays, only the screen goes off; and Up and
/// Down are the handheld's volume.
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
            Order::InOrder => "IN ORDER",
            Order::RepeatAll => "REPEAT ALL",
            Order::RepeatOne => "REPEAT ONE",
            Order::Shuffle => "SHUFFLE",
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

/// What the cart on the stairs is doing.
#[derive(Clone, Debug, PartialEq)]
enum Move {
    Still,
    /// Hopping from step `from` to step `to`.
    Hop {
        from: f32,
        to: f32,
        t: u32,
        len: u32,
    },
    /// Going in through the door on his step, to the folder behind it.
    Enter {
        t: u32,
        to: PathBuf,
    },
    /// Out of the EXIT door at the top of a new folder, and down to its first step.
    Drop {
        t: u32,
    },
    /// Up from his step and in through the EXIT door, back to the folder before.
    Leave {
        t: u32,
        from: PathBuf,
    },
    /// Out of the door of the folder he just left.
    Emerge {
        t: u32,
    },
    /// Down the warp pipe, to the top of the music.
    Pipe {
        t: u32,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Classic,
    Monochrome,
}

impl Theme {
    fn palette(self) -> Palette {
        match self {
            Theme::Classic => CLASSIC,
            Theme::Monochrome => MONOCHROME,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Theme::Classic => "classic",
            Theme::Monochrome => "monochrome",
        }
    }

    fn parse(s: &str) -> Theme {
        match s {
            "monochrome" => Theme::Monochrome,
            _ => Theme::Classic,
        }
    }
}

pub struct MusicPlayer {
    top: PathBuf,
    dir: PathBuf,
    entries: Vec<Entry>,
    cursor: usize,
    /// The first step on screen, and where the stairs are drawn from as they glide to it.
    scroll: usize,
    view: f32,
    /// Where the cart is on the stairs, in steps.
    guide: f32,
    moving: Move,
    queue: Vec<PathBuf>,
    /// The playlist the queue came from, when it did.
    list: Option<PathBuf>,
    playing: Option<usize>,
    song: Option<Song>,
    paused: bool,
    order: Order,
    /// A song that wouldn't play, to say so.
    trouble: Option<String>,
    held: u16,
    /// How long Left or Right has been held, and when it hops next.
    held_for: u32,
    next_hop: u32,
    gap: u32,
    /// How long L or R has been held.
    shoulder_for: u32,
    /// How long SELECT has been held, and whether this hold already switched the theme.
    select_for: u32,
    select_used: bool,
    theme: Theme,
    /// Where the EXIT door is flying, on screen.
    door: (f32, f32),
    screen: Screen,
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

fn ease(p: f32) -> f32 {
    p * p * (3.0 - 2.0 * p)
}

/// Where step `k` (counted from the bottom of the screen) is: its left edge and its tread.
fn step_at(k: f32) -> (f32, f32) {
    (X0 as f32 + k * STEP as f32, BASE as f32 - k * ROW as f32)
}

impl MusicPlayer {
    pub fn new() -> MusicPlayer {
        MusicPlayer {
            top: PathBuf::new(),
            dir: PathBuf::new(),
            entries: Vec::new(),
            cursor: 0,
            scroll: 0,
            view: 0.0,
            guide: 0.0,
            moving: Move::Still,
            queue: Vec::new(),
            list: None,
            playing: None,
            song: None,
            paused: false,
            order: Order::InOrder,
            trouble: None,
            held: 0,
            held_for: 0,
            next_hop: 0,
            gap: REPEAT_FIRST,
            shoulder_for: 0,
            select_for: 0,
            select_used: false,
            theme: Theme::Classic,
            door: DOOR_START,
            screen: Screen::default(),
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

    pub fn theme(&self) -> Theme {
        self.theme
    }

    /// Whether this folder's stairs end in a warp pipe (every folder but the top).
    fn has_pipe(&self) -> bool {
        self.dir != self.top
    }

    /// How many places he can stand: a step for each file, and the pipe.
    fn spots(&self) -> usize {
        self.entries.len() + self.has_pipe() as usize
    }

    /// Whether he's standing on the warp pipe.
    pub fn on_pipe(&self) -> bool {
        self.has_pipe() && self.cursor == self.entries.len()
    }

    pub fn position_ms(&self) -> u64 {
        self.song.as_ref().map_or(0, Song::position_ms)
    }

    /// Whether the cart is busy going through a door (input waits for him).
    pub fn walking(&self) -> bool {
        !matches!(self.moving, Move::Still | Move::Hop { .. })
    }

    /// The step that stands for what's playing: its playlist, or the song itself.
    fn playing_step(&self) -> Option<&Path> {
        self.list.as_deref().or(self.now_playing())
    }

    fn open_dir(&mut self, dir: PathBuf, select: Option<&Path>) {
        self.entries = library::list(&dir);
        self.dir = dir;
        self.cursor = select
            .and_then(|s| self.entries.iter().position(|e| e.path() == s))
            .unwrap_or(0);
        self.scroll = 0;
        self.keep_cursor_shown();
        self.view = self.scroll as f32;
        self.guide = self.cursor as f32;
    }

    fn keep_cursor_shown(&mut self) {
        // Keep a step above him in view while there is one, so he can see where he's going.
        let top_room = (self.cursor + 2).min(self.spots());
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if top_room > self.scroll + VISIBLE {
            self.scroll = top_room - VISIBLE;
        }
    }

    /// Hop `by` steps, as far as the stairs go.
    fn hop(&mut self, by: i32) {
        let n = self.spots();
        if n == 0 {
            return;
        }
        let to = (self.cursor as i64 + by as i64).clamp(0, n as i64 - 1) as usize;
        if to == self.cursor {
            return;
        }
        self.cursor = to;
        self.keep_cursor_shown();
        let len = HOP_FRAMES.min(self.gap.max(REPEAT_FASTEST + 1));
        self.moving = Move::Hop {
            from: self.guide,
            to: to as f32,
            t: 0,
            len,
        };
    }

    fn pick(&mut self) {
        match self.entries.get(self.cursor).cloned() {
            Some(Entry::Folder { path, .. }) => {
                self.moving = Move::Enter { t: 0, to: path };
                self.guide = self.cursor as f32;
            }
            Some(Entry::Song { path, .. }) => {
                self.queue = library::songs_in(&self.dir);
                self.list = None;
                let at = self.queue.iter().position(|p| *p == path).unwrap_or(0);
                self.start(at, 0);
            }
            Some(Entry::Playlist { path, .. }) => {
                self.queue = library::playlist_songs(&path);
                if self.queue.is_empty() {
                    self.trouble = path.file_stem().map(|s| s.to_string_lossy().into_owned());
                    return;
                }
                self.list = Some(path);
                let at = match self.order {
                    Order::Shuffle => self.random(self.queue.len()),
                    _ => 0,
                };
                self.start(at, 0);
            }
            None if self.on_pipe() => self.moving = Move::Pipe { t: 0 },
            None => {}
        }
    }

    /// B: pause what's playing; again, stop it; with nothing playing, out of the EXIT door.
    fn b(&mut self) {
        match (&self.song, self.paused) {
            (Some(_), false) => {
                self.paused = true;
                self.keep();
            }
            (Some(_), true) => self.stop(),
            (None, _) if self.playing.is_some() => self.stop(),
            (None, _) => self.back(),
        }
    }

    fn back(&mut self) {
        if self.dir == self.top || self.dir.parent().is_none() {
            return;
        }
        self.moving = Move::Leave {
            t: 0,
            from: self.dir.clone(),
        };
    }

    /// Play the queue's song `at`, from `from_ms` in.
    fn start(&mut self, at: usize, from_ms: u64) {
        let Some(path) = self.queue.get(at).cloned() else {
            return self.stop();
        };
        // He follows the music along a folder when he was standing on it.
        let follow = self.list.is_none()
            && self.now_playing().is_some()
            && self.entries.get(self.cursor).map(Entry::path) == self.now_playing();
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
        if follow && !self.walking() {
            if let Some(i) = self.entries.iter().position(|e| e.path() == path) {
                self.hop(i as i32 - self.cursor as i32);
            }
        }
        self.keep();
    }

    fn stop(&mut self) {
        self.song = None;
        self.playing = None;
        self.list = None;
        self.paused = false;
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
    }

    fn toggle_pause(&mut self) {
        match self.playing {
            Some(_) if self.song.is_some() => {
                self.paused = !self.paused;
                self.keep();
            }
            // Nothing playing yet: START plays whatever he's standing on.
            _ => self.pick(),
        }
    }

    fn input(&mut self, mask: u16) {
        let fresh = mask & !self.held;
        let released = self.held & !mask;
        self.held = mask;
        if self.walking() {
            return;
        }
        let pressed = |b: u16| fresh & b != 0;
        if pressed(ButtonMask::A) {
            return self.pick();
        }
        if pressed(ButtonMask::B) {
            return self.b();
        }
        if pressed(ButtonMask::START) {
            self.toggle_pause();
        }
        // SELECT: tapped, the order; held, the theme.
        if mask & ButtonMask::SELECT != 0 {
            self.select_for += 1;
            if self.select_for >= THEME_HOLD && !self.select_used {
                self.select_used = true;
                self.theme = match self.theme {
                    Theme::Classic => Theme::Monochrome,
                    Theme::Monochrome => Theme::Classic,
                };
                self.keep();
            }
        } else {
            if released & ButtonMask::SELECT != 0 && !self.select_used {
                self.order = self.order.next();
                self.keep();
            }
            self.select_for = 0;
            self.select_used = false;
        }
        if pressed(ButtonMask::UP) {
            self.hop(PAGE);
        }
        if pressed(ButtonMask::DOWN) {
            self.hop(-PAGE);
        }
        // Left and Right: a hop, then hops that come faster the longer they're held.
        let lr = mask & (ButtonMask::LEFT | ButtonMask::RIGHT);
        let way = if lr & ButtonMask::RIGHT != 0 { 1 } else { -1 };
        if fresh & (ButtonMask::LEFT | ButtonMask::RIGHT) != 0 {
            self.held_for = 0;
            self.gap = REPEAT_FIRST;
            self.next_hop = REPEAT_AFTER;
            self.hop(way);
        } else if lr != 0 {
            self.held_for += 1;
            if self.held_for >= self.next_hop {
                self.hop(way);
                self.next_hop = self.held_for + self.gap;
                self.gap = (self.gap - 1).max(REPEAT_FASTEST);
            }
        }
        // L and R: a tap skips, a hold winds.
        let lr2 = mask & (ButtonMask::L | ButtonMask::R);
        if lr2 != 0 {
            self.shoulder_for += 1;
            let back = lr2 & ButtonMask::L != 0;
            if self.shoulder_for >= WIND_AFTER
                && (self.shoulder_for - WIND_AFTER).is_multiple_of(WIND_EVERY)
            {
                self.wind(back);
            }
        } else {
            if released & (ButtonMask::L | ButtonMask::R) != 0 && self.shoulder_for < WIND_AFTER {
                match released & ButtonMask::L != 0 {
                    true => self.skip_back(),
                    false => self.skip_on(),
                }
            }
            self.shoulder_for = 0;
        }
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

    /// Move him along, and the stairs with him.
    fn step(&mut self) {
        let target = self.scroll as f32;
        let gap = target - self.view;
        self.view = match gap.abs() < 0.05 {
            true => target,
            false => self.view + gap * 0.3,
        };
        match &mut self.moving {
            Move::Still => {}
            Move::Hop { from, to, t, len } => {
                *t += 1;
                let p = (*t as f32 / *len as f32).min(1.0);
                self.guide = *from + (*to - *from) * p;
                if *t >= *len {
                    self.guide = *to;
                    self.moving = Move::Still;
                }
            }
            Move::Enter { t, to } => {
                *t += 1;
                if *t >= DOOR_FRAMES {
                    let to = to.clone();
                    self.open_dir(to, None);
                    self.door = DOOR_START;
                    self.moving = Move::Drop { t: 0 };
                }
            }
            Move::Pipe { t } => {
                *t += 1;
                if *t >= DOOR_FRAMES + 6 {
                    // Back at the top, out of the door of the folder he was under.
                    let first = self
                        .dir
                        .strip_prefix(&self.top)
                        .ok()
                        .and_then(|rel| rel.components().next())
                        .map(|c| self.top.join(c.as_os_str()));
                    self.open_dir(self.top.clone(), first.as_deref());
                    self.moving = Move::Emerge { t: 0 };
                }
            }
            Move::Drop { t } => {
                *t += 1;
                if *t >= DROP_FRAMES {
                    self.moving = Move::Still;
                }
            }
            Move::Leave { t, from } => {
                *t += 1;
                if *t >= DROP_FRAMES {
                    let from = from.clone();
                    let up = from.parent().map(Path::to_path_buf).unwrap_or_default();
                    self.open_dir(up, Some(&from));
                    self.moving = Move::Emerge { t: 0 };
                }
            }
            Move::Emerge { t } => {
                *t += 1;
                if *t >= DOOR_FRAMES {
                    self.moving = Move::Still;
                }
            }
        }
        // The EXIT door flies after him, or to its corner while there's a song on. It holds still
        // while he goes in or out of it.
        if !matches!(self.moving, Move::Drop { .. } | Move::Leave { .. }) {
            let target = match self.song.is_some() {
                true => DOOR_CORNER,
                false => {
                    let (gx, gy) = self.spot(self.guide, self.view);
                    (
                        (gx + DOOR_OFFSET.0).clamp(4.0, 200.0),
                        (gy + DOOR_OFFSET.1).clamp(30.0, 130.0),
                    )
                }
            };
            self.door.0 += (target.0 - self.door.0) * DOOR_EASE;
            self.door.1 += (target.1 - self.door.1) * DOOR_EASE;
        }
    }

    /// Where he stands on spot `g`, on screen: the tread above its name, or the pipe's top.
    fn spot(&self, g: f32, view: f32) -> (f32, f32) {
        let (xs, yb) = step_at(g + 1.0 - view);
        let n = self.entries.len() as f32;
        let lift = match self.has_pipe() {
            true => PIPE_H * (1.0 - (g - n).abs()).clamp(0.0, 1.0),
            false => 0.0,
        };
        (xs + 4.0, yb - lift)
    }

    /// What the cart keeps: what's playing (from the music folder), its place in the queue,
    /// where it was, and the order.
    fn keep(&mut self) {
        let rel = |p: &Path, top: &Path| {
            p.strip_prefix(top)
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned()
        };
        let (source, index, at) = match (&self.list, self.now_playing()) {
            (Some(list), Some(_)) => (
                rel(list, &self.top),
                self.playing.unwrap_or(0),
                self.position_ms(),
            ),
            (None, Some(p)) => (rel(p, &self.top), 0, self.position_ms()),
            _ => (String::new(), 0, 0),
        };
        self.kept = format!(
            "{SAVE_MAGIC}\n{source}\n{index}\n{at}\n{}\n{}\n",
            self.order.word(),
            self.theme.word()
        )
        .into_bytes();
    }

    /// Pick up from a kept record: the folder open on what was playing, paused where it was.
    fn resume(&mut self, data: &[u8]) -> bool {
        let text = String::from_utf8_lossy(data);
        let mut lines = text.lines();
        let (source, index, at, order) = match lines.next() {
            Some(SAVE_MAGIC) => {
                let source = lines.next().unwrap_or_default().to_string();
                let index: usize = lines
                    .next()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                let at: u64 = lines
                    .next()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                let order = lines.next().unwrap_or_default();
                self.theme = Theme::parse(lines.next().unwrap_or_default().trim());
                (source, index, at, order)
            }
            Some(SAVE_MAGIC_1) => {
                let source = lines.next().unwrap_or_default().to_string();
                let at: u64 = lines
                    .next()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                (source, 0, at, lines.next().unwrap_or_default())
            }
            _ => return false,
        };
        self.order = Order::parse(order.trim());
        let path = self.top.join(&source);
        if source.is_empty() || !path.is_file() {
            self.keep();
            return true;
        }
        let Some(dir) = path.parent().map(Path::to_path_buf) else {
            return true;
        };
        self.open_dir(dir.clone(), Some(&path));
        let found = match library::is_playlist(&path) {
            true => {
                self.queue = library::playlist_songs(&path);
                self.list = Some(path.clone());
                (index < self.queue.len()).then_some(index)
            }
            false => {
                self.queue = library::songs_in(&dir);
                self.list = None;
                self.queue.iter().position(|p| *p == path)
            }
        };
        if let Some(i) = found {
            self.start(i, at);
            self.paused = true;
            self.keep();
        }
        true
    }

    fn draw(&mut self) {
        let frame = self.frame;
        let pal = self.theme.palette();
        self.screen.pal = pal;
        let playing_step = self.playing_step().map(Path::to_path_buf);
        let sounding = self.song.is_some() && !self.paused;
        let dance = match (frame / DANCE_FRAMES).is_multiple_of(2) {
            true => Pose::DanceLeft,
            false => Pose::DanceRight,
        };
        let view = self.view;
        let n = self.entries.len();
        let first = (view.floor() as isize - 1).max(0) as usize;
        let last = (view.ceil() as usize + VISIBLE + 1).min(n);
        // What each step on screen shows: him in front of its door or note, or its door
        // part-shut.
        let marks: Vec<(bool, Option<i32>)> = (first..last)
            .map(|i| {
                let here = (self.guide - i as f32).abs() < 0.5 && !self.walking_away_from(i);
                (here, self.door_open_at(i))
            })
            .collect();
        let (gx, gy, pose, shown) = self.guide_spot(view, dance, sounding, playing_step.as_deref());
        let pipe = self.has_pipe().then(|| {
            let (xs, yb) = step_at(n as f32 + 1.0 - view);
            (xs.round() as i32 + 2, yb.round() as i32)
        });
        let door = (self.door.0.round() as i32, self.door.1.round() as i32);
        let door_shown = self.has_pipe() && !self.door_hidden();
        let place = match self.dir.strip_prefix(&self.top) {
            Ok(rel) if !rel.as_os_str().is_empty() => rel
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_default(),
            _ => "Music".to_string(),
        };
        let title = match (&self.song, self.now_playing()) {
            (Some(song), Some(path)) => Some(song.tags.title.clone().unwrap_or_else(|| {
                path.file_stem()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default()
            })),
            _ => None,
        };
        let times = self
            .song
            .as_ref()
            .map(|song| (song.position_ms(), song.length_ms));
        let inside = self.has_pipe();
        let entries = std::mem::take(&mut self.entries);
        let w = screen::W as i32;
        let s = &mut self.screen;
        s.fill(pal.back);

        if entries.is_empty() {
            s.text("No music here", Type::Title, 60, 70, (0, w), 0, pal.text);
        }
        // The staircase's outline, from the bottom step up and off the top of the screen.
        let lowest = (view.floor() as i32 - 1).max(0);
        for i in lowest..lowest + VISIBLE as i32 + 6 {
            let (xs, yb) = step_at(i as f32 - view);
            let (xs, yb) = (xs.round() as i32, yb.round() as i32);
            if yb < -ROW {
                break;
            }
            s.hline(xs, xs + STEP, yb, pal.line);
            s.vline(xs + STEP, yb - ROW, yb, pal.line);
        }
        // Each file's name to the right of its riser, the picked one and the one playing lit,
        // and its door or note on the tread above.
        let ripple = sounding.then_some(frame as f32 * 0.3);
        for i in first..last {
            let (xs, yb) = step_at(i as f32 - view);
            let (xs, yb) = (xs.round() as i32, yb.round() as i32);
            let xt = xs + STEP;
            let e = &entries[i];
            let lit = playing_step.as_deref() == Some(e.path());
            let picked = i == self.cursor;
            match (picked, lit) {
                (true, true) => s.band(xt + 1, yb - ROW + 1, ROW - 1, ripple, true),
                (true, false) => s.band(xt + 1, yb - ROW + 1, ROW - 1, None, true),
                (false, true) if sounding => s.band(xt + 1, yb - ROW + 1, ROW - 1, ripple, false),
                _ => {}
            }
            let (guide_here, open) = marks[i - first];
            let (ix, floor) = (xt + 4, yb - ROW);
            // While a song plays the notes dance: each bobs in turn, the playing one most.
            let bob = match sounding {
                true if lit => [0, 1, 2, 1][((frame / 6) % 4) as usize],
                true => ((frame / 12 + i as u64) % 2) as i32,
                false => 0,
            };
            let sway = match sounding && lit {
                true => [0, 1, 0, -1][((frame / 8) % 4) as usize],
                false => 0,
            };
            match e {
                Entry::Folder { .. } => match open {
                    Some(shut) => s.doorway(ix, floor, shut, pal.door),
                    None if !guide_here => s.door(ix, floor, pal.door),
                    None => s.doorway(ix, floor, 0, pal.door),
                },
                Entry::Song { .. } if !guide_here => {
                    s.note(ix + 2 + sway, floor - 1 - bob, pal.note)
                }
                Entry::Playlist { .. } if !guide_here => {
                    s.notes(ix + sway, floor - 1 - bob, pal.note)
                }
                _ => {}
            }
            let ink = if picked { pal.text_on } else { pal.text };
            let name = e.name().to_string();
            s.text(
                &name,
                Type::Title,
                xt + 3,
                yb - ROW - 3,
                (xt + 1, w - 1),
                0,
                ink,
            );
        }
        self.entries = entries;
        let s = &mut self.screen;
        if let Some((px, floor)) = pipe {
            s.pipe(px, floor);
            s.text("TOP", Type::Small, px, floor + 1, (0, w), 0, pal.dim);
        }

        // Him.
        if shown > 0 {
            s.guide(gx, gy, pose, shown);
        }
        if door_shown {
            s.exit_door(door.0, door.1, frame);
        }

        // What's playing: its title in a frame at the top, the big play or pause sign, and
        // how far in it is along the bottom.
        match &title {
            Some(title) => {
                let (bx, by, bw, bh) = (24, 2, 94, 19);
                s.rect(bx, by, bw, bh, pal.back);
                s.hline(bx, bx + bw - 1, by, pal.line);
                s.hline(bx, bx + bw - 1, by + bh - 1, pal.line);
                s.vline(bx, by, by + bh - 1, pal.line);
                s.vline(bx + bw - 1, by, by + bh - 1, pal.line);
                let (left, right) = (bx + 3, bx + bw - 4);
                let tw = s.width(title, Type::Title);
                // A long title scrolls by, after a moment's pause at the start.
                let skip = match tw > right - left {
                    true => {
                        let span = (tw + 24) as u64;
                        ((frame / 2) % (span + 60)).saturating_sub(60) as i32
                    }
                    false => 0,
                };
                s.text(
                    title,
                    Type::Title,
                    left,
                    by - 1,
                    (left, right),
                    skip,
                    pal.text,
                );
                if skip > 0 {
                    s.text(
                        title,
                        Type::Title,
                        left + tw + 24,
                        by - 1,
                        (left, right),
                        skip,
                        pal.text,
                    );
                }
                match sounding {
                    true => s.big_play(26, 50, 37),
                    false => s.big_pause(26, 52, 33),
                }
            }
            None => {
                let say = match &self.trouble {
                    Some(name) => format!("Can't play {name}"),
                    None => place,
                };
                s.text(&say, Type::Title, 24, 1, (24, 118), 0, pal.dim);
            }
        }
        // The bottom line: how far into the song, the play order, and the time.
        s.rect(0, BAR_Y, w, screen::H as i32 - BAR_Y, pal.back);
        s.hline(4, w - 5, BAR_Y, pal.line);
        let order = self.order.text();
        s.text(order, Type::Small, 4, BAR_Y + 1, (0, w), 0, pal.dim);
        match times {
            Some((at, len)) => {
                if let Some(len) = len.filter(|l| *l > 0) {
                    let x = 4 + ((w - 12) as u64 * at.min(len) / len) as i32;
                    s.rect(x, BAR_Y - 1, 3, 3, pal.note);
                }
                let t = match len {
                    Some(len) => format!("{} / {}", clock(at), clock(len)),
                    None => clock(at),
                };
                let tw = s.width(&t, Type::Small);
                s.text(&t, Type::Small, w - 4 - tw, BAR_Y + 1, (0, w), 0, pal.line);
            }
            None if inside => {
                s.b_button(w - 14, BAR_Y + 1);
                let ew = s.width("EXIT", Type::Small);
                s.text(
                    "EXIT",
                    Type::Small,
                    w - 17 - ew,
                    BAR_Y + 1,
                    (0, w),
                    0,
                    pal.dim,
                );
            }
            None => {}
        }
    }

    /// Whether the EXIT door is out of sight: he's just gone in it, or not yet out.
    fn door_hidden(&self) -> bool {
        false
    }

    /// Whether he's on his way out of step `i` (so its door or note shows behind him).
    fn walking_away_from(&self, i: usize) -> bool {
        matches!(self.moving, Move::Leave { .. } | Move::Pipe { .. }) && i == self.cursor
    }

    /// How shut the door on step `i` is, while he goes through it.
    fn door_open_at(&self, i: usize) -> Option<i32> {
        match &self.moving {
            Move::Enter { t, .. } if i == self.cursor => {
                let half = DOOR_FRAMES / 2;
                Some(match *t < half {
                    true => 0,
                    false => ((*t - half) * 9 / half) as i32,
                })
            }
            Move::Emerge { t } if i == self.cursor => {
                let half = DOOR_FRAMES / 2;
                Some(match *t < half {
                    true => 9 - (*t * 9 / half) as i32,
                    false => 0,
                })
            }
            _ => None,
        }
    }

    /// Where he is on screen, how he stands, and how many rows of him show.
    fn guide_spot(
        &self,
        view: f32,
        dance: Pose,
        dancing: bool,
        playing_step: Option<&Path>,
    ) -> (i32, i32, Pose, i32) {
        let spot = |g: f32| self.spot(g, view);
        let exit = (self.door.0 + 1.0, self.door.1);
        let arc = |a: (f32, f32), b: (f32, f32), p: f32, high: f32| {
            let e = ease(p);
            let x = a.0 + (b.0 - a.0) * e;
            let y = a.1 + (b.1 - a.1) * e - (std::f32::consts::PI * p).sin() * high;
            (x, y)
        };
        let full = 17;
        let (pos, pose, shown) = match &self.moving {
            Move::Hop { from, to, t, len } => {
                let p = (*t as f32 / *len as f32).min(1.0);
                let (x, y) = arc(spot(*from), spot(*to), p, 5.0);
                ((x, y), Pose::Hop, full)
            }
            Move::Enter { t, .. } => {
                // He steps into the doorway and the door shuts on him.
                let shown = match *t < DOOR_FRAMES / 2 {
                    true => full,
                    false => 0,
                };
                (spot(self.cursor as f32), Pose::Stand, shown)
            }
            Move::Pipe { t } => {
                // A little hop, then down the pipe.
                let (x, y) = spot(self.cursor as f32);
                let sink = (*t as i32 - 4).clamp(0, full);
                let hop = match *t < 4 {
                    true => [0.0, 2.0, 3.0, 2.0][*t as usize],
                    false => 0.0,
                };
                ((x, y - hop + sink as f32), Pose::Hop, full - sink)
            }
            Move::Drop { t } => {
                let p = *t as f32 / DROP_FRAMES as f32;
                let (x, y) = arc(exit, spot(0.0), p, 8.0);
                let pose = if p < 0.95 { Pose::Hop } else { Pose::Stand };
                ((x, y), pose, full)
            }
            Move::Leave { t, .. } => {
                let p = *t as f32 / DROP_FRAMES as f32;
                let (x, y) = arc(spot(self.cursor as f32), exit, p, 12.0);
                ((x, y), Pose::Hop, if p < 0.9 { full } else { 0 })
            }
            Move::Emerge { t } => {
                let shown = match *t < DOOR_FRAMES / 2 {
                    true => 0,
                    false => full,
                };
                (spot(self.cursor as f32), Pose::Stand, shown)
            }
            Move::Still => {
                let on_it = self.entries.get(self.cursor).map(Entry::path) == playing_step;
                let pose = match dancing && on_it {
                    true => dance,
                    false => Pose::Stand,
                };
                (spot(self.guide), pose, full)
            }
        };
        (pos.0.round() as i32, pos.1.round() as i32, pose, shown)
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
        self.step();
        set_playing(&self.top, self.song.is_some() && !self.paused);
        self.draw();
    }

    fn reset(&mut self) {
        self.stop();
        self.moving = Move::Still;
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

    /// Through a door, waiting for him to come out the other side.
    fn through(p: &mut MusicPlayer, b: u16) {
        tap(p, b);
        run(p, (DOOR_FRAMES + DROP_FRAMES + 4) as usize);
        assert!(!p.walking());
    }

    #[test]
    fn it_browses_folders_and_plays_a_song_then_the_rest_of_its_folder() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        let names: Vec<_> = p.entries().iter().map(Entry::name).collect();
        assert_eq!(names, ["Album", "Loose"]);
        through(&mut p, ButtonMask::A);
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
        assert_eq!(p.cursor(), 1, "and he hops along with it");
        run(&mut p, 40);
        assert_eq!(p.now_playing(), None, "in order, it stops at the end");
        through(&mut p, ButtonMask::B);
        assert_eq!(p.dir(), m);
        assert_eq!(p.cursor(), 0, "back on the folder he came out of");
        through(&mut p, ButtonMask::B);
        assert_eq!(p.dir(), m, "never above the music folder");
    }

    #[test]
    fn right_climbs_left_descends_and_a_held_hop_speeds_up() {
        let d = tempfile::tempdir().unwrap();
        let m = d.path().join("Music");
        std::fs::create_dir_all(&m).unwrap();
        for i in 0..40 {
            std::fs::write(m.join(format!("{i:02}.mp3")), b"").unwrap();
        }
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        tap(&mut p, ButtonMask::RIGHT);
        assert_eq!(p.cursor(), 1);
        tap(&mut p, ButtonMask::LEFT);
        assert_eq!(p.cursor(), 0);
        tap(&mut p, ButtonMask::LEFT);
        assert_eq!(p.cursor(), 0, "the bottom step is the bottom");
        let mut hops_at = Vec::new();
        let mut last = p.cursor();
        for f in 0..120 {
            p.run_frame(ButtonMask(ButtonMask::RIGHT));
            if p.cursor() != last {
                hops_at.push(f);
                last = p.cursor();
            }
        }
        let gaps: Vec<_> = hops_at.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(gaps.len() > 6);
        assert!(
            gaps[1] > *gaps.last().unwrap(),
            "faster as it's held: {gaps:?}"
        );
        assert!(
            gaps.windows(2).all(|w| w[1] <= w[0]),
            "never slower: {gaps:?}"
        );
        tap(&mut p, ButtonMask::DOWN);
        assert_eq!(p.cursor(), last - PAGE as usize, "Down goes a few at once");
    }

    #[test]
    fn a_playlist_plays_its_songs_in_its_order() {
        let (_d, m) = card();
        std::fs::write(m.join("Mix.m3u"), "Album/02 Second.wav\nLoose.wav\n").unwrap();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        let names: Vec<_> = p.entries().iter().map(Entry::name).collect();
        assert_eq!(names, ["Album", "Mix", "Loose"]);
        tap(&mut p, ButtonMask::RIGHT);
        run(&mut p, 10);
        tap(&mut p, ButtonMask::A);
        assert_eq!(
            p.now_playing(),
            Some(m.join("Album/02 Second.wav").as_path())
        );
        let save = p.save_ram().unwrap();
        run(&mut p, 40);
        assert_eq!(p.now_playing(), Some(m.join("Loose.wav").as_path()));
        assert_eq!(p.cursor(), 1, "he stays on the playlist's step");

        let mut again = MusicPlayer::new();
        again.load(&m).unwrap();
        again.load_save_ram(&save).unwrap();
        assert_eq!(
            again.now_playing(),
            Some(m.join("Album/02 Second.wav").as_path())
        );
        assert_eq!(again.cursor(), 1, "back on the playlist");
    }

    #[test]
    fn start_pauses_and_the_save_picks_up_where_it_was() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        through(&mut p, ButtonMask::A);
        tap(&mut p, ButtonMask::RIGHT);
        run(&mut p, 10);
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

        // A save from before playlists still picks up.
        let old = format!("{SAVE_MAGIC_1}\nAlbum/01 First.wav\n100\nshuffle\n");
        let mut older = MusicPlayer::new();
        older.load(&m).unwrap();
        older.load_save_ram(old.as_bytes()).unwrap();
        assert_eq!(
            older.now_playing(),
            Some(m.join("Album/01 First.wav").as_path())
        );
        assert_eq!(older.order(), Order::Shuffle);
    }

    #[test]
    fn repeat_one_plays_the_same_song_again_and_a_tap_on_r_or_l_skips() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        through(&mut p, ButtonMask::A);
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
        tap(&mut p, ButtonMask::RIGHT);
        run(&mut p, 10);
        tap(&mut p, ButtonMask::A);
        assert!(p.trouble.as_deref() == Some("Broken"));
        assert_eq!(run(&mut p, 3).len(), 3 * 735 * 2);
    }

    #[test]
    fn b_pauses_then_stops_then_leaves_the_folder() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        through(&mut p, ButtonMask::A);
        tap(&mut p, ButtonMask::A);
        assert!(p.now_playing().is_some() && !p.paused());
        tap(&mut p, ButtonMask::B);
        assert!(p.paused(), "the first B pauses");
        assert!(p.now_playing().is_some());
        tap(&mut p, ButtonMask::B);
        assert_eq!(p.now_playing(), None, "the second stops");
        assert_eq!(p.dir(), m.join("Album"), "and he's still in the folder");
        through(&mut p, ButtonMask::B);
        assert_eq!(p.dir(), m, "the third takes him out");
    }

    #[test]
    fn the_warp_pipe_at_the_top_of_a_folder_goes_back_to_the_top_of_the_music() {
        let (_d, m) = card();
        std::fs::create_dir_all(m.join("Album/Disc 2")).unwrap();
        wav(&m.join("Album/Disc 2/01 Third.wav"), 44_100, 2, 0.5);
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        assert!(!p.has_pipe(), "no pipe at the top");
        tap(&mut p, ButtonMask::RIGHT);
        run(&mut p, 10);
        assert_eq!(p.cursor(), 1, "the top has only its two steps");
        tap(&mut p, ButtonMask::LEFT);
        run(&mut p, 10);
        through(&mut p, ButtonMask::A);
        through(&mut p, ButtonMask::A);
        assert_eq!(p.dir(), m.join("Album/Disc 2"));
        tap(&mut p, ButtonMask::RIGHT);
        run(&mut p, 10);
        assert!(p.on_pipe(), "the step after the last song is the pipe");
        tap(&mut p, ButtonMask::RIGHT);
        run(&mut p, 10);
        assert!(p.on_pipe(), "and it's the top");
        through(&mut p, ButtonMask::A);
        assert_eq!(p.dir(), m, "down the pipe to the top of the music");
        assert_eq!(p.cursor(), 0, "out of the door of the folder he was under");
    }

    #[test]
    fn select_tapped_changes_the_order_and_held_changes_the_theme() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        tap(&mut p, ButtonMask::SELECT);
        assert_eq!(p.order(), Order::RepeatAll);
        assert_eq!(p.theme(), Theme::Classic);
        for _ in 0..THEME_HOLD + 5 {
            p.run_frame(ButtonMask(ButtonMask::SELECT));
        }
        p.run_frame(ButtonMask(0));
        assert_eq!(p.theme(), Theme::Monochrome);
        assert_eq!(
            p.order(),
            Order::RepeatAll,
            "a hold doesn't change the order"
        );
        assert_eq!(p.screen.pal, MONOCHROME);
        let mut again = MusicPlayer::new();
        again.load(&m).unwrap();
        again.load_save_ram(&p.save_ram().unwrap()).unwrap();
        assert_eq!(again.theme(), Theme::Monochrome, "the cart remembers it");
    }

    #[test]
    fn the_exit_door_follows_him_and_flies_to_its_corner_while_a_song_plays() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        through(&mut p, ButtonMask::A);
        run(&mut p, 60);
        let near = p.door;
        tap(&mut p, ButtonMask::RIGHT);
        run(&mut p, 60);
        assert!(
            p.door.0 > near.0 && p.door.1 < near.1,
            "up the stairs with him"
        );
        tap(&mut p, ButtonMask::A);
        run(&mut p, 25);
        assert!((p.door.0 - DOOR_CORNER.0).abs() < 2.0 && (p.door.1 - DOOR_CORNER.1).abs() < 2.0);
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
