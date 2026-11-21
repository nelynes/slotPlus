//! slotPlus music cart: the player, run as a core of slotPlus's own, so the cart goes in,
//! comes out, pauses and saves like any other.
//!
//! In the spirit of the music carts sold for the GBA: each file is a step on a staircase,
//! folders are doors and songs are notes (playlists two), and a little cart climbs it.
//! Left and Right hop him down and up a step (faster the longer they're held), Up and Down
//! a few at once, A plays a song (and the rest of its folder or playlist after it) or takes
//! him through a door, B takes him back out of the EXIT door, START pauses, L and R skip
//! back and on (held, they wind ten seconds at a time), and SELECT changes the order: in
//! order, repeat all, repeat one, shuffle. While a song plays he dances on its step, and Up
//! and Down are the handheld's volume (slotPlus takes them before they reach the player).
//!
//! What it was playing, and where, is kept as the cart's save, so it picks up there.

use std::path::{Path, PathBuf};

use slot_retro::{AvInfo, ButtonMask, CoreError, RetroCore};

use super::decode::{Song, OUT_RATE};
use super::library::{self, Entry};
use super::screen::{
    self, clock, Pose, Screen, Type, BACK, DIM, INK, LINE, NOTE, PICK, PICK_STRIPE,
};

const FPS: f64 = 60.0;

