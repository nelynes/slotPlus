//! Optional, softcore RetroAchievements. Emulator threads only copy RAM into a bounded,
//! nonblocking queue; evaluation/storage and HTTPS each have their own worker.
mod badges;
mod core;
mod library;
mod network;
mod ready;
mod played;
mod runtime;
mod scan;
mod settings;
mod storage;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

pub use badges::{load_badge, BadgeImage};
pub use ready::Readiness;
pub use settings::{save, Change, Settings};
use slot_retro::{AchievementMemory, RetroCore};
use storage::{Config, Store, Unlock};

const RAM_SIZE: usize = slot_retro::ACHIEVEMENT_RAM_SIZE;
const QUEUE_SIZE: usize = 8;

/// A passive shelf indicator. Reading it never acquires a worker's lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SyncStatus {
    Disabled,
    Syncing,
    Ready,
    Offline,
    Attention,
}

#[derive(Default)]
struct Status {
    network: AtomicU8,
    unsaved: AtomicBool,
    reconnect: AtomicBool,
    // Zero means unknown; otherwise percent + 1, keeping Default valid.
    progress: AtomicU8,
    pending: AtomicBool,
    // Bumped after each game cache write, so the shelf knows when to look again.
    cached: AtomicU64,
    // Shared only by the evaluator and HTTP workers, never gameplay/UI.
    presence: Mutex<Option<(u64, String)>>,
    // slotPlus: Download Achievement Sets: asked for, asked to stop, and how it's going.
    scan_request: AtomicBool,
    scan_stop: AtomicBool,
    scan: Mutex<ScanReport>,
    // slotPlus: the ROMs with a downloaded set, for the shelf's trophy, and a count bumped
    // whenever that list changes.
    sets: Mutex<Arc<HashSet<PathBuf>>>,
    sets_changed: AtomicU64,
    // slotPlus: the game just loaded has its set active; the game shows "Achievements
    // Ready" once.
    set_ready: AtomicBool,
}

/// slotPlus: where a Download Achievement Sets run is up to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScanState {
    /// Not started (or the service isn't running at all).
    #[default]
    Idle,
    SigningIn,
    Running,
    Done,
    Stopped,
    /// The connection dropped part way.
    Offline,
    /// The server turned the account down.
    SignInFailed,
    /// No account is set up, so there is nothing to sign in with.
    NoAccount,
    /// The card couldn't be written.
    CardError,
}

impl ScanState {
    pub fn running(self) -> bool {
        matches!(self, ScanState::SigningIn | ScanState::Running)
    }
}

/// slotPlus: the counts Download Achievement Sets shows while it runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScanReport {
    pub state: ScanState,
    /// The ROM being looked at, by file name.
    pub game: String,
    /// The system being scanned (from 1), and how many have games on the card.
    pub system: usize,
    pub systems: usize,
    /// Sets downloaded this time, and ones already on the card.
    pub new: usize,
    pub already: usize,
    /// ROMs the server has no achievements for.
    pub none: usize,
    /// ROMs that couldn't be read or looked up.
    pub failed: usize,
    /// Badge images downloaded this time.
    pub badges: usize,
}

impl Status {
    fn publish_sets(&self, sets: HashSet<PathBuf>) {
        *self.sets.lock().unwrap_or_else(|e| e.into_inner()) = Arc::new(sets);
        self.sets_changed.fetch_add(1, Ordering::AcqRel);
    }

    fn add_set(&self, path: &Path) {
        self.mark_set(path, true);
    }

