use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

use slot_retro::{AchievementMemory, AvInfo, ButtonMask, CoreError, Link, RetroCore, Rumble};

use crate::{network, Control, Frame, Service, Status, QUEUE_SIZE, RAM_SIZE};

static NEXT_GAME: AtomicU64 = AtomicU64::new(1);

pub(crate) struct Tracked {
    core: Box<dyn RetroCore>,
    controls: mpsc::Sender<Control>,
    frames: mpsc::SyncSender<Frame>,
    recycled: mpsc::Receiver<Vec<u8>>,
    recycle: mpsc::SyncSender<Vec<u8>>,
    enabled: Arc<AtomicBool>,
    current: Arc<AtomicU64>,
    tracking: Arc<AtomicU64>,
    /// slotPlus: the game's RetroAchievements console, which decides its memory layout.
    console: u32,
    generation: u64,
    sequence: u64,
    timeline: u64,
    allocated: usize,
    /// slotPlus: asked about once a second whether the game is doing something that pauses
    /// achievements, as PokeDaisy's own apps pause them; nothing is checked while it is.
    pause: Option<Box<dyn FnMut() -> bool + Send>>,
    paused: bool,
    status: Arc<Status>,
}

/// How many frames apart `pause` is asked.
const PAUSE_EVERY: u64 = 60;

impl Tracked {
    pub fn new(core: Box<dyn RetroCore>, console: u32, service: &Service) -> Self {
        let (recycle, recycled) = mpsc::sync_channel(QUEUE_SIZE + 1);
        Self {
            core,
            controls: service.controls.clone(),
            frames: service.frames.clone(),
            recycled,
            recycle,
            enabled: service.enabled.clone(),
            current: service.current.clone(),
            tracking: service.tracking.clone(),
            console,
            generation: 0,
            sequence: 0,
            timeline: 0,
            allocated: 0,
            pause: None,
            paused: false,
            status: service.status.clone(),
        }
    }

    pub fn pausing(mut self, pause: Box<dyn FnMut() -> bool + Send>) -> Self {
        self.pause = Some(pause);
        self
    }

    fn snapshot(&mut self) {
        self.sequence += 1;
        if self.sequence % PAUSE_EVERY == 1 {
            if let Some(pause) = &mut self.pause {
                self.paused = pause();
                let state = 1 + u8::from(self.paused);
                self.status.paused.store(state, Ordering::Release);
            }
        }
        // Skipped frames leave a gap in the sequence, which resets hit counts as a dropped
        // frame does, so nothing carries across the pause.
        if self.paused {
            return;
        }
        // slotPlus: RAM is copied only once this game's achievements are active, so a game
        // with none, or one still being identified, costs nothing per frame.
        if !self.enabled.load(Ordering::Acquire)
            || self.tracking.load(Ordering::Acquire) != self.generation
        {
            return;
        }
        let mut ram = match self.recycled.try_recv() {
            Ok(ram) => ram,
            Err(_) if self.allocated < QUEUE_SIZE + 1 => {
                self.allocated += 1;
                vec![0; RAM_SIZE]
            }
            Err(_) => return,
        };
        let valid = self.core.achievement_memory(self.console, &mut ram);
        let frame = Frame {
            generation: self.generation,
            sequence: self.sequence,
            timeline: self.timeline,
            earned_at: network::now(),
            ram,
            valid,
            recycle: self.recycle.clone(),
        };
        // In particular, never use send(): HTTPS, fsync, or evaluation must not backpressure
        // gameplay. Sequence gaps reset the worker's hit/delta state before it proceeds.
        if let Err(error) = self.frames.try_send(frame) {
            let (mpsc::TrySendError::Full(frame) | mpsc::TrySendError::Disconnected(frame)) = error;
            let _ = self.recycle.try_send(frame.ram);
        }
    }
}

impl RetroCore for Tracked {
    fn load(&mut self, rom: &Path) -> Result<(), CoreError> {
        self.core.load(rom)?;
        self.generation = NEXT_GAME.fetch_add(1, Ordering::Relaxed);
        self.current.store(self.generation, Ordering::Release);
        let _ = self.controls.send(Control::Load(network::Load {
            generation: self.generation,
            path: rom.to_path_buf(),
        }));
        Ok(())
    }
    fn run_frame(&mut self, input: ButtonMask) {
        self.core.run_frame(input);
        self.snapshot();
    }
    fn run_frame_linked(&mut self, p1: ButtonMask, p2: ButtonMask) {
        self.core.run_frame_linked(p1, p2);
        self.timeline += 1;
    }
    fn set_option(&mut self, key: &str, value: &str) {
        self.core.set_option(key, value);
    }
    fn set_frame_skip(&mut self, skip: bool) {
        self.core.set_frame_skip(skip);
    }
    // Everything the core does is passed through, defaults included: a call left to
    // RetroCore's default would be swallowed here. A reset starts a new timeline, as a loaded
    // state does, so no hit count carries across it.
    fn reset(&mut self) {
        self.core.reset();
        self.timeline += 1;
    }
    fn set_touch(&mut self, at: Option<(u8, u8)>) {
        self.core.set_touch(at);
    }
    fn set_pointer(&mut self, pointer: Option<slot_retro::Pointer>) {
        self.core.set_pointer(pointer);
    }
    fn set_lid(&mut self, shut: bool) {
        self.core.set_lid(shut);
    }
    fn takes_xy(&self) -> bool {
        self.core.takes_xy()
    }
    fn video_xrgb8888(&self) -> &[u8] {
        self.core.video_xrgb8888()
    }
    fn video_size(&self) -> (u32, u32) {
        self.core.video_size()
    }
    fn frame_size(&self) -> (u32, u32) {
        self.core.frame_size()
    }
    fn take_audio(&mut self) -> Vec<i16> {
        self.core.take_audio()
    }
    fn serialize(&mut self) -> Result<Vec<u8>, CoreError> {
        self.core.serialize()
    }
    fn unserialize(&mut self, data: &[u8]) -> Result<(), CoreError> {
        self.core.unserialize(data)?;
        self.timeline += 1;
        Ok(())
    }
    fn save_ram(&self) -> Option<Vec<u8>> {
        self.core.save_ram()
    }
    fn load_save_ram(&mut self, data: &[u8]) -> Result<(), CoreError> {
        self.core.load_save_ram(data)
    }
    fn av_info(&self) -> AvInfo {
        self.core.av_info()
    }
    fn rumble(&self) -> Rumble {
        self.core.rumble()
    }
    fn net(&self) -> Link {
        self.core.net()
    }
    fn start_link(&mut self, client_id: u16) {
        self.timeline += 1;
        self.core.start_link(client_id);
    }
    fn pump_link(&mut self) {
        self.core.pump_link();
    }
    fn stop_link(&mut self) {
        self.core.stop_link();
        self.timeline += 1;
    }
    fn achievement_memory(&self, console: u32, ram: &mut [u8]) -> AchievementMemory {
        self.core.achievement_memory(console, ram)
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        let _ = self.controls.send(Control::Unload(self.generation));
        if self.pause.is_some() {
            self.status.paused.store(0, Ordering::Release);
        }
        let _ =
            self.current
                .compare_exchange(self.generation, 0, Ordering::AcqRel, Ordering::Relaxed);
    }
}