/// The staircase: each step `STEP` across and `ROW` up from the one before, the first with
/// its tread at `BASE`. `VISIBLE` steps fit under the band along the top.
const ROW: i32 = 17;
const STEP: i32 = 17;
const X0: i32 = 2;
const BASE: i32 = 158;
const VISIBLE: usize = 7;
const BAND: i32 = 34;
/// Where the EXIT door stands, in the band.
const EXIT_X: i32 = 8;
const EXIT_FLOOR: i32 = 24;

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
        let top_room = (self.cursor + 2).min(self.entries.len());
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if top_room > self.scroll + VISIBLE {
            self.scroll = top_room - VISIBLE;
        }
    }

    /// Hop `by` steps, as far as the stairs go.
    fn hop(&mut self, by: i32) {
        let n = self.entries.len();
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
            None => {}
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
            return self.back();
        }
        if pressed(ButtonMask::START) {
            self.toggle_pause();
        }
        if pressed(ButtonMask::SELECT) {
            self.order = self.order.next();
            self.keep();
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
                    self.moving = Move::Drop { t: 0 };
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
            "{SAVE_MAGIC}\n{source}\n{index}\n{at}\n{}\n",
            self.order.word()
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
                (source, index, at, lines.next().unwrap_or_default())
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
        let playing_step = self.playing_step().map(Path::to_path_buf);
        let dancing = self.song.is_some() && !self.paused;
        let pose_now = match (frame / DANCE_FRAMES).is_multiple_of(2) {
            true => Pose::DanceLeft,
            false => Pose::DanceRight,
        };
        let view = self.view;
        let first = (view.floor() as isize - 1).max(0) as usize;
        let last = (view.ceil() as usize + VISIBLE + 1).min(self.entries.len());
        // What each step on screen shows: him in front of it, or its door part-shut.
        let marks: Vec<(bool, Option<i32>)> = (first..last)
            .map(|i| {
                let here = (self.guide - i as f32).abs() < 0.5 && !self.walking_away_from(i);
                (here, self.door_open_at(i))
            })
            .collect();
        let entries = std::mem::take(&mut self.entries);
        let s = &mut self.screen;
        s.fill(BACK);

        if entries.is_empty() {
            s.text(
                "No music here",
                Type::Title,
                40,
                80,
                (0, screen::W as i32),
                0,
                LINE,
            );
        }
        // The stairs, bottom to top, with what each step holds.
        for i in first..last {
            let k = i as f32 - view;
            let (xs, yb) = step_at(k);
            let (xs, yb) = (xs.round() as i32, yb.round() as i32);
            let xt = xs + STEP;
            s.hline(xs, screen::W as i32 - 1, yb, LINE);
            s.vline(xt, yb - ROW, yb, LINE);
            if i + 1 == entries.len() {
                s.hline(xt, screen::W as i32 - 1, yb - ROW, LINE);
            }
            let here = i == self.cursor;
            if here {
                s.rect(
                    xt + 1,
                    yb - ROW + 1,
                    screen::W as i32 - xt - 1,
                    ROW - 1,
                    PICK,
                );
                let mut x = xt + 1;
                while x < screen::W as i32 {
                    s.vline(x, yb - ROW + 1, yb - 1, PICK_STRIPE);
                    x += 12;
                }
            }
            let e = &entries[i];
            let lit = playing_step.as_deref() == Some(e.path());
            // A step's own mark, unless he's standing in front of it.
            let (guide_here, open) = marks[i - first];
            match e {
                Entry::Folder { .. } => match open {
                    Some(shut) => s.doorway(xs + 4, yb, shut, LINE),
                    None if !guide_here => s.door(xs + 4, yb, LINE),
                    None => s.doorway(xs + 4, yb, 0, LINE),
                },
                Entry::Song { .. } if !guide_here => {
                    let bob = if lit && dancing && (frame / 15).is_multiple_of(2) {
                        1
                    } else {
                        0
                    };
                    s.note(xs + 6, yb - 1 - bob, NOTE)
                }
                Entry::Playlist { .. } if !guide_here => s.notes(xs + 4, yb - 1, NOTE),
                _ => {}
            }
            let ink = if here { INK } else { LINE };
            let name = e.name().to_string();
            s.text(
                &name,
                Type::Title,
                xt + 3,
                yb - ROW - 3,
                (xt + 1, screen::W as i32 - 1),
                0,
                ink,
            );
        }

        self.entries = entries;
        // Him.
        let (gx, gy, pose, shown) =
            self.guide_spot(view, pose_now, dancing, playing_step.as_deref());
        if shown > 0 {
            self.screen.guide(gx, gy, pose, shown);
        }

        // The band along the top: the EXIT door, and what's playing.
        let inside = self.dir != self.top;
        let now = self.now_playing().map(Path::to_path_buf);
        let place = match self.dir.strip_prefix(&self.top) {
            Ok(rel) if !rel.as_os_str().is_empty() => rel
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            _ => "Music".to_string(),
        };
        let s = &mut self.screen;
        s.rect(0, 0, screen::W as i32, BAND + 4, BACK);
        if inside {
            s.exit_door(EXIT_X, EXIT_FLOOR);
        }
        let left = 32;
        let right = screen::W as i32 - 4;
        match (&self.song, now) {
            (Some(song), Some(path)) => {
                let title = song.tags.title.clone().unwrap_or_else(|| {
                    path.file_stem()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
                let w = s.width(&title, Type::Title);
                let room = right - left;
                // A long title scrolls by, after a moment's pause at the start.
                let skip = match w > room {
                    true => {
                        let span = (w + 24) as u64;
                        let t = (frame / 2) % (span + 60);
                        t.saturating_sub(60) as i32
                    }
                    false => 0,
                };
                s.text(&title, Type::Title, left, -1, (left, right), skip, INK);
                if skip > 0 {
                    s.text(
                        &title,
                        Type::Title,
                        left + w + 24,
                        -1,
                        (left, right),
                        skip,
                        INK,
                    );
                }
                let (at, len) = (song.position_ms(), song.length_ms);
                s.rect(left, 18, room - 40, 3, DIM);
                if let Some(len) = len.filter(|l| *l > 0) {
                    let fill = ((room - 40) as u64 * at.min(len) / len) as i32;
                    s.rect(left, 18, fill, 3, NOTE);
                }
                if self.paused {
                    s.rect(right - 30, 16, 2, 7, LINE);
                    s.rect(right - 26, 16, 2, 7, LINE);
                }
                let times = match len {
                    Some(len) => format!("{} / {}", clock(at), clock(len)),
                    None => clock(at),
                };
                s.text(&times, Type::Small, left, 22, (left, right), 0, LINE);
            }
            _ => {
                let say = match &self.trouble {
                    Some(name) => format!("Can't play {name}"),
                    None => place,
                };
                s.text(&say, Type::Title, left, -1, (left, right), 0, DIM);
            }
        }
        let order = self.order.text();
        let ow = s.width(order, Type::Small);
        s.text(
            order,
            Type::Small,
            right - ow,
            22,
            (left, right + 1),
            0,
            DIM,
        );
        if inside {
            s.b_button(screen::W as i32 - 13, screen::H as i32 - 13);
        }
    }

    /// Whether he's on his way out of step `i` (so its door or note shows behind him).
    fn walking_away_from(&self, i: usize) -> bool {
        matches!(self.moving, Move::Leave { .. }) && i == self.cursor
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
        let spot = |g: f32| {
            let (xs, yb) = step_at(g - view);
            (xs + 5.0, yb)
        };
        let exit = (EXIT_X as f32 + 1.0, EXIT_FLOOR as f32);
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
                let half = DOOR_FRAMES / 2;
                let shown = match *t < half {
                    true => full,
                    false => 0,
                };
                (spot(self.cursor as f32), Pose::Stand, shown)
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
    fn a_save_from_something_else_is_left_alone() {
        let (_d, m) = card();
        let mut p = MusicPlayer::new();
        p.load(&m).unwrap();
        p.load_save_ram(&[0u8; 32]).unwrap();
        assert_eq!(p.now_playing(), None);
        assert!(p.unserialize(&[1, 2, 3]).is_err());
    }
}
