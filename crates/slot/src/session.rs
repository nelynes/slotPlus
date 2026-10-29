use std::path::PathBuf;
use std::thread::JoinHandle;
use std::time::Duration;

use slot_input::{Action, Btn, Chord, Contexts, Entry, Gestures, Millis, Mod, RawEvent};
use slot_retro::{Rumble, WsLayout, HANDHELD_TURN_OPTION};
use slot_store::{Core, Platform};
use slot_ui::FfState;

pub const RUMBLE_MIN_MS: Millis = 100;
pub const RUMBLE_MAX_MS: Millis = 150;

use crate::app::{App, Phase};
use crate::audio::{opened_sink, AudioSink, Ring, Sfx};
use crate::core::{open_core, Opened};
use crate::emu::{CoreState, EmuHandle, Speed};
use crate::frames::FrameRef;
use crate::input::Pad;
use crate::persist;
use crate::prefetch::Prefetch;

pub struct Session {
    root: PathBuf,
    app: App,
    emu: Option<EmuHandle>,
    opening: Option<Opening>,
    sink: Box<dyn AudioSink>,
    gestures: Gestures,
    entries: Vec<Entry>,
    captured: Vec<Btn>,
    pad: Pad,
    sent: u16,
    rewinding: bool,
    fast: bool,
    motor: u16,
    pulse: Option<(Millis, u16)>,
    prefetch: Prefetch,
    reloading: bool,
    driven: bool,
    ws_vertical: bool,
    /// SELECT flips this Lynx game between the turned and the upright layout.
    lynx_select: bool,
    /// The Lynx game is held the usual way, drawn upright with bars at the sides.
    lynx_upright: bool,
    /// How the handheld is held for the Lynx game playing, as the host is told.
    lynx_held: &'static str,
}

impl Session {
    pub fn boot(root: PathBuf) -> Self {
        Self::boot_with(root, opened_sink())
    }

    pub fn boot_with(root: PathBuf, sink: Box<dyn AudioSink>) -> Self {
        let mut session = Session {
            app: App::boot(&root),
            root,
            emu: None,
            opening: None,
            sink,
            gestures: Gestures::new(),
            entries: Vec::new(),
            captured: Vec::new(),
            pad: Pad::default(),
            sent: 0,
            rewinding: false,
            fast: false,
            motor: 0,
            pulse: None,
            prefetch: Prefetch::default(),
            reloading: false,
            driven: false,
            ws_vertical: false,
            lynx_select: false,
            lynx_upright: false,
            lynx_held: "off",
        };
        // A game left running comes straight back. Its core loads on its own thread, so start it
        // now and let the card read overlap the rest of the boot. Waiting for the first update
        // left a cold boot (after a power off) idle on the slot for the whole load.
        if session.inserting() {
            session.sync_core();
        }
        session
    }

    pub fn play_sfx(&mut self, sfx: Sfx) {
        let ring = self.sink.ring();
        let rate = ring.sample_rate();
        if rate == 0 {
            return;
        }
        let mut samples = sfx.render(rate);
        crate::audio::volume::apply(&mut samples, self.app.output_volume());
        ring.mix(&samples);
    }

    pub fn audio_queued(&self) -> usize {
        self.sink.ring().queued_frames()
    }

    pub fn audio_ring(&self) -> std::sync::Arc<Ring> {
        self.sink.ring()
    }

    pub fn close_audio(&mut self) {
        self.sink.close();
    }

    pub fn rumble(&mut self, strength: u16) {
        if strength == self.motor {
            return;
        }
        self.motor = strength;
        self.app.set_rumble(strength);
    }

    pub fn prefetching(&self) -> Option<&std::path::Path> {
        self.prefetch.reading()
    }