    /// The shelf's trophy for `path`, on or off.
    fn mark_set(&self, path: &Path, has_set: bool) {
        let mut sets = self.sets.lock().unwrap_or_else(|e| e.into_inner());
        if sets.contains(path) != has_set {
            let sets = Arc::make_mut(&mut sets);
            match has_set {
                true => sets.insert(path.to_path_buf()),
                false => sets.remove(path),
            };
            self.sets_changed.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn set(&self, value: SyncStatus) {
        self.network.store(value as u8, Ordering::Release);
    }

    fn get(&self) -> SyncStatus {
        if self.unsaved.load(Ordering::Acquire) {
            return SyncStatus::Attention;
        }
        match self.network.load(Ordering::Acquire) {
            0 => SyncStatus::Disabled,
            1 => SyncStatus::Syncing,
            2 => SyncStatus::Ready,
            3 => SyncStatus::Offline,
            _ => SyncStatus::Attention,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeKind {
    Status,
    Earned,
}

#[derive(Clone, Debug)]
pub struct Notice {
    /// Local cached image, read only by the notification raster worker.
    pub badge: Option<PathBuf>,
    pub generation: u64,
    pub title: String,
    pub detail: String,
    pub kind: NoticeKind,
}

impl Notice {
    fn status(generation: u64, title: &str) -> Self {
        // Background diagnostics only; routine status is represented by the shelf icon.
        eprintln!("slot: {title}");
        Self {
            badge: None,
            generation,
            title: title.into(),
            detail: String::new(),
            kind: NoticeKind::Status,
        }
    }
}

enum Control {
    Load(network::Load),
    Unload(u64),
    Flush(Arc<AtomicBool>),
}

struct Frame {
    generation: u64,
    sequence: u64,
    timeline: u64,
    earned_at: u64,
    ram: Vec<u8>,
    valid: AchievementMemory,
    recycle: mpsc::SyncSender<Vec<u8>>,
}

/// Outlives games, so pending unlocks sync while the player is on the shelf, too.
pub struct Service {
    controls: mpsc::Sender<Control>,
    frames: mpsc::SyncSender<Frame>,
    notices: mpsc::Receiver<Notice>,
    enabled: Arc<AtomicBool>,
    current: Arc<AtomicU64>,
    // slotPlus: the game whose achievements are active; RAM is copied only for it.
    tracking: Arc<AtomicU64>,
    flushing: AtomicBool,
    flushed: Arc<AtomicBool>,
    status: Arc<Status>,
}

impl Service {
    pub fn start(root: PathBuf) -> Self {
        Self::start_with(root, network::Http::new())
    }

    fn start_with(root: PathBuf, transport: impl network::Transport + Send + 'static) -> Self {
        let (controls, control_rx) = mpsc::channel();
        let (frames, frame_rx) = mpsc::sync_channel(QUEUE_SIZE);
        let (notice_tx, notices) = mpsc::channel();
        let enabled = Arc::new(AtomicBool::new(false));
        let current = Arc::new(AtomicU64::new(0));
        let tracking = Arc::new(AtomicU64::new(0));
        let worker_tracking = tracking.clone();
        let flushed = Arc::new(AtomicBool::new(false));
        let worker_flushed = flushed.clone();
        let worker_enabled = enabled.clone();
        let status = Arc::new(Status::default());
        let worker_status = status.clone();
        let spawned = std::thread::Builder::new()
            .name("slot-ra".into())
            .spawn(move || {
                run(
                    root,
                    worker_enabled.clone(),
                    worker_tracking,
                    control_rx,
                    frame_rx,
                    notice_tx,
                    worker_status,
                    transport,
                );
                worker_enabled.store(false, Ordering::Release);
                worker_flushed.store(true, Ordering::Release);
            });
        if spawned.is_err() {
            status.set(SyncStatus::Attention);
            flushed.store(true, Ordering::Release);
            eprintln!("slot: achievements worker could not start");
        }
        Self {
            controls,
            frames,
            notices,
            enabled,
            current,
            tracking,
            flushing: AtomicBool::new(false),
            flushed,
            status,
        }
    }

    /// slotPlus: a service that does nothing: no worker, no network, nothing read from the
    /// card. Used while RetroAchievements is switched off in this build.
    pub fn off() -> Self {
        let (controls, _) = mpsc::channel();
        let (frames, _) = mpsc::sync_channel(QUEUE_SIZE);
        let (_, notices) = mpsc::channel();
        Self {
            controls,
            frames,
            notices,
            enabled: Arc::new(AtomicBool::new(false)),
            current: Arc::new(AtomicU64::new(0)),
            tracking: Arc::new(AtomicU64::new(0)),
            flushing: AtomicBool::new(false),
            flushed: Arc::new(AtomicBool::new(true)),
            status: Arc::new(Status::default()),
        }
    }

    pub fn sync_progress(&self) -> Option<u8> {
        self.status.progress.load(Ordering::Acquire).checked_sub(1)
    }

    pub fn sync_pending(&self) -> bool {
        self.status.pending.load(Ordering::Acquire) || self.status.unsaved.load(Ordering::Acquire)
    }

    /// Changes whenever a game's achievement cache is written. See [`Readiness`].
    pub fn cache_generation(&self) -> u64 {
        self.status.cached.load(Ordering::Acquire)
    }

    pub fn sync_status(&self) -> SyncStatus {
        self.status.get()
    }

    /// slotPlus: start Download Achievement Sets. It runs on the network worker while slot
    /// sits on the shelf; [`Service::scan_report`] says how it's going.
    pub fn start_scan(&self) {
        let state = match self.enabled.load(Ordering::Acquire) {
            true => ScanState::SigningIn,
            false => ScanState::NoAccount,
        };
        *self.status.scan.lock().unwrap_or_else(|e| e.into_inner()) = ScanReport {
            state,
            ..ScanReport::default()
        };
        self.status.scan_stop.store(false, Ordering::Release);
        self.status
            .scan_request
            .store(state == ScanState::SigningIn, Ordering::Release);
    }

    /// slotPlus: stop Download Achievement Sets after the request in hand. What it has
    /// saved so far stays.
    pub fn stop_scan(&self) {
        self.status.scan_stop.store(true, Ordering::Release);
        // Not picked up yet: it never starts.
        if self.status.scan_request.swap(false, Ordering::AcqRel) {
            self.status.scan.lock().unwrap_or_else(|e| e.into_inner()).state = ScanState::Stopped;
        }
    }

    pub fn scan_report(&self) -> ScanReport {
        self.status.scan.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// slotPlus: the ROMs with a set on the card, and a count that changes with the list.
    pub fn achievement_sets(&self) -> (u64, Arc<HashSet<PathBuf>>) {
        (
            self.status.sets_changed.load(Ordering::Acquire),
            self.status.sets.lock().unwrap_or_else(|e| e.into_inner()).clone(),
        )
    }

    /// slotPlus: true once after a game's achievement set becomes active.
    pub fn take_set_ready(&self) -> bool {
        self.status.set_ready.swap(false, Ordering::AcqRel)
    }

    /// The Wi-Fi worker observed a new connection. Wake retries without blocking the UI.
    pub fn network_available(&self) {
        self.status.reconnect.store(true, Ordering::Release);
    }

    /// Poll during power-off. The UI keeps rendering while the evaluator drains final
    /// frames and fsyncs their awards; an HTTP request is never part of this barrier.
    pub fn flush_ready(&self) -> bool {
        if !self.flushing.swap(true, Ordering::AcqRel)
            && self
                .controls
                .send(Control::Flush(self.flushed.clone()))
                .is_err()
        {
            self.flushed.store(true, Ordering::Release);
        }
        self.flushed.load(Ordering::Acquire)
    }

    pub fn wrap(&self, core: Box<dyn RetroCore>) -> Box<dyn RetroCore> {
        Box::new(core::Tracked::new(core, self))
    }

    pub fn take_notice(&self) -> Option<Notice> {
        while let Ok(notice) = self.notices.try_recv() {
            // Earned notices survive an eject; stale loading notices do not.
            if notice.kind == NoticeKind::Earned
                || notice.generation == 0
                || notice.generation == self.current.load(Ordering::Acquire)
            {
                return Some(notice);
            }
        }
        None
    }
}

struct Playing {
    generation: u64,
    hash: String,
    runtime: runtime::Runtime,
    achievements: BTreeMap<u32, storage::Achievement>,
    /// Encore: active again though already unlocked, so earning one is shown, not recorded.
    encore: std::collections::BTreeSet<u32>,
    previous: Option<(u64, u64)>,
    warned_gap: bool,
}

#[allow(clippy::too_many_arguments)]
fn run(
    root: PathBuf,
    enabled: Arc<AtomicBool>,
    tracking: Arc<AtomicU64>,
    controls: mpsc::Receiver<Control>,
    frames: mpsc::Receiver<Frame>,
    notices: mpsc::Sender<Notice>,
    status: Arc<Status>,
    transport: impl network::Transport + Send + 'static,
) {
    let config_path = root.join("Config/retroachievements.toml");
    if !config_path.exists() {
        return;
    }
    let config = match Config::read(&config_path) {
        Ok(config) => config,
        Err(_) => {
            status.set(SyncStatus::Attention);
            let _ = notices.send(Notice::status(0, "Invalid RetroAchievements config"));
            return;
        }
    };
    if !config.enabled {
        return;
    }
    if config.username.trim().is_empty() {
        status.set(SyncStatus::Attention);
        let _ = notices.send(Notice::status(0, "Achievements need an account"));
        return;
    }
    let encore = config.encore;
    let store = match Store::open(&root, &config.username) {
        Ok(store) => Arc::new(Mutex::new(store)),
        Err(error) => {
            status.set(SyncStatus::Attention);
            let _ = notices.send(Notice::status(0, &error));
            return;
        }
    };
    let (loads, load_rx) = mpsc::channel();
    let (prepared_tx, prepared_rx) = mpsc::channel();
    let network_store = store.clone();
    let network_notices = notices.clone();
    let network_status = status.clone();
    status.set(SyncStatus::Syncing);
    if std::thread::Builder::new()
        .name("slot-ra-http".into())
        .spawn(move || {
            network::run(
                root,
                config,
                network_store,
                load_rx,
                network_notices,
                network_status,
                transport,
            );
        })
        .is_err()
    {
        status.set(SyncStatus::Attention);
        let _ = notices.send(Notice::status(0, "Achievements network worker failed"));
        return;
    }
    enabled.store(true, Ordering::Release);
    let mut generation = 0;
    let mut playing: Option<Playing> = None;
    let mut unsaved = BTreeMap::new();
    let mut retried = Instant::now();
    let mut presence_updated = Instant::now();
    loop {
        if retried.elapsed() >= Duration::from_secs(1) {
            retry_unsaved(&mut unsaved, &store, &notices);
            retried = Instant::now();
        }
        loop {
            // Frames queued before an eject/load must be evaluated before retiring that game.
            for _ in 0..QUEUE_SIZE {
                let Ok(frame) = frames.try_recv() else {
                    break;
                };
                evaluate(frame, &mut playing, &store, &notices, &mut unsaved);
            }
            match controls.try_recv() {
                Ok(Control::Flush(done)) => {
                    retry_unsaved(&mut unsaved, &store, &notices);
                    library::flush();
                    done.store(true, Ordering::Release);
                }
                Ok(Control::Load(load)) => {
                    generation = load.generation;
                    status.set_ready.store(false, Ordering::Release);
                    playing = None;
                    tracking.store(0, Ordering::Release);
                    *status.presence.lock().unwrap() = None;
                    // A game earns from the set Download Achievement Sets saved; nothing is
                    // fetched for it while it plays.
                    let dir = store.lock().unwrap().dir.clone();
                    if let Ok(hash) = library::hash(&dir, &load.path) {
                        // This may have indexed a renamed ROM; the shelf can seal it.
                        status.cached.fetch_add(1, Ordering::AcqRel);
                        let name = load
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        let cached = storage::read::<(
                            storage::Game,
                            std::collections::BTreeSet<u32>,
                        )>(&dir.join(format!("{hash}.json")));
                        match &cached {
                            Err(_) => eprintln!(
                                "slot: achievements: no set on the card for {name} \
                                 (Download Achievement Sets looks it up)"
                            ),
                            Ok((game, _)) if game.stand_in() => eprintln!(
                                "slot: achievements: RetroAchievements doesn't support this \
                                 version of {name} ({})",
                                game.title
                            ),
                            Ok((game, _)) if game.id == 0 => eprintln!(
                                "slot: achievements: RetroAchievements has no game for {name}"
                            ),
                            Ok(_) => {}
                        }
                        if let Ok((game, unlocked)) = cached {
                            // The cached set has the last word on the trophy: a renamed ROM
                            // gets it back, and a stand-in or notice-only set loses it.
                            let found = scan::has_set(&game);
                            library::checked(&dir, &load.path, network::now(), found);
                            status.mark_set(&load.path, found);
                            if game.id != 0 && !game.stand_in() {
                                // Reported from the shelf, so it shows as played.
                                played::push(
                                    &dir,
                                    played::Played {
                                        game: game.id,
                                        hash: hash.clone(),
                                    },
                                );
                            }
                            if game.supported_console() {
                                let _ = prepared_tx.send(network::Prepared {
                                    generation,
                                    hash,
                                    game,
                                    unlocked,
                                    cached: true,
                                });
                            }
                        }
                    }
                    // Only whether a game is in play: the network waits for the shelf.
                    let _ = loads.send(true);
                }
                Ok(Control::Unload(id)) if id == generation => {
                    generation = 0;
                    status.set_ready.store(false, Ordering::Release);
                    playing = None;
                    tracking.store(0, Ordering::Release);
                    *status.presence.lock().unwrap() = None;
                    let _ = loads.send(false);
                }
                Ok(_) => {}
                Err(mpsc::TryRecvError::Disconnected) => return,
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
        for prepared in prepared_rx.try_iter() {
            if prepared.generation != generation {
                continue;
            }
            if let Some(game) = playing.as_mut() {
                // Preserve hits for unchanged definitions. Retire removed/unlocked entries
                // and activate new or revised definitions when online data arrives.
                game.runtime.activate_presence(&prepared.game.presence);
                let store = store.lock().unwrap();
                let mut updated = BTreeMap::new();
                let mut again = std::collections::BTreeSet::new();
                let real = prepared.game.id != 0 && !prepared.game.stand_in();
                for achievement in prepared.game.achievements {
                    if !real || achievement.flags != 3 || achievement.id >= storage::NOTICE_IDS {
                        continue;
                    }
                    let done = prepared.unlocked.contains(&achievement.id)
                        || store.unlocks.contains_key(&achievement.id)
                        || unsaved.contains_key(&achievement.id);
                    if done && !encore {
                        continue;
                    }
                    if done {
                        again.insert(achievement.id);
                    }
                    let unchanged = game
                        .achievements
                        .get(&achievement.id)
                        .is_some_and(|old| old.definition == achievement.definition);
                    if unchanged
                        || game
                            .runtime
                            .activate(achievement.id, &achievement.definition)
                    {
                        updated.insert(achievement.id, achievement);
                    }
                }
                for id in game
                    .achievements
                    .keys()
                    .filter(|id| !updated.contains_key(id))
                {
                    game.runtime.deactivate(*id);
                }
                game.achievements = updated;
                game.encore = again;
                continue;
            }
            let Some(mut runtime) = runtime::Runtime::new() else {
                continue;
            };
            runtime.activate_presence(&prepared.game.presence);
            let store = store.lock().unwrap();
            let mut achievements = BTreeMap::new();
            let mut again = std::collections::BTreeSet::new();
            let mut unsupported = 0;
            let mut set_aside = 0;
            let mut earned_before = 0;
            let real = prepared.game.id != 0 && !prepared.game.stand_in();
            let game_id = prepared.game.id;
            let game_title = prepared.game.title.clone();
            for achievement in prepared.game.achievements {
                // Unofficial ones, and RetroAchievements' own notices, can't be earned.
                if !real || achievement.flags != 3 || achievement.id >= storage::NOTICE_IDS {
                    set_aside += 1;
                    continue;
                }
                let done = prepared.unlocked.contains(&achievement.id)
                    || store.unlocks.contains_key(&achievement.id);
                if done {
                    earned_before += 1;
                }
                if done && !encore {
                    continue;
                }
                if done {
                    again.insert(achievement.id);
                }
                if runtime.activate(achievement.id, &achievement.definition) {
                    achievements.insert(achievement.id, achievement);
                } else {
                    unsupported += 1;
                }
            }
            drop(store);
            let state = if prepared.cached { "cached" } else { "online" };
            eprintln!(
                "slot: achievements: game {game_id} \"{game_title}\": {} active, {earned_before} \
                 earned before{}, {set_aside} unofficial or notices, {unsupported} unsupported",
                achievements.len(),
                if encore { " (encore)" } else { "" },
            );
            let title = if !real {
                "No achievements for this ROM".into()
            } else {
                let encore = if encore { ", encore" } else { "" };
                format!("Achievements ready ({state}, softcore{encore})")
            };
            let detail = if unsupported == 0 {
                prepared.game.title
            } else {
                format!("{} - {unsupported} unsupported", prepared.game.title)
            };
            let _ = notices.send(Notice {
                badge: None,
                generation,
                title,
                detail,
                kind: NoticeKind::Status,
            });
            playing = Some(Playing {
                generation,
                hash: prepared.hash,
                runtime,
                achievements,
                encore: again,
                previous: None,
                warned_gap: false,
            });
            tracking.store(generation, Ordering::Release);
            if real && !playing.as_ref().is_some_and(|g| g.achievements.is_empty()) {
                status.set_ready.store(true, Ordering::Release);
            }
        }
        match frames.recv_timeout(Duration::from_millis(5)) {
            Ok(frame) => {
                evaluate(frame, &mut playing, &store, &notices, &mut unsaved);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if presence_updated.elapsed() >= Duration::from_secs(1) {
            *status.presence.lock().unwrap() = playing
                .as_ref()
                .map(|game| (game.generation, game.runtime.presence()));
            presence_updated = Instant::now();
        }
        status.unsaved.store(!unsaved.is_empty(), Ordering::Release);
    }
}

fn evaluate(
    frame: Frame,
    playing: &mut Option<Playing>,
    store: &Arc<Mutex<Store>>,
    notices: &mpsc::Sender<Notice>,
    unsaved: &mut BTreeMap<u32, (Unlock, Notice)>,
) {
    if let Some(game) = playing
        .as_mut()
        .filter(|g| g.generation == frame.generation)
    {
        if let Some((sequence, timeline)) = game.previous {
            if timeline != frame.timeline || sequence + 1 != frame.sequence {
                game.runtime.reset();
                if timeline == frame.timeline && !game.warned_gap {
                    let _ = notices.send(Notice::status(
                        game.generation,
                        "Achievement tracking interrupted",
                    ));
                    game.warned_gap = true;
                }
            }
        }
        game.previous = Some((frame.sequence, frame.timeline));
        if frame.valid.regions().is_empty() {
            if !game.warned_gap {
                let _ = notices.send(Notice::status(
                    game.generation,
                    "Core does not expose achievement RAM",
                ));
                game.warned_gap = true;
            }
        } else {
            let earned = game.runtime.frame(&frame.ram, &frame.valid).to_vec();
            for id in earned {
                let unlock = Unlock {
                    id,
                    hash: game.hash.clone(),
                    earned_at: frame.earned_at,
                    synced: false,
                };
                if let Some(achievement) = game.achievements.get(&id) {
                    let notice = Notice {
                        badge: badges::path(&store.lock().unwrap().dir, &achievement.badge),
                        generation: game.generation,
                        title: format!("{} (+{})", achievement.title, achievement.points),
                        detail: achievement.description.clone(),
                        kind: NoticeKind::Earned,
                    };
                    let how = match game.encore.contains(&id) {
                        true => "again (encore, not sent)",
                        false => "and saved for the next sync",
                    };
                    eprintln!(
                        "slot: achievements: earned {id} \"{}\" {how}",
                        achievement.title
                    );
                    if game.encore.remove(&id) {
                        // Encore: already on the account, so shown and nothing more.
                        let _ = notices.send(notice);
                        game.runtime.deactivate(id);
                        continue;
                    }
                    match store.lock().unwrap().record(unlock.clone()) {
                        Ok(true) => {
                            let _ = notices.send(notice);
                        }
                        Ok(false) => {}
                        Err(error) => {
                            let _ = notices.send(Notice::status(game.generation, &error));
                            unsaved.insert(id, (unlock, notice));
                        }
                    }
                }
                game.runtime.deactivate(id);
            }
        }
    }
    let _ = frame.recycle.try_send(frame.ram);
}

fn retry_unsaved(
    unsaved: &mut BTreeMap<u32, (Unlock, Notice)>,
    store: &Arc<Mutex<Store>>,
    notices: &mpsc::Sender<Notice>,
) {
    unsaved.retain(
        |_, (unlock, notice)| match store.lock().unwrap().record(unlock.clone()) {
            Ok(true) => {
                let _ = notices.send(notice.clone());
                false
            }
            Ok(false) => false,
            Err(_) => true,
        },
    );
}

/// Canonical title from the achievement cache identified by this ROM's content hash.
/// Call on an I/O worker. This does not authenticate, send ROM data, or update the shared
/// library index. A renamed ROM can reuse the same cached identity immediately.
pub fn artwork_title(root: &std::path::Path, rom: &std::path::Path) -> Option<String> {
    let config = storage::Config::read(&root.join("Config/retroachievements.toml")).ok()?;
    if !config.enabled || config.username.is_empty() {
        return None;
    }
    let account = format!("{:x}", md5::compute(config.username.to_lowercase()));
    let dir = root.join("Saves/RetroAchievements").join(account);
    let hash = library::content_hash(rom).ok()?;
    let (game, _): (storage::Game, std::collections::BTreeSet<u32>) =
        storage::read(&dir.join(format!("{hash}.json"))).ok()?;
    (game.supported_console()
        && game.id != 0
        && !game.title.trim().is_empty()
        && !game.title.starts_with("Unsupported Game Version"))
    .then_some(game.title)
}