    pub fn core_rumble(&self) -> Option<&Rumble> {
        self.emu.as_ref().map(EmuHandle::rumble)
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn emu(&self) -> Option<&EmuHandle> {
        self.emu.as_ref()
    }

    pub fn set_driven(&mut self, driven: bool) {
        self.driven = driven;
        if let Some(emu) = &self.emu {
            emu.set_driven(driven);
        }
    }

    pub fn step_emulator(&self, present: Duration, timeout: Duration) -> bool {
        self.emu.as_ref().is_some_and(|emu| {
            emu.tick(present);
            !emu.wait_frame(timeout) && emu.locked()
        })
    }

    pub fn frame(&self) -> Option<FrameRef> {
        self.emu.as_ref().and_then(|e| e.latest_frame())
    }

    pub fn core_settling(&self) -> bool {
        self.opening.is_some()
            || self
                .emu
                .as_ref()
                .is_some_and(|e| e.state() == CoreState::Loading)
    }

    pub fn has_core(&self) -> bool {
        self.emu.is_some() || self.opening.is_some()
    }

    pub fn frame_ready(&self) -> bool {
        self.emu.as_ref().is_some_and(EmuHandle::frame_ready)
    }

    pub fn frames_published(&self) -> u64 {
        self.emu.as_ref().map_or(0, EmuHandle::published_count)
    }

    pub fn observed_speed(&self) -> Option<Speed> {
        self.emu.as_ref().map(EmuHandle::observed_speed)
    }

    pub fn frames_taken(&self) -> u64 {
        self.emu.as_ref().map_or(0, EmuHandle::frames_taken)
    }

    pub fn game_visible(&self) -> bool {
        self.app.game_visible()
    }

    pub fn feed(&mut self, events: impl IntoIterator<Item = RawEvent>, now: Millis) {
        let mut actions = Vec::new();
        let latched = self.gestures.ff_latched();
        self.gestures.set_context(self.app.input_context());
        let mut entries = self.app.binding_table().entries();
        if self.app.repair_offered() {
            let chord = Chord {
                modifier: Some(Mod::Select),
                button: Btn::Start,
            };
            entries.insert(
                0,
                Entry {
                    chord,
                    action: Action::RepairCard,
                    contexts: Contexts::ALL,
                },
            );
        }
        if self.entries != entries {
            self.gestures.set_entries(entries.clone());
            self.entries = entries;
        }
        for ev in events {
            if self.app.capturing() {
                match ev {
                    RawEvent::Down(b) => self.captured.push(b),
                    RawEvent::Up(b) => self.captured.retain(|x| *x != b),
                }
                self.gestures.observe(ev);
                self.app.capture_input(ev);
                continue;
            }
            if let RawEvent::Up(b) = ev {
                if let Some(at) = self.captured.iter().position(|x| *x == b) {
                    self.captured.remove(at);
                    self.gestures.observe(ev);
                    continue;
                }
            }
            if self.ds_takes(ev, now) {
                self.gestures.observe(ev);
                continue;
            }
            actions.extend(self.gestures.feed(ev, now));
        }
        actions.extend(self.gestures.tick(now));
        if !latched && self.app.in_game_menu().is_some() {
            actions.extend(self.gestures.drop_ff_latch());
        }
        for action in actions {
            self.act(action);
        }
        self.sync_pad();
    }

    /// A DS game's own controls, ahead of slot's: L2 swaps its main screen,
    /// SELECT + L2 steps through its layouts,
    /// and R2 holds the stylus out, which the D-pad moves and A touches. They
    /// are taken from slot's L2 and R2 shortcuts (rewind and fast forward) in
    /// a DS game. Letting go of a stylus button still reaches the game, so
    /// nothing it held when the stylus came out stays down.
    fn ds_takes(&mut self, ev: RawEvent, now: Millis) -> bool {
        if !self.app.ds_controls() {
            return false;
        }
        match ev {
            RawEvent::Down(Btn::L2) if self.gestures.take_select_chord(now) => {
                self.app.next_ds_layout();
                true
            }
            RawEvent::Down(Btn::L2) => {
                self.app.swap_ds_screens();
                true
            }
            RawEvent::Up(Btn::L2) => true,
            RawEvent::Down(Btn::R2) => {
                self.app.stylus_out(true);
                true
            }
            RawEvent::Up(Btn::R2) => {
                self.app.stylus_out(false);
                true
            }
            RawEvent::Down(b) if self.app.stylus_takes(b) => {
                self.app.stylus_press(b, true);
                true
            }
            RawEvent::Up(b) if self.app.stylus_takes(b) => {
                self.app.stylus_press(b, false);
                false
            }
            _ => false,
        }
    }

    fn sync_pad(&mut self) {
        for btn in self.app.taken_buttons() {
            self.pad.apply(Action::GbaUp(btn));
        }
        if let Some(emu) = &self.emu {
            let mask = self.pad.mask(&self.app.controls());
            let mask = match self.app.core() {
                Core::Cygne => mask.wonderswan_for(self.ws_layout()),
                Core::Lynx if self.lynx_select => mask.without_select(),
                // Gearsystem takes SELECT as a console reset, which the Game
                // Gear does not have.
                Core::Gearsystem => mask.without_select(),
                _ => mask,
            };
            if trace() && mask.0 != self.sent {
                eprintln!("slot: pad {:#06x}", mask.0);
            }
            self.sent = mask.0;
            emu.set_input(mask);
        }
    }

    /// Power the handheld off. A core that writes its own save file does so
    /// when its game is unloaded, so once Slot's own saves have settled, that
    /// core is stopped before the power goes.
    pub fn poweroff(&mut self) {
        self.stop_self_saving_core();
        self.app.poweroff();
    }

    /// Get ready for Slot to exit and be started again: write the resume
    /// state for the game in the slot and every pending save, and stop a core
    /// that writes its own saves, so the game comes back where it was.
    pub fn relaunch(&mut self) {
        self.app.flush_resume();
        self.stop_self_saving_core();
    }

    /// Restart the handheld, stopping a core that writes its own saves first,
    /// as `poweroff` does.
    pub fn restart(&mut self) {
        self.stop_self_saving_core();
        self.app.restart();
    }

    /// Step aside for the SD card repair, stopping a core that writes its own
    /// saves first, as `restart` does, in case the card still takes them.
    pub fn repair(&mut self) {
        self.stop_self_saving_core();
        self.app.repair();
    }

    fn stop_self_saving_core(&mut self) {
        self.app.settle_saves();
        if crate::core::writes_own_saves(self.app.core()) {
            self.emu = None;
        }
    }

    fn ws_layout(&self) -> WsLayout {
        match (self.ws_vertical, self.app.frame_is_tall()) {
            (false, _) => WsLayout::Horizontal,
            (true, true) => WsLayout::Upright,
            (true, false) => WsLayout::Sideways,
        }
    }

    /// SELECT switches between the way the handheld was being held and the
    /// other natural way: upright, or the way the game is made to be held.
    fn flip_lynx_layout(&mut self) {
        self.lynx_upright = !self.lynx_upright;
        let value = match self.lynx_upright {
            true => crate::hold_picker::select_held(self.lynx_held),
            false => self.lynx_held,
        };
        if trace() {
            eprintln!("slot: lynx: {HANDHELD_TURN_OPTION}={value}");
        }
        if let Some(emu) = &self.emu {
            emu.set_option(HANDHELD_TURN_OPTION, value);
        }
    }

    fn bridge_link(&mut self, f: impl FnOnce(&mut App)) {
        let had_link = self.app.link_active();
        f(&mut self.app);
        if had_link && !self.app.link_active() {
            if let Some(emu) = &self.emu {
                emu.end_link();
            }
        }
    }

    fn act(&mut self, action: Action) {
        if trace() {
            eprintln!("slot: {action:?} in {:?}", self.app.phase());
        }
        let menu_open = self.app.in_game_menu().is_some();
        if matches!(action, Action::GbaDown(Btn::Select))
            && self.lynx_select
            && !menu_open
            && self.playing()
            && !self.overlaid()
        {
            self.flip_lynx_layout();
        }
        match action {
            Action::RewindStart if !menu_open => self.rewinding = true,
            Action::RewindStop => self.rewinding = false,
            Action::FfStart if !menu_open => self.fast = true,
            Action::FfStop => self.fast = false,
            _ => {}
        }
        let menu = self.overlaid();
        self.bridge_link(|app| app.apply(action));
        if matches!(
            action,
            Action::VolumeUp | Action::VolumeDown | Action::MuteToggle
        ) {
            if let Some(emu) = &self.emu {
                emu.set_volume(self.app.output_volume());
            }
        }
        if menu || self.overlaid() {
            self.pad.clear();
        } else if self.app.takes_from_the_game(action) {
            if let Action::GbaDown(btn) | Action::GbaUp(btn) = action {
                self.pad.apply(Action::GbaUp(btn));
            }
        } else {
            self.pad.apply(action);
        }
        self.sync_rumble();
    }

    pub fn update(&mut self, dt: f32) {
        self.bridge_link(|app| app.update(dt));
        self.sink.hold_open(self.app.headphones());
        if let Some(emu) = &self.emu {
            emu.set_volume(self.app.output_volume());
        }
        if let Some((client_id, transport)) = self.app.take_link_transport() {
            match &self.emu {
                Some(emu) => match self.app.link_player() {
                    Some(player) => emu.begin_cable(player, transport),
                    None => emu.begin_link(client_id, transport),
                },
                None => eprintln!("slot: link: a transport arrived with no core to run it"),
            }
        }
        let mut recoloured = false;
        if let Some(p) = self.app.take_gb_colors() {
            if let Some(emu) = &self.emu {
                emu.set_option("mgba_gb_colors", p.core_name());
                recoloured = true;
            }
        }
        if let Some(on) = self.app.take_colour_correction() {
            if let Some((key, value)) =
                crate::core::colour_option(self.app.core(), self.app.playing_platform(), on)
            {
                if let Some(emu) = &self.emu {
                    emu.set_option(key, value);
                    recoloured = true;
                }
            }
        }
        if let Some(blending) = self.app.take_blending() {
            if let (Some(emu), Some((key, value))) = (
                &self.emu,
                crate::core::blend_option(self.app.core(), blending),
            ) {
                emu.set_option(key, value);
                recoloured = true;
            }
        }
        if recoloured && self.held() && !self.app.link_active() {
            if let Some(emu) = &self.emu {
                emu.refresh();
            }
        }
        if let Some((stem, serial)) = self.app.take_link_reload() {
            self.reload_for_link(&stem, serial);
        }
        if self.app.link_active() {
            if self.emu.as_ref().is_some_and(EmuHandle::bios_mismatch) {
                self.bridge_link(|app| app.bios_mismatch());
            } else if self.emu.as_ref().is_some_and(EmuHandle::peer_ended) {
                self.bridge_link(|app| app.peer_ended());
            } else if self.emu.as_ref().is_some_and(EmuHandle::link_lost) {
                self.app.peer_lost();
            }
        }
        if let Some(sfx) = self.app.take_sfx() {
            self.play_sfx(sfx);
        }
        self.sync_core();
        self.sync_reload();
        if !self.has_core() {
            for action in self.gestures.drop_ff_latch() {
                self.act(action);
            }
        }
        let ready = self.emu.as_ref().is_some_and(EmuHandle::has_published);
        if ready && !self.app.game_ready() {
            launch("first frame shown");
        }
        self.app.set_game_ready(ready);
        self.sync_speed();
        self.sync_rewind_hud();
        self.sync_ff_hud();
        self.sync_rumble();
        self.sync_pad();
        let now = self.app.now();
        self.prefetch.browse(self.app.browsed_rom(), now);
    }

    fn sync_rumble(&mut self) {
        let asked = match &self.emu {
            Some(emu) if self.playing() && self.app.rumble_enabled() => emu.rumble().strength(),
            _ => {
                self.pulse = None;
                return self.rumble(0);
            }
        };
        let now = self.app.now();
        let want = match (asked, self.pulse) {
            (0, Some((at, strength))) if now.saturating_sub(at) < RUMBLE_MIN_MS => strength,
            (0, _) => {
                self.pulse = None;
                0
            }
            (asked, Some((at, _))) => {
                self.pulse = Some((at, asked));
                if now.saturating_sub(at) < RUMBLE_MAX_MS {
                    asked
                } else {
                    0
                }
            }
            (asked, None) => {
                self.pulse = Some((now, asked));
                asked
            }
        };
        self.rumble(want);
    }

    fn sync_ff_hud(&mut self) {
        let ff = match (self.actually_fast_forwarding(), self.gestures.ff_latched()) {
            (false, _) => FfState::Off,
            (true, false) => FfState::Held,
            (true, true) => FfState::Latched,
        };
        self.app.set_ff(ff);
    }

    fn sync_rewind_hud(&mut self) {
        let fill = self
            .actually_rewinding()
            .then(|| self.emu.as_ref().map(EmuHandle::rewind_fill))
            .flatten();
        match fill {
            Some(fill) => self.app.show_rewind(fill),
            None => self.app.hide_rewind(),
        }
    }

    fn actually_rewinding(&self) -> bool {
        self.rewinding && self.playing() && self.app.may_rewind()
    }

    fn actually_fast_forwarding(&self) -> bool {
        self.fast && self.playing() && self.app.may_fast_forward()
    }

    fn inserting(&self) -> bool {
        matches!(self.app.phase(), Phase::Inserting { .. })
    }

    fn showing_polaroids(&self) -> bool {
        matches!(self.app.phase(), Phase::Polaroids { .. })
    }

    fn overlaid(&self) -> bool {
        self.showing_polaroids() || self.held()
    }

    fn playing(&self) -> bool {
        matches!(self.app.phase(), Phase::Playing { .. }) && !self.held()
    }

    fn held(&self) -> bool {
        self.app.game_menu_open()
            || self.app.in_game_menu().is_some()
            || self.app.repair_prompt()
            || self.app.shutting_down()
    }

    fn dozing(&self) -> bool {
        matches!(self.app.phase(), Phase::Doze { .. })
    }

    fn ejecting(&self) -> bool {
        matches!(self.app.phase(), Phase::Ejecting { .. })
    }

    fn sync_speed(&self) {
        let mut ds_running = false;
        if let Some(emu) = &self.emu {
            emu.set_fast_quarters(u32::from(self.app.ff_speed()));
            emu.set_ff_sound(self.app.ff_sound());
            let speed = if self.inserting()
                || self.ejecting()
                || self.showing_polaroids()
                || self.dozing()
                || (self.held() && !self.app.link_active())
            {
                Speed::Paused
            } else if self.actually_fast_forwarding() {
                Speed::Fast
            } else {
                Speed::Normal
            };
            emu.set_speed(speed);
            emu.set_rewinding(self.actually_rewinding());
            emu.set_turbo(self.app.turbo());
            emu.set_face_xy(self.app.core() == Core::Dsperate);
            emu.set_skip_slow(self.app.core() == Core::Dsperate);
            emu.set_pointer(self.app.pointer());
            emu.set_rewind_recording(self.app.records_rewind());
            ds_running = self.app.core() == Core::Dsperate && speed != Speed::Paused;
        }
        crate::cpu_governor::hold_full(ds_running);
    }

    fn no_core() -> bool {
        std::env::var_os("SLOT_NO_CORE").is_some_and(|v| v != "0")
    }

    fn sync_core(&mut self) {
        if Self::no_core() {
            return;
        }
        let stem = match self.app.phase() {
            Phase::Shelf => {
                self.opening = None;
                self.emu = None;
                return;
            }
            Phase::Inserting { cart, .. } => cart.clone(),
            _ => return,
        };
        if self.emu.is_none() && self.opening.is_none() {
            let (_, serial) = self.app.link_mode(&stem);
            self.spawn_core(&stem, serial);
        }
        if !self.finish_opening() {
            return;
        }
        match self.emu.as_ref().map(EmuHandle::state) {
            Some(CoreState::Loading) => {}
            Some(CoreState::Ready) => {
                if let (true, Some(emu)) = (self.app.palette_live(), &self.emu) {
                    emu.set_option("mgba_gb_colors", self.app.gb_colors().core_name());
                }
                self.app.on_core_ready()
            }
            Some(CoreState::Failed) | None => {
                self.emu = None;
                self.app.on_core_failed();
            }
        }
    }

    fn spawn_core(&mut self, stem: &str, serial: &'static str) {
        let Some((rom, platform)) = self
            .app
            .seated_cart()
            .filter(|c| c.stem == stem)
            .map(|c| (c.rom.clone(), c.platform))
        else {
            return;
        };
        let core = slot_store::core_for_platform(&self.root, stem, platform);
        launch(&format!("opening {stem} on {}", core.as_str()));
        self.app.set_core(core);
        self.app.set_platform(platform);
        self.app
            .set_video_mode(crate::video_mode::video_mode_for(&self.root, stem));
        self.app
            .set_ds_view(crate::video_mode::ds_view_for(&self.root, stem));
        self.app.set_link_loaded(serial);
        let root = self.root.clone();
        let stem = stem.to_owned();
        let clean = self.app.starting_clean();
        let player = self.app.link_player();
        let chosen = self.app.gb_colors();
        let colour = self.app.colour_correction_for(platform);
        let blending = self.app.blending_for(platform);
        let join = std::thread::Builder::new()
            .name("slot-open".into())
            .spawn(move || {
                let resume = match clean {
                    true => Ok(None),
                    false => persist::read_resume(&root, platform, core, &stem),
                };
                launch("resume state read");
                let palette = crate::core::palette_for(chosen, platform, &rom, player);
                let mut opened = open_core(
                    &root, core, platform, serial, colour, blending, palette, player,
                );
                crate::core::apply_game_boy_player(&mut *opened.core, core, platform, &rom);
                launch("core opened");
                let sav = persist::read_sav(&root, platform, &stem);
                launch("save read");
                // A Lynx game's turn and a WonderSwan game's orientation come
                // from files too, so they are read here with the rest.
                let ws_vertical = platform == Platform::Ws && slot_store::ws_vertical(&rom);
                let lynx_turn = (platform == Platform::Lynx)
                    .then(|| crate::hold_picker::lynx_setup(&root, &stem));
                Loaded {
                    opened,
                    rom,
                    platform,
                    player,
                    palette_live: palette.is_some(),
                    sav,
                    resume,
                    ws_vertical,
                    lynx_turn,
                }
            })
            .ok();
        self.opening = join.map(|join| Opening { join: Some(join) });
    }

    fn finish_opening(&mut self) -> bool {
        let Some(opening) = &mut self.opening else {
            return true;
        };
        if !opening.join.as_ref().is_some_and(JoinHandle::is_finished) {
            return false;
        }
        let loaded = opening.join.take().and_then(|j| j.join().ok());
        self.opening = None;
        let Some(loaded) = loaded else {
            return true;
        };
        self.app.set_palette_live(loaded.palette_live);
        self.app.set_named_core(loaded.opened.named);
        let ring = self.sink.ring();
        let sav = self.app.card_read("save", loaded.sav);
        let resume = self.app.card_read("resume", loaded.resume);
        let (sav_unread, resume_unread) = (sav.is_none(), resume.is_none());
        let (core, rom, sav, resume) = (
            loaded.opened.core,
            loaded.rom,
            sav.flatten(),
            resume.flatten(),
        );
        let emu = match loaded.player.filter(|_| loaded.platform == Platform::Gba) {
            Some(p) => EmuHandle::spawn_linked(core, rom, ring, sav, resume, p),
            None => EmuHandle::spawn(core, rom, ring, sav, resume),
        };
        if sav_unread {
            emu.refuse_save_ram();
        }
        if resume_unread {
            emu.refuse_resume();
        }
        self.app.set_resume_unread(resume_unread);
        self.ws_vertical = loaded.ws_vertical;
        self.lynx_select = false;
        self.lynx_upright = false;
        if let Some(setup) = loaded.lynx_turn {
            eprintln!(
                "slot: lynx: lynx_rot_screen={}, {HANDHELD_TURN_OPTION}={}",
                setup.core_turn, setup.held
            );
            emu.set_option("lynx_rot_screen", setup.core_turn);
            emu.set_option(HANDHELD_TURN_OPTION, setup.held);
            self.lynx_held = setup.held;
            self.lynx_select = crate::core::lynx_select_toggles(setup.core_turn);
        }
        emu.set_volume(self.app.output_volume());
        emu.set_driven(self.driven);
        self.app.set_snapshot(Box::new(emu.snapshot()));
        self.emu = Some(emu);
        true
    }

    fn reload_for_link(&mut self, stem: &str, serial: &'static str) {
        eprintln!("slot: link: loading {stem} again with gpsp_serial={serial}");
        if self.emu.is_some() {
            self.app.flush_resume();
        }
        self.opening = None;
        self.emu = None;
        self.spawn_core(stem, serial);
        self.reloading = true;
    }

    fn sync_reload(&mut self) {
        if !self.reloading || !self.finish_opening() {
            return;
        }
        match self.emu.as_ref().map(EmuHandle::state) {
            Some(CoreState::Loading) => {}
            Some(CoreState::Ready) => {
                self.reloading = false;
                self.app.link_reload_done();
            }
            Some(CoreState::Failed) | None => {
                self.reloading = false;
                self.emu = None;
                self.app.link_reload_failed();
                if let Some((stem, serial)) = self.app.take_link_reload() {
                    self.reload_for_link(&stem, serial);
                }
            }
        }
    }
}

struct Loaded {
    opened: Opened,
    rom: PathBuf,
    platform: Platform,
    player: Option<u8>,
    palette_live: bool,
    sav: std::io::Result<Option<Vec<u8>>>,
    resume: std::io::Result<Option<Vec<u8>>>,
    ws_vertical: bool,
    lynx_turn: Option<crate::hold_picker::LynxSetup>,
}

struct Opening {
    join: Option<JoinHandle<Loaded>>,
}

impl Drop for Opening {
    fn drop(&mut self) {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

pub(crate) fn trace() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("SLOT_TRACE").is_some())
}

fn launch(step: &str) {
    eprintln!("slot: launch: {step} at {:.1} ms", crate::boot_time::ms());
}
