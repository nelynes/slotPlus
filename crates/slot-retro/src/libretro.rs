use std::cell::Cell;
use std::ffi::{c_char, c_int, c_uint, c_void, CString};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use libloading::Library;

use crate::core::{AvInfo, ButtonMask, CoreError, RetroCore, GBA_H, GBA_W};
use crate::ffi::*;
use crate::link::Link;
use crate::rumble::Rumble;
use crate::{AchievementMemory, ACHIEVEMENT_RAM_SIZE};

const VIDEO_BYTES: usize = (GBA_W * GBA_H * 4) as usize;

static LIVE: AtomicBool = AtomicBool::new(false);

/// A host option, not a core one: how the handheld is held. `off` (upright),
/// `ccw` (a quarter anticlockwise), `180` (upside down) or `cw` (a quarter
/// clockwise) draw the core's picture so it is upright to someone holding the
/// handheld that way, and turn the pad's directions to follow it. `native`
/// holds it however the core asks its picture to be turned (SET_ROTATION), so
/// the picture is drawn as the core made it and the pad is left as it is, as
/// on the console held the way the game was made to be held.
pub const HANDHELD_TURN_OPTION: &str = "slot_handheld_turn";

/// How the handheld is held, from `HANDHELD_TURN_OPTION`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Held {
    /// Turned this many quarters anticlockwise.
    Turned(u32),
    /// However the core asks its picture to be turned.
    Native,
}

impl Held {
    fn parse(value: &str) -> Held {
        match value {
            "ccw" => Held::Turned(1),
            "180" => Held::Turned(2),
            "cw" => Held::Turned(3),
            "native" => Held::Native,
            _ => Held::Turned(0),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum PixelFormat {
    Xrgb8888,
    Rgb565,
}

struct Host {
    video: Vec<u8>,
    video_w: u32,
    video_h: u32,
    shown_w: u32,
    shown_h: u32,
    /// Quarter turns anticlockwise the core asked the frontend to draw its
    /// picture at (libretro's SET_ROTATION). 0 is upright.
    rotation: u32,
    /// How the handheld is held. See `HANDHELD_TURN_OPTION`.
    held: Held,
    format: PixelFormat,
    audio: Vec<i16>,
    inputs: [u16; 2],
    system_dir: CString,
    save_dir: CString,
    rumble: Rumble,
    asked_for_rumble: bool,
    netpacket: Option<NetpacketCallback>,
    net: Link,
    net_peer: Option<u16>,
    audio_status: Option<AudioBufferStatusFn>,
    options: std::collections::HashMap<String, std::ffi::CString>,
    options_dirty: bool,
    declared: std::collections::HashMap<String, Vec<String>>,
    /// The core's memory map (SET_MEMORY_MAPS), for RetroAchievements.
    memory: Vec<MemoryDescriptor>,
}

thread_local! {
    static ACTIVE: Cell<*mut Host> = const { Cell::new(ptr::null_mut()) };
}

struct Active<'a>(PhantomData<&'a mut Host>);

impl Active<'_> {
    fn bind(host: &mut Host) -> Active<'_> {
        ACTIVE.with(|a| a.set(host as *mut Host));
        Active(PhantomData)
    }
}

impl Drop for Active<'_> {
    fn drop(&mut self) {
        ACTIVE.with(|a| a.set(ptr::null_mut()));
    }
}

unsafe fn with_host<R>(f: impl FnOnce(&mut Host) -> R) -> Option<R> {
    let p = ACTIVE.with(|a| a.get());
    if p.is_null() {
        return None;
    }
    Some(f(&mut *p))
}

unsafe extern "C" fn log_noop(_level: c_uint, _fmt: *const c_char) {}

unsafe extern "C" fn set_rumble_state(port: c_uint, effect: c_uint, strength: u16) -> bool {
    with_host(|h| h.rumble.set(port, effect, strength)).unwrap_or(false)
}

unsafe extern "C" fn environment(cmd: c_uint, data: *mut c_void) -> bool {
    match cmd {
        SET_ROTATION => {
            if data.is_null() {
                return false;
            }
            let turns = *(data as *const c_uint);
            if turns > 3 {
                return false;
            }
            eprintln!("slot-retro: the core turned the picture {turns} quarter turn(s)");
            with_host(|h| h.rotation = turns).is_some()
        }
        GET_CAN_DUPE => {
            if data.is_null() {
                return false;
            }
            *(data as *mut bool) = true;
            true
        }
        SET_PIXEL_FORMAT => {
            if data.is_null() {
                return false;
            }
            let want = match *(data as *const c_uint) {
                PIXEL_FORMAT_XRGB8888 => PixelFormat::Xrgb8888,
                PIXEL_FORMAT_RGB565 => PixelFormat::Rgb565,
                _ => return false,
            };
            with_host(|h| h.format = want).is_some()
        }
        GET_SYSTEM_DIRECTORY | GET_SAVE_DIRECTORY => {
            if data.is_null() {
                return false;
            }
            with_host(|h| {
                let dir = if cmd == GET_SYSTEM_DIRECTORY {
                    &h.system_dir
                } else {
                    &h.save_dir
                };
                *(data as *mut *const c_char) = dir.as_ptr();
                true
            })
            .unwrap_or(false)
        }
        GET_VARIABLE => {
            if data.is_null() {
                return false;
            }
            let var = &mut *(data as *mut Variable);
            if var.key.is_null() {
                return false;
            }
            let Ok(key) = std::ffi::CStr::from_ptr(var.key).to_str() else {
                return false;
            };
            with_host(|h| match h.options.get(key) {
                Some(value) => {
                    var.value = value.as_ptr();
                    true
                }
                None => {
                    var.value = ptr::null();
                    false
                }
            })
            .unwrap_or(false)
        }
        GET_VARIABLE_UPDATE => {
            if data.is_null() {
                return false;
            }
            with_host(|h| {
                *(data as *mut bool) = h.options_dirty;
                h.options_dirty = false;
                true
            })
            .unwrap_or(false)
        }
        SET_VARIABLES => {
            if data.is_null() {
                return with_host(|h| h.declared.clear()).is_some();
            }
            let mut list = std::collections::HashMap::new();
            let mut p = data as *const Variable;
            for _ in 0..4096 {
                let var = &*p;
                if var.key.is_null() {
                    break;
                }
                let Ok(key) = std::ffi::CStr::from_ptr(var.key).to_str() else {
                    break;
                };
                let values = if var.value.is_null() {
                    Vec::new()
                } else {
                    std::ffi::CStr::from_ptr(var.value)
                        .to_str()
                        .ok()
                        .and_then(|v| v.split_once(';'))
                        .map(|(_, values)| {
                            values
                                .trim()
                                .split('|')
                                .map(|s| s.trim().to_string())
                                .collect()
                        })
                        .unwrap_or_default()
                };
                list.insert(key.to_string(), values);
                p = p.add(1);
            }
            with_host(|h| h.declared = list).is_some()
        }
        GET_RUMBLE_INTERFACE => {
            if data.is_null() {
                return false;
            }
            with_host(|h| {
                h.asked_for_rumble = true;
                (*(data as *mut RumbleInterface)).set_rumble_state = set_rumble_state;
            })
            .is_some()
        }
        GET_LOG_INTERFACE => {
            if data.is_null() {
                return false;
            }
            (*(data as *mut LogCallback)).log = log_noop as *const c_void;
            true
        }
        SET_MEMORY_MAPS => {
            if data.is_null() {
                return false;
            }
            let map = &*(data as *const MemoryMap);
            if map.num_descriptors > 1024 || (map.num_descriptors > 0 && map.descriptors.is_null())
            {
                return false;
            }
            with_host(|h| {
                h.memory = if map.num_descriptors == 0 {
                    Vec::new()
                } else {
                    std::slice::from_raw_parts(map.descriptors, map.num_descriptors as usize)
                        .to_vec()
                };
            })
            .is_some()
        }
        SET_AUDIO_BUFFER_STATUS_CALLBACK => {
            if data.is_null() {
                return with_host(|h| h.audio_status = None).is_some();
            }
            let cb = std::ptr::read(data as *const AudioBufferStatusCallback);
            with_host(|h| h.audio_status = cb.callback).is_some()
        }
        SET_NETPACKET_INTERFACE => {
            if data.is_null() {
                return with_host(|h| h.netpacket = None).is_some();
            }
            with_host(|h| {
                h.netpacket = Some(std::ptr::read(data as *const NetpacketCallback));
            })
            .is_some()
        }
        _ => false,
    }
}

unsafe extern "C" fn netpacket_send(
    _flags: c_int,
    buf: *const c_void,
    len: usize,
    _client_id: u16,
) {
    if buf.is_null() || len == 0 {
        return;
    }
    let bytes = std::slice::from_raw_parts(buf as *const u8, len).to_vec();
    with_host(|h| h.net.push_outbound(bytes));
}

unsafe extern "C" fn netpacket_poll_receive() {
    let Some((receive, net, client_id)) = with_host(|h| {
        let receive = h.netpacket.as_ref().and_then(|cb| cb.receive)?;
        let client_id = h.net_peer?;
        Some((receive, h.net.clone(), client_id))
    })
    .flatten() else {
        return;
    };
    if !net.is_active() {
        return;
    }
    while let Some(packet) = net.take_inbound() {
        receive(packet.as_ptr() as *const c_void, packet.len(), client_id);
    }
}

unsafe fn begin_link(client_id: u16) {
    let Some(start) = with_host(|h| h.netpacket.as_ref().and_then(|cb| cb.start)).flatten() else {
        return;
    };
    start(client_id, netpacket_send, netpacket_poll_receive);

    let peer = 1u16.wrapping_sub(client_id);
    let connected = with_host(|h| {
        h.net.set_active(true);
        h.net_peer = Some(peer);
        h.netpacket.as_ref().and_then(|cb| cb.connected)
    })
    .flatten();
    if let Some(connected) = connected {
        connected(peer);
    }
}

unsafe fn halt_link() {
    let Some((stop, peer, disconnected)) = with_host(|h| {
        let stop = h.netpacket.as_ref().and_then(|cb| cb.stop);
        let peer = h.net_peer.take();
        let disconnected = h.netpacket.as_ref().and_then(|cb| cb.disconnected);
        (stop, peer, disconnected)
    }) else {
        return;
    };
    if let (Some(peer), Some(disconnected)) = (peer, disconnected) {
        disconnected(peer);
    }
    if let Some(stop) = stop {
        stop();
    }
}

unsafe fn drain_link() {
    let Some((receive, poll, net, client_id)) = with_host(|h| {
        let cb = h.netpacket.as_ref()?;
        let client_id = h.net_peer?;
        Some((cb.receive, cb.poll, h.net.clone(), client_id))
    })
    .flatten() else {
        return;
    };
    if !net.is_active() {
        return;
    }
    if let Some(receive) = receive {
        while let Some(packet) = net.take_inbound() {
            receive(packet.as_ptr() as *const c_void, packet.len(), client_id);
        }
    }
    if let Some(poll) = poll {
        poll();
    }
}

unsafe fn report_audio_status(skip: bool) {
    let Some(status) = with_host(|h| h.audio_status).flatten() else {
        return;
    };
    status(true, if skip { 0 } else { 100 }, skip);
}

unsafe extern "C" fn video_refresh(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    if data.is_null() {
        return;
    }
    with_host(|h| {
        let turns = drawn_turns(h);
        if turns != 0 {
            draw_turned(
                h,
                data as *const u8,
                width as usize,
                height as usize,
                pitch,
                turns,
            );
            return;
        }
        let (bw, bh) = (h.video_w as usize, h.video_h as usize);
        let cols = (width as usize).min(bw);
        let rows = (height as usize).min(bh);
        let ox = (bw - cols) / 2;
        let oy = (bh - rows) / 2;
        if cols != bw || rows != bh {
            h.video.fill(0);
        }
        h.shown_w = cols as u32;
        h.shown_h = rows as u32;
        for y in 0..rows {
            let src = (data as *const u8).add(y * pitch);
            let row = ((y + oy) * bw + ox) * 4;
            match h.format {
                PixelFormat::Xrgb8888 => {
                    ptr::copy_nonoverlapping(src, h.video.as_mut_ptr().add(row), cols * 4);
                }
                PixelFormat::Rgb565 => {
                    for x in 0..cols {
                        let p = ptr::read_unaligned((src as *const u16).add(x));
                        let r = ((p >> 11) & 0x1f) as u8;
                        let g = ((p >> 5) & 0x3f) as u8;
                        let b = (p & 0x1f) as u8;
                        let o = row + x * 4;
                        h.video[o] = (b << 3) | (b >> 2);
                        h.video[o + 1] = (g << 2) | (g >> 4);
                        h.video[o + 2] = (r << 3) | (r >> 2);
                        h.video[o + 3] = 0;
                    }
                }
            }
        }
    });
}

/// One pixel of a core frame as the host stores it: B, G, R, 0.
unsafe fn frame_pixel(format: PixelFormat, row: *const u8, x: usize) -> [u8; 4] {
    match format {
        PixelFormat::Xrgb8888 => {
            let p = ptr::read_unaligned((row as *const u32).add(x));
            let [b, g, r, _] = p.to_le_bytes();
            [b, g, r, 0]
        }
        PixelFormat::Rgb565 => {
            let p = ptr::read_unaligned((row as *const u16).add(x));
            let r = ((p >> 11) & 0x1f) as u8;
            let g = ((p >> 5) & 0x3f) as u8;
            let b = (p & 0x1f) as u8;
            [
                (b << 3) | (b >> 2),
                (g << 2) | (g >> 4),
                (r << 3) | (r >> 2),
                0,
            ]
        }
    }
}

/// Quarter turns anticlockwise the handheld is held at.
fn held_turns(h: &Host) -> u32 {
    match h.held {
        Held::Turned(t) => t,
        Held::Native => h.rotation,
    }
}

/// Quarter turns anticlockwise to draw the core's frame at: the turn the core
/// asks for, less the turn the handheld is held at, so the picture is upright
/// to the player holding it.
fn drawn_turns(h: &Host) -> u32 {
    (h.rotation + 4 - held_turns(h)) % 4
}

/// The pad as the core should see it: as though the handheld were upright.
/// Held turned, the handheld's own directions are turned with it, a quarter
/// for each quarter it is turned.
fn held_pad(h: &Host, input: u16) -> u16 {
    (0..held_turns(h)).fold(input, |pad, _| turned_pad(pad))
}

/// The pad as the core should see it. On a handheld held turned a quarter
/// anticlockwise the top of the picture is the pad's right, so a turned game
/// is given right as up, down as right, left as down and up as left. The core
/// then applies its own turn on top, as it would for an upright picture.
fn turned_pad(input: u16) -> u16 {
    const DIRECTIONS: u16 =
        ButtonMask::UP | ButtonMask::DOWN | ButtonMask::LEFT | ButtonMask::RIGHT;
    let mut out = input & !DIRECTIONS;
    for (from, to) in [
        (ButtonMask::RIGHT, ButtonMask::UP),
        (ButtonMask::DOWN, ButtonMask::RIGHT),
        (ButtonMask::LEFT, ButtonMask::DOWN),
        (ButtonMask::UP, ButtonMask::LEFT),
    ] {
        if input & from != 0 {
            out |= to;
        }
    }
    out
}

/// Draw a frame the core left upright, turned `turns` quarter turns
/// anticlockwise, centred in the buffer. A turned frame swaps its width and
/// height, and `shown_w` / `shown_h` say so.
unsafe fn draw_turned(
    h: &mut Host,
    data: *const u8,
    fw: usize,
    fh: usize,
    pitch: usize,
    turns: u32,
) {
    let (bw, bh) = (h.video_w as usize, h.video_h as usize);
    let (dw, dh) = if turns & 1 == 1 { (fh, fw) } else { (fw, fh) };
    let cols = dw.min(bw);
    let rows = dh.min(bh);
    let ox = (bw - cols) / 2;
    let oy = (bh - rows) / 2;
    if cols != bw || rows != bh {
        h.video.fill(0);
    }
    h.shown_w = cols as u32;
    h.shown_h = rows as u32;
    for dy in 0..rows {
        for dx in 0..cols {
            let (sx, sy) = match turns {
                1 => (fw - 1 - dy, dx),
                2 => (fw - 1 - dx, fh - 1 - dy),
                _ => (dy, fh - 1 - dx),
            };
            let px = frame_pixel(h.format, data.add(sy * pitch), sx);
            let o = ((dy + oy) * bw + dx + ox) * 4;
            h.video[o..o + 4].copy_from_slice(&px);
        }
    }
}

unsafe extern "C" fn audio_sample(left: i16, right: i16) {
    with_host(|h| h.audio.extend_from_slice(&[left, right]));
}

unsafe extern "C" fn audio_batch(data: *const i16, frames: usize) -> usize {
    if !data.is_null() {
        with_host(|h| {
            h.audio
                .extend_from_slice(std::slice::from_raw_parts(data, frames * 2))
        });
    }
    frames
}

unsafe extern "C" fn input_poll() {}

unsafe extern "C" fn input_state(port: c_uint, device: c_uint, _index: c_uint, id: c_uint) -> i16 {
    if device != DEVICE_JOYPAD {
        return 0;
    }
    with_host(|h| {
        let Some(&input) = h.inputs.get(port as usize) else {
            return 0;
        };
        let input = held_pad(h, input);
        match id {
            JOYPAD_MASK => input as i16,
            _ if id < 16 => ((input >> id) & 1) as i16,
            _ => 0,
        }
    })
    .unwrap_or(0)
}

pub struct LibretroCore {
    api: Api,
    host: Box<Host>,
    rom: Vec<u8>,
    rom_path: Option<CString>,
    av: AvInfo,
    loaded: bool,
    state_size: Option<(usize, usize)>,
    _lib: Library,
}

fn cdir(path: &Path) -> Result<CString, CoreError> {
    CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| CoreError::Load(format!("{} contains a nul", path.display())))
}

impl LibretroCore {
    pub fn open(dylib: &Path) -> Result<Self, CoreError> {
        let dir = dylib.parent().unwrap_or(Path::new(".")).to_path_buf();
        Self::open_with(dylib, &dir, &dir)
    }

    pub fn open_with(dylib: &Path, system_dir: &Path, save_dir: &Path) -> Result<Self, CoreError> {
        if LIVE.swap(true, Ordering::SeqCst) {
            return Err(CoreError::Unsupported("a core is already open".into()));
        }
        Self::open_inner(dylib, system_dir, save_dir)
            .inspect_err(|_| LIVE.store(false, Ordering::SeqCst))
    }

    /// Resize the host's video buffer. Frames the core draws are centred in
    /// it; anything bigger than the buffer is cropped. Call before running.
    pub fn set_video_size(&mut self, w: u32, h: u32) {
        self.host.video_w = w;
        self.host.video_h = h;
        self.host.shown_w = w;
        self.host.shown_h = h;
        self.host.video = vec![0; (w * h * 4) as usize];
    }

    pub fn reported_system_dir(&self) -> String {
        self.host.system_dir.to_string_lossy().into_owned()
    }

    pub fn reported_save_dir(&self) -> String {
        self.host.save_dir.to_string_lossy().into_owned()
    }

    pub fn asked_for_rumble(&self) -> bool {
        self.host.asked_for_rumble
    }

    pub fn set_option(&mut self, key: &str, value: &str) {
        if key == HANDHELD_TURN_OPTION {
            self.host.held = Held::parse(value);
            return;
        }
        if !self.host.declared.is_empty() {
            match self.host.declared.get(key) {
                None => eprintln!("slot-retro: core declares no option {key:?}, setting it anyway"),
                Some(values) if !values.is_empty() && !values.iter().any(|v| v == value) => {
                    eprintln!("slot-retro: core declares {key:?} as {values:?}, not {value:?}")
                }
                Some(_) => {}
            }
        }
        let Ok(value) = CString::new(value) else {
            return;
        };
        self.host.options.insert(key.to_string(), value);
        self.host.options_dirty = true;
    }

    pub fn options(&self) -> Vec<(String, String)> {
        self.host
            .options
            .iter()
            .filter_map(|(k, v)| Some((k.clone(), v.to_str().ok()?.to_string())))
            .collect()
    }

    pub fn declared_options(&self) -> &std::collections::HashMap<String, Vec<String>> {
        &self.host.declared
    }

    pub fn option(&self, key: &str) -> Option<String> {
        self.host
            .options
            .get(key)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    fn open_inner(dylib: &Path, system_dir: &Path, save_dir: &Path) -> Result<Self, CoreError> {
        let lib = unsafe { Library::new(dylib) }.map_err(|e| CoreError::Load(e.to_string()))?;
        let api = unsafe { Api::load(&lib) }?;
        let version = unsafe { (api.api_version)() };
        if version != API_VERSION {
            return Err(CoreError::Unsupported(format!("libretro api {version}")));
        }
        let mut host = Box::new(Host {
            video: vec![0; VIDEO_BYTES],
            video_w: GBA_W,
            video_h: GBA_H,
            shown_w: GBA_W,
            shown_h: GBA_H,
            rotation: 0,
            held: Held::Turned(0),
            format: PixelFormat::Xrgb8888,
            audio: Vec::new(),
            inputs: [0; 2],
            system_dir: cdir(system_dir)?,
            save_dir: cdir(save_dir)?,
            rumble: Rumble::default(),
            asked_for_rumble: false,
            netpacket: None,
            net: Link::default(),
            net_peer: None,
            audio_status: None,
            options: std::collections::HashMap::new(),
            options_dirty: false,
            declared: std::collections::HashMap::new(),
            memory: Vec::new(),
        });
        unsafe {
            let _a = Active::bind(&mut host);
            (api.set_environment)(environment);
            (api.set_video_refresh)(video_refresh);
            (api.set_audio_sample)(audio_sample);
            (api.set_audio_sample_batch)(audio_batch);
            (api.set_input_poll)(input_poll);
            (api.set_input_state)(input_state);
            (api.init)();
        }
        Ok(LibretroCore {
            api,
            host,
            rom: Vec::new(),
            rom_path: None,
            av: AvInfo {
                fps: 0.0,
                sample_rate: 0.0,
            },
            loaded: false,
            state_size: None,
            _lib: lib,
        })
    }

    fn unload(&mut self) {
        if !self.loaded {
            return;
        }
        {
            let _a = Active::bind(&mut self.host);
            unsafe { (self.api.unload_game)() };
        }
        self.loaded = false;
        self.host.memory.clear();
    }

    fn serialize_at(&mut self, size: usize) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; size];
        let ok = {
            let _a = Active::bind(&mut self.host);
            unsafe { (self.api.serialize)(buf.as_mut_ptr() as *mut c_void, size) }
        };
        ok.then_some(buf)
    }

    fn memory(&self, id: c_uint) -> Option<&[u8]> {
        let data = unsafe { (self.api.get_memory_data)(id) };
        let len = unsafe { (self.api.get_memory_size)(id) };
        if data.is_null() || len == 0 {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts(data as *const u8, len) })
    }

    fn memory_mut(&mut self, id: c_uint) -> Option<&mut [u8]> {
        let data = unsafe { (self.api.get_memory_data)(id) };
        let len = unsafe { (self.api.get_memory_size)(id) };
        if data.is_null() || len == 0 {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts_mut(data as *mut u8, len) })
    }
}

impl Drop for LibretroCore {
    fn drop(&mut self) {
        self.unload();
        unsafe {
            let _a = Active::bind(&mut self.host);
            (self.api.deinit)();
        }
        LIVE.store(false, Ordering::SeqCst);
    }
}

impl RetroCore for LibretroCore {
    fn achievement_memory(&self, ram: &mut [u8]) -> AchievementMemory {
        let mut valid = AchievementMemory::default();
        if !self.loaded || ram.len() < ACHIEVEMENT_RAM_SIZE {
            return valid;
        }
        let game_boy = self.host.memory.iter().any(|d| d.start == 0xc000);
        // rcheevos consoleinfo.c: GB/GBC use native addresses plus fixed banks;
        // GBA uses IWRAM, EWRAM, then SRAM. Echo RAM aliases the fixed WRAM banks.
        // From Prashant Vaibhav's slot fork (pvaibhav/slot), GPL-3.0.
        let regions: &[(usize, usize, usize)] = if game_boy {
            &[
                (0x0000, 0x0000, 0x4000),
                (0x4000, 0x4000, 0x4000),
                (0x8000, 0x8000, 0x2000),
                (0xa000, 0xa000, 0x2000),
                (0xc000, 0xc000, 0x1000),
                (0xd000, 0xd000, 0x1000),
                (0xc000, 0xe000, 0x1000),
                (0xd000, 0xf000, 0x0e00),
                (0xfe00, 0xfe00, 0x00a0),
                (0xff00, 0xff00, 0x0080),
                (0xff80, 0xff80, 0x007f),
                (0xffff, 0xffff, 0x0001),
                (0x10000, 0x10000, 0x6000),
                (0x16000, 0x16000, 0x1e000),
            ]
        } else {
            &[
                (0x03000000, 0, 0x8000),
                (0x02000000, 0x8000, 0x40000),
                (0x0e000000, 0x48000, 0x10000),
            ]
        };
        for &(physical, offset, size) in regions {
            if let Some(d) =
                self.host.memory.iter().find(|d| {
                    d.start == physical && d.disconnect == 0 && !d.ptr.is_null() && d.len > 0
                })
            {
                let n = d.len.min(size);
                // The core owns the descriptors' memory until unload and no core function
                // runs concurrently. `offset` is the libretro offset from ptr to the region.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        (d.ptr as *const u8).add(d.offset),
                        ram[offset..].as_mut_ptr(),
                        n,
                    );
                }
                valid.push(offset..offset + n);
            } else if !game_boy && physical == 0x0e000000 {
                let p = unsafe { (self.api.get_memory_data)(MEMORY_SAVE_RAM) };
                let n = unsafe { (self.api.get_memory_size)(MEMORY_SAVE_RAM) }.min(size);
                if !p.is_null() && n > 0 {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            p as *const u8,
                            ram[offset..].as_mut_ptr(),
                            n,
                        );
                    }
                    valid.push(offset..offset + n);
                }
            }
        }
        valid
    }

    fn set_option(&mut self, key: &str, value: &str) {
        LibretroCore::set_option(self, key, value);
    }

    fn load(&mut self, rom: &Path) -> Result<(), CoreError> {
        self.unload();
        self.state_size = None;
        self.rom = std::fs::read(rom)?;
        self.rom_path = Some(
            CString::new(rom.as_os_str().as_encoded_bytes())
                .map_err(|_| CoreError::Load("rom path contains a nul".into()))?,
        );
        let info = GameInfo {
            path: self.rom_path.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            data: self.rom.as_ptr() as *const c_void,
            size: self.rom.len(),
            meta: ptr::null(),
        };
        let ok = {
            let _a = Active::bind(&mut self.host);
            unsafe { (self.api.load_game)(&info) }
        };
        if !ok {
            self.rom = Vec::new();
            self.rom_path = None;
            return Err(CoreError::Load(format!("core refused {}", rom.display())));
        }
        self.loaded = true;
        let mut av = SystemAvInfo::default();
        unsafe { (self.api.get_system_av_info)(&mut av) };
        self.av = AvInfo {
            fps: av.timing.fps,
            sample_rate: av.timing.sample_rate,
        };
        unsafe { (self.api.set_controller_port_device)(0, DEVICE_JOYPAD) };
        Ok(())
    }

    fn run_frame(&mut self, input: ButtonMask) {
        self.run_frame_linked(input, ButtonMask::default());
    }

    fn run_frame_linked(&mut self, p1: ButtonMask, p2: ButtonMask) {
        if !self.loaded {
            return;
        }
        self.host.inputs = [p1.0, p2.0];
        let _a = Active::bind(&mut self.host);
        unsafe { (self.api.run)() };
    }

    fn set_frame_skip(&mut self, skip: bool) {
        let _a = Active::bind(&mut self.host);
        unsafe { report_audio_status(skip) };
    }

    fn video_xrgb8888(&self) -> &[u8] {
        &self.host.video
    }

    fn video_size(&self) -> (u32, u32) {
        (self.host.video_w, self.host.video_h)
    }

    fn frame_size(&self) -> (u32, u32) {
        (self.host.shown_w, self.host.shown_h)
    }

    fn take_audio(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.host.audio)
    }

    fn serialize(&mut self) -> Result<Vec<u8>, CoreError> {
        let chip = unsafe { (self.api.get_memory_size)(MEMORY_SAVE_RAM) };
        if let Some(buf) = self
            .state_size
            .filter(|&(_, at)| at == chip)
            .and_then(|(size, _)| self.serialize_at(size))
        {
            return Ok(buf);
        }
        let size = unsafe { (self.api.serialize_size)() };
        self.state_size = (size > 0).then_some((size, chip));
        if size == 0 {
            return Err(CoreError::State("core reports no state".into()));
        }
        self.serialize_at(size)
            .ok_or_else(|| CoreError::State("serialize refused".into()))
    }

    fn unserialize(&mut self, data: &[u8]) -> Result<(), CoreError> {
        if data.is_empty() {
            return Err(CoreError::State("empty state".into()));
        }
        let ok = {
            let _a = Active::bind(&mut self.host);
            unsafe { (self.api.unserialize)(data.as_ptr() as *const c_void, data.len()) }
        };
        if ok {
            Ok(())
        } else {
            Err(CoreError::State("unserialize refused".into()))
        }
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        let mut sav = self.memory(MEMORY_SAVE_RAM)?.to_vec();
        if let Some(rtc) = self.memory(MEMORY_RTC) {
            sav.extend_from_slice(rtc);
        }
        Some(sav)
    }

    fn load_save_ram(&mut self, data: &[u8]) -> Result<(), CoreError> {
        let Some(ram) = self.memory_mut(MEMORY_SAVE_RAM) else {
            return Err(CoreError::Unsupported("core exposes no save ram".into()));
        };
        let n = ram.len().min(data.len());
        ram[..n].copy_from_slice(&data[..n]);
        let rest = &data[n..];
        if let Some(rtc) = self.memory_mut(MEMORY_RTC) {
            let m = rtc.len().min(rest.len());
            rtc[..m].copy_from_slice(&rest[..m]);
        }
        Ok(())
    }

    fn av_info(&self) -> AvInfo {
        self.av
    }

    fn rumble(&self) -> Rumble {
        self.host.rumble.clone()
    }

    fn net(&self) -> Link {
        self.host.net.clone()
    }

    fn start_link(&mut self, client_id: u16) {
        let _a = Active::bind(&mut self.host);
        unsafe { begin_link(client_id) };
    }

    fn pump_link(&mut self) {
        let _a = Active::bind(&mut self.host);
        unsafe { drain_link() };
    }

    fn stop_link(&mut self) {
        let _a = Active::bind(&mut self.host);
        unsafe { halt_link() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::ffi::CStr;

    fn host_with(options: HashMap<String, CString>, options_dirty: bool) -> Box<Host> {
        Box::new(Host {
            video: vec![0; VIDEO_BYTES],
            video_w: GBA_W,
            video_h: GBA_H,
            shown_w: GBA_W,
            shown_h: GBA_H,
            rotation: 0,
            held: Held::Turned(0),
            format: PixelFormat::Xrgb8888,
            audio: Vec::new(),
            inputs: [0; 2],
            system_dir: CString::new(".").unwrap(),
            save_dir: CString::new(".").unwrap(),
            rumble: Rumble::default(),
            asked_for_rumble: false,
            netpacket: None,
            net: Link::default(),
            net_peer: None,
            audio_status: None,
            options,
            options_dirty,
            declared: HashMap::new(),
            memory: Vec::new(),
        })
    }

    fn declaration(key: &str, value: &str) -> (Variable, CString, CString) {
        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();
        let var = Variable {
            key: key.as_ptr(),
            value: value.as_ptr(),
        };
        (var, key, value)
    }

    fn end_of_list() -> Variable {
        Variable {
            key: ptr::null(),
            value: ptr::null(),
        }
    }

    #[test]
    fn set_variables_records_what_the_core_declared() {
        let mut host = host_with(HashMap::new(), false);
        let (a, _ak, _av) = declaration("mgba_sgb_borders", "Use Super Game Boy Borders; ON|OFF");
        let (b, _bk, _bv) =
            declaration("mgba_gb_colors_preset", "Game Boy Palette Preset; 0|1|2|3");
        let list = [a, b, end_of_list()];
        let ok = {
            let _active = Active::bind(&mut host);
            unsafe { environment(SET_VARIABLES, list.as_ptr() as *mut c_void) }
        };

        assert!(ok);
        assert_eq!(
            host.declared.get("mgba_sgb_borders").map(Vec::as_slice),
            Some(["ON".to_string(), "OFF".to_string()].as_slice())
        );
        assert_eq!(
            host.declared
                .get("mgba_gb_colors_preset")
                .map(Vec::as_slice),
            Some(["0", "1", "2", "3"].map(str::to_string).as_slice())
        );
        assert_eq!(
            host.declared.len(),
            2,
            "the walk ran past the terminator and read whatever was after it"
        );
        assert!(
            !host.declared.contains_key("mgba_sgb_border"),
            "a key the core never declared came back declared"
        );
    }

    #[test]
    fn set_variables_with_no_list_declares_nothing() {
        let mut host = host_with(HashMap::new(), false);
        host.declared
            .insert("stale".to_string(), vec!["x".to_string()]);

        let ok = {
            let _active = Active::bind(&mut host);
            unsafe { environment(SET_VARIABLES, ptr::null_mut()) }
        };

        assert!(ok);
        assert!(host.declared.is_empty(), "a stale declaration survived");
    }

    #[test]
    fn get_variable_writes_the_options_pointer_for_the_core_to_read() {
        let mut options = HashMap::new();
        options.insert("gpsp_serial".to_string(), CString::new("rfu").unwrap());
        let mut host = host_with(options, false);
        let _active = Active::bind(&mut host);

        let key = CString::new("gpsp_serial").unwrap();
        let mut var = Variable {
            key: key.as_ptr(),
            value: ptr::null(),
        };
        let ok = unsafe { environment(GET_VARIABLE, &mut var as *mut Variable as *mut c_void) };

        assert!(ok);
        let value = unsafe { CStr::from_ptr(var.value) };
        assert_eq!(value.to_str().unwrap(), "rfu");
    }

    #[test]
    fn get_variable_reports_false_for_an_unknown_key() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);

        let key = CString::new("nope").unwrap();
        let mut var = Variable {
            key: key.as_ptr(),
            value: ptr::null(),
        };
        let ok = unsafe { environment(GET_VARIABLE, &mut var as *mut Variable as *mut c_void) };

        assert!(!ok);
        assert!(var.value.is_null());
    }

    #[test]
    fn get_variable_update_reports_and_clears_the_dirty_flag() {
        let mut host = host_with(HashMap::new(), true);
        let _active = Active::bind(&mut host);

        let mut dirty = false;
        let ok =
            unsafe { environment(GET_VARIABLE_UPDATE, &mut dirty as *mut bool as *mut c_void) };
        assert!(ok);
        assert!(dirty, "first call must report the pending change");

        let mut dirty_again = true;
        let ok = unsafe {
            environment(
                GET_VARIABLE_UPDATE,
                &mut dirty_again as *mut bool as *mut c_void,
            )
        };
        assert!(ok);
        assert!(!dirty_again, "flag must be cleared after being read once");
    }

    #[test]
    fn get_variable_refuses_a_null_key() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);

        let mut var = Variable {
            key: ptr::null(),
            value: ptr::null(),
        };
        let ok = unsafe { environment(GET_VARIABLE, &mut var as *mut Variable as *mut c_void) };
        assert!(!ok);
    }

    thread_local! {
        static TEST_AUDIO_STATUS: RefCell<Vec<(bool, c_uint, bool)>> =
            const { RefCell::new(Vec::new()) };
    }

    unsafe extern "C" fn test_audio_status(active: bool, occupancy: c_uint, underrun: bool) {
        TEST_AUDIO_STATUS.with(|r| r.borrow_mut().push((active, occupancy, underrun)));
    }

    #[test]
    fn set_audio_buffer_status_callback_stores_what_the_core_hands_over() {
        let mut host = host_with(HashMap::new(), false);
        let mut cb = AudioBufferStatusCallback {
            callback: Some(test_audio_status),
        };
        let ok = {
            let _active = Active::bind(&mut host);
            unsafe {
                environment(
                    SET_AUDIO_BUFFER_STATUS_CALLBACK,
                    &mut cb as *mut AudioBufferStatusCallback as *mut c_void,
                )
            }
        };

        assert!(ok, "refusing this turns the cores' frameskip off entirely");
        assert!(host.audio_status.is_some(), "the callback was never stored");
    }

    #[test]
    fn set_audio_buffer_status_callback_null_withdraws_it() {
        let mut host = host_with(HashMap::new(), false);
        host.audio_status = Some(test_audio_status);
        let ok = {
            let _active = Active::bind(&mut host);
            unsafe { environment(SET_AUDIO_BUFFER_STATUS_CALLBACK, ptr::null_mut()) }
        };

        assert!(ok, "withdrawing is a legal call and must be answered true");
        assert!(host.audio_status.is_none());
    }

    #[test]
    fn set_frame_skip_tells_the_core_to_skip_by_reporting_an_underrun() {
        TEST_AUDIO_STATUS.with(|r| r.borrow_mut().clear());
        let mut host = host_with(HashMap::new(), false);
        host.audio_status = Some(test_audio_status);
        {
            let _active = Active::bind(&mut host);
            unsafe {
                report_audio_status(true);
                report_audio_status(false);
            }
        }

        TEST_AUDIO_STATUS.with(|r| {
            assert_eq!(
                r.borrow().as_slice(),
                &[(true, 0, true), (true, 100, false)],
                "the skip and the draw did not read as an underrun and a healthy buffer"
            );
        });
    }

    #[test]
    fn set_frame_skip_is_a_noop_when_the_core_registered_no_callback() {
        TEST_AUDIO_STATUS.with(|r| r.borrow_mut().clear());
        let mut host = host_with(HashMap::new(), false);
        {
            let _active = Active::bind(&mut host);
            unsafe { report_audio_status(true) };
        }

        TEST_AUDIO_STATUS.with(|r| assert!(r.borrow().is_empty()));
    }

    thread_local! {
        static TEST_RECEIVED: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
        static TEST_RECEIVED_CLIENT_IDS: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
        static TEST_POLLS: Cell<u32> = const { Cell::new(0) };
        static TEST_START_CLIENT: Cell<Option<u16>> = const { Cell::new(None) };
        static TEST_STOP_CALLS: Cell<u32> = const { Cell::new(0) };
        static TEST_CONNECTED_CLIENT: Cell<Option<u16>> = const { Cell::new(None) };
        static TEST_DISCONNECTED_CLIENT: Cell<Option<u16>> = const { Cell::new(None) };
    }

    fn reset_test_netpacket_recorders() {
        TEST_RECEIVED.with(|r| r.borrow_mut().clear());
        TEST_RECEIVED_CLIENT_IDS.with(|c| c.borrow_mut().clear());
        TEST_POLLS.with(|p| p.set(0));
        TEST_START_CLIENT.with(|c| c.set(None));
        TEST_STOP_CALLS.with(|c| c.set(0));
        TEST_CONNECTED_CLIENT.with(|c| c.set(None));
        TEST_DISCONNECTED_CLIENT.with(|c| c.set(None));
    }

    unsafe extern "C" fn test_receive(buf: *const c_void, len: usize, client_id: u16) {
        let bytes = std::slice::from_raw_parts(buf as *const u8, len).to_vec();
        TEST_RECEIVED.with(|r| r.borrow_mut().push(bytes));
        TEST_RECEIVED_CLIENT_IDS.with(|c| c.borrow_mut().push(client_id));
    }

    unsafe extern "C" fn test_receive_reentrant(buf: *const c_void, len: usize, client_id: u16) {
        test_receive(buf, len, client_id);
        let reentrant = b"reentrant";
        netpacket_send(0, reentrant.as_ptr() as *const c_void, reentrant.len(), 0);
    }

    unsafe extern "C" fn test_start_reentrant(
        client_id: u16,
        _send: NetpacketSend,
        poll_receive: NetpacketPollReceive,
    ) {
        TEST_START_CLIENT.with(|c| c.set(Some(client_id)));
        poll_receive();
    }

    unsafe extern "C" fn test_poll() {
        TEST_POLLS.with(|p| p.set(p.get() + 1));
    }

    unsafe extern "C" fn test_start(
        client_id: u16,
        _send: NetpacketSend,
        _poll_receive: NetpacketPollReceive,
    ) {
        TEST_START_CLIENT.with(|c| c.set(Some(client_id)));
    }

    unsafe extern "C" fn test_stop() {
        TEST_STOP_CALLS.with(|c| c.set(c.get() + 1));
    }

    unsafe extern "C" fn test_connected(client_id: u16) -> bool {
        TEST_CONNECTED_CLIENT.with(|c| c.set(Some(client_id)));
        true
    }

    unsafe extern "C" fn test_disconnected(client_id: u16) {
        TEST_DISCONNECTED_CLIENT.with(|c| c.set(Some(client_id)));
    }

    fn test_netpacket_callback() -> NetpacketCallback {
        NetpacketCallback {
            start: Some(test_start),
            receive: Some(test_receive),
            stop: None,
            poll: Some(test_poll),
            connected: None,
            disconnected: None,
            protocol_version: ptr::null(),
        }
    }

    #[test]
    fn set_netpacket_interface_stores_the_callback_the_core_hands_over() {
        let mut host = host_with(HashMap::new(), false);
        let mut cb = test_netpacket_callback();
        let ok = {
            let _active = Active::bind(&mut host);
            unsafe {
                environment(
                    SET_NETPACKET_INTERFACE,
                    &mut cb as *mut NetpacketCallback as *mut c_void,
                )
            }
        };

        assert!(ok);
        assert!(host.netpacket.is_some(), "the callback was never stored");
    }

    #[test]
    fn set_netpacket_interface_null_data_withdraws_the_interface() {
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        let ok = {
            let _active = Active::bind(&mut host);
            unsafe { environment(SET_NETPACKET_INTERFACE, ptr::null_mut()) }
        };

        assert!(ok, "withdrawing is a legal call and must be answered true");
        assert!(
            host.netpacket.is_none(),
            "a NULL data pointer must clear a previously registered callback"
        );
    }

    #[test]
    fn netpacket_send_pushes_the_cores_packet_onto_outbound() {
        let mut host = host_with(HashMap::new(), false);
        let packet = b"link cable byte";
        {
            let _active = Active::bind(&mut host);
            unsafe {
                netpacket_send(
                    NETPACKET_RELIABLE,
                    packet.as_ptr() as *const c_void,
                    packet.len(),
                    0,
                )
            };
        }

        assert_eq!(host.net.take_outbound().as_deref(), Some(&packet[..]));
        assert_eq!(host.net.take_outbound(), None, "only one packet was sent");
    }

    #[test]
    fn netpacket_send_ignores_a_null_or_empty_packet() {
        let mut host = host_with(HashMap::new(), false);
        {
            let _active = Active::bind(&mut host);
            unsafe { netpacket_send(0, ptr::null(), 4, 0) };
            unsafe { netpacket_send(0, [1u8].as_ptr() as *const c_void, 0, 0) };
        }

        assert_eq!(
            host.net.take_outbound(),
            None,
            "a null buffer or zero length must not enqueue a phantom packet"
        );
    }

    #[test]
    fn netpacket_poll_receive_drains_inbound_into_the_cores_receive_in_order() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        host.net.set_active(true);
        host.net_peer = Some(1);
        host.net.push_inbound(b"first".to_vec());
        host.net.push_inbound(b"second".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { netpacket_poll_receive() };
        }

        TEST_RECEIVED.with(|r| {
            assert_eq!(
                r.borrow().as_slice(),
                &[b"first".to_vec(), b"second".to_vec()],
                "order was not kept"
            );
        });
        assert_eq!(host.net.take_inbound(), None, "queue must be drained");
    }

    #[test]
    fn netpacket_poll_receive_does_nothing_without_a_receive_callback() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.net.set_active(true);
        host.net.push_inbound(b"stranded".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { netpacket_poll_receive() };
        }

        TEST_RECEIVED.with(|r| assert!(r.borrow().is_empty()));
        assert_eq!(
            host.net.take_inbound().as_deref(),
            Some(&b"stranded"[..]),
            "with nowhere to hand the packet it must be left queued, not dropped"
        );
    }

    #[test]
    fn netpacket_poll_receive_does_nothing_once_the_session_is_no_longer_active() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        host.net.push_inbound(b"stale".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { netpacket_poll_receive() };
        }

        TEST_RECEIVED.with(|r| assert!(r.borrow().is_empty(), "an ended session reached the core"));
        assert_eq!(
            host.net.take_inbound().as_deref(),
            Some(&b"stale"[..]),
            "the packet must be left queued, not delivered to a session that already ended"
        );
    }

    #[test]
    fn netpacket_poll_receive_does_nothing_without_a_recorded_peer() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        host.net.set_active(true);
        host.net.push_inbound(b"orphaned".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { netpacket_poll_receive() };
        }

        TEST_RECEIVED.with(|r| {
            assert!(
                r.borrow().is_empty(),
                "a packet must not be delivered with no peer to tag it"
            )
        });
        assert_eq!(
            host.net.take_inbound().as_deref(),
            Some(&b"orphaned"[..]),
            "the packet must be left queued, not delivered mislabelled as our own id"
        );
    }

    #[test]
    fn netpacket_poll_receive_tags_packets_with_the_peers_client_id() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(0) };
        }
        reset_test_netpacket_recorders();
        host.net.push_inbound(b"from the peer".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { netpacket_poll_receive() };
        }

        TEST_RECEIVED_CLIENT_IDS.with(|c| {
            assert_eq!(
                c.borrow().as_slice(),
                &[1],
                "packets must be tagged with the peer's id, not the host's own"
            );
        });
    }

    #[test]
    fn drain_link_hands_the_core_everything_waiting_then_polls() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        host.net.set_active(true);
        host.net_peer = Some(1);
        host.net.push_inbound(b"queued".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_RECEIVED.with(|r| assert_eq!(r.borrow().as_slice(), &[b"queued".to_vec()]));
        TEST_POLLS.with(|p| assert_eq!(p.get(), 1, "poll must run once a frame"));
    }

    #[test]
    fn drain_link_polls_even_with_nothing_inbound() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        host.net.set_active(true);
        host.net_peer = Some(1);
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_POLLS.with(|p| assert_eq!(p.get(), 1));
    }

    #[test]
    fn drain_link_skips_poll_when_the_core_offered_none() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            poll: None,
            ..test_netpacket_callback()
        });
        host.net.set_active(true);
        host.net_peer = Some(1);
        host.net.push_inbound(b"still delivered".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_RECEIVED.with(|r| {
            assert_eq!(r.borrow().as_slice(), &[b"still delivered".to_vec()]);
        });
        TEST_POLLS.with(|p| assert_eq!(p.get(), 0, "poll is optional and was not offered"));
    }

    #[test]
    fn drain_link_is_a_noop_when_the_core_never_registered_netpacket() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.net.set_active(true);
        host.net.push_inbound(b"nobody asked".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_RECEIVED.with(|r| assert!(r.borrow().is_empty()));
        TEST_POLLS.with(|p| assert_eq!(p.get(), 0));
        assert_eq!(
            host.net.take_inbound().as_deref(),
            Some(&b"nobody asked"[..]),
            "with no registered core the packet must be left queued, not lost"
        );
    }

    #[test]
    fn drain_link_does_nothing_once_the_session_is_no_longer_active() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        host.net.push_inbound(b"stale".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_RECEIVED.with(|r| assert!(r.borrow().is_empty(), "an ended session reached the core"));
        TEST_POLLS.with(|p| assert_eq!(p.get(), 0, "an ended session must not be polled either"));
        assert_eq!(
            host.net.take_inbound().as_deref(),
            Some(&b"stale"[..]),
            "the packet must be left queued, not delivered to a session that already ended"
        );
    }

    #[test]
    fn drain_link_does_nothing_without_a_recorded_peer() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        host.net.set_active(true);
        host.net.push_inbound(b"orphaned".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_RECEIVED.with(|r| {
            assert!(
                r.borrow().is_empty(),
                "a packet must not be delivered with no peer to tag it"
            )
        });
        TEST_POLLS.with(|p| {
            assert_eq!(
                p.get(),
                0,
                "must not even poll with no peer to tag a receive"
            )
        });
        assert_eq!(
            host.net.take_inbound().as_deref(),
            Some(&b"orphaned"[..]),
            "the packet must be left queued, not delivered mislabelled as our own id"
        );
    }

    #[test]
    fn drain_link_tags_packets_with_the_peers_client_id() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(0) };
        }
        reset_test_netpacket_recorders();
        host.net.push_inbound(b"from the peer".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_RECEIVED_CLIENT_IDS.with(|c| {
            assert_eq!(
                c.borrow().as_slice(),
                &[1],
                "packets must be tagged with the peer's id, not our own"
            );
        });
    }

    #[test]
    fn drain_link_survives_the_reentrancy_gpsp_documents() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            receive: Some(test_receive_reentrant),
            ..test_netpacket_callback()
        });
        host.net.set_active(true);
        host.net_peer = Some(1);
        host.net.push_inbound(b"queued".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { drain_link() };
        }

        TEST_RECEIVED.with(|r| assert_eq!(r.borrow().as_slice(), &[b"queued".to_vec()]));
        assert_eq!(
            host.net.take_outbound().as_deref(),
            Some(&b"reentrant"[..]),
            "the reentrant netpacket_send call must still reach outbound"
        );
    }

    #[test]
    fn netpacket_poll_receive_survives_the_reentrancy_gpsp_documents() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            receive: Some(test_receive_reentrant),
            ..test_netpacket_callback()
        });
        host.net.set_active(true);
        host.net_peer = Some(1);
        host.net.push_inbound(b"queued".to_vec());
        {
            let _active = Active::bind(&mut host);
            unsafe { netpacket_poll_receive() };
        }

        assert_eq!(host.net.take_outbound().as_deref(), Some(&b"reentrant"[..]));
    }

    #[test]
    fn begin_link_survives_a_core_that_reenters_from_start() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            start: Some(test_start_reentrant),
            ..test_netpacket_callback()
        });
        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(0) };
        }

        TEST_START_CLIENT.with(|c| assert_eq!(c.get(), Some(0)));
        assert!(host.net.is_active());
    }

    #[test]
    fn begin_link_hands_the_core_its_client_id_and_marks_the_session_active() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());
        assert!(
            !host.net.is_active(),
            "a fresh link is not backing a session"
        );
        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(1) };
        }

        TEST_START_CLIENT.with(|c| assert_eq!(c.get(), Some(1), "client id was not forwarded"));
        assert!(
            host.net.is_active(),
            "starting a session must mark the link active"
        );
    }

    #[test]
    fn begin_link_is_a_noop_when_the_core_never_registered_netpacket() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(0) };
        }

        TEST_START_CLIENT.with(|c| assert_eq!(c.get(), None, "nothing to start, nothing called"));
        assert!(!host.net.is_active());
    }

    #[test]
    fn begin_link_calls_connected_with_the_peers_client_id() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            connected: Some(test_connected),
            ..test_netpacket_callback()
        });
        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(0) };
        }

        TEST_CONNECTED_CLIENT.with(|c| {
            assert_eq!(
                c.get(),
                Some(1),
                "connected was not called with the peer's id"
            )
        });
    }

    #[test]
    fn begin_link_is_fine_with_no_connected_callback() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());

        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(0) };
        }

        assert!(
            host.net.is_active(),
            "a missing connected callback must not stop the session from starting"
        );
    }

    #[test]
    fn halt_link_calls_the_cores_stop_when_it_registered_one() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            stop: Some(test_stop),
            ..test_netpacket_callback()
        });
        {
            let _active = Active::bind(&mut host);
            unsafe { halt_link() };
        }

        TEST_STOP_CALLS.with(|c| assert_eq!(c.get(), 1, "stop was never called"));
    }

    #[test]
    fn halt_link_is_a_noop_when_the_core_never_offered_a_stop() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(test_netpacket_callback());

        {
            let _active = Active::bind(&mut host);
            unsafe { halt_link() };
        }

        TEST_STOP_CALLS.with(|c| assert_eq!(c.get(), 0));
    }

    #[test]
    fn halt_link_calls_disconnected_with_the_peers_client_id() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            disconnected: Some(test_disconnected),
            ..test_netpacket_callback()
        });
        {
            let _active = Active::bind(&mut host);
            unsafe { begin_link(1) };
        }
        {
            let _active = Active::bind(&mut host);
            unsafe { halt_link() };
        }

        TEST_DISCONNECTED_CLIENT.with(|c| {
            assert_eq!(
                c.get(),
                Some(0),
                "disconnected was not called with the peer's id"
            )
        });
    }

    #[test]
    fn halt_link_does_not_call_disconnected_when_no_session_ever_started() {
        reset_test_netpacket_recorders();
        let mut host = host_with(HashMap::new(), false);
        host.netpacket = Some(NetpacketCallback {
            disconnected: Some(test_disconnected),
            ..test_netpacket_callback()
        });

        {
            let _active = Active::bind(&mut host);
            unsafe { halt_link() };
        }

        TEST_DISCONNECTED_CLIENT.with(|c| assert_eq!(c.get(), None, "nothing was ever connected"));
    }

    #[test]
    fn input_state_answers_each_port_from_its_own_mask() {
        let mut host = host_with(HashMap::new(), false);
        host.inputs = [ButtonMask::A, ButtonMask::B | ButtonMask::START];
        let _active = Active::bind(&mut host);

        let mask = |port| unsafe { input_state(port, DEVICE_JOYPAD, 0, JOYPAD_MASK) } as u16;
        assert_eq!(mask(0), ButtonMask::A);
        assert_eq!(mask(1), ButtonMask::B | ButtonMask::START);
    }

    #[test]
    fn input_state_answers_single_buttons_on_port_1() {
        let mut host = host_with(HashMap::new(), false);
        host.inputs = [0, ButtonMask::B];
        let _active = Active::bind(&mut host);

        let button = |id| unsafe { input_state(1, DEVICE_JOYPAD, 0, id) };
        assert_eq!(button(0), 1, "B is bit 0 of the libretro joypad");
        assert_eq!(button(8), 0, "A is not held on port 1");
    }

    #[test]
    fn input_state_answers_nothing_past_port_1() {
        let mut host = host_with(HashMap::new(), false);
        host.inputs = [u16::MAX, u16::MAX];
        let _active = Active::bind(&mut host);

        assert_eq!(unsafe { input_state(2, DEVICE_JOYPAD, 0, JOYPAD_MASK) }, 0);
    }

    #[test]
    fn a_small_picture_is_centred_in_the_buffer() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);
        let frame = vec![0xffu8; 160 * 144 * 4];
        unsafe { video_refresh(frame.as_ptr() as *const c_void, 160, 144, 160 * 4) };

        let lit =
            |x: usize, y: usize| unsafe { with_host(|h| h.video[(y * 240 + x) * 4]) }.unwrap() != 0;
        assert!(lit(40, 8), "the top left of the picture is not at (40, 8)");
        assert!(lit(199, 151), "the bottom right is not at (199, 151)");
        assert!(!lit(39, 8), "the picture starts one column too early");
        assert!(!lit(40, 7), "the picture starts one row too early");
    }

    #[test]
    fn the_margin_is_cleared_when_the_picture_shrinks() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);
        unsafe {
            let full = vec![0xffu8; 240 * 160 * 4];
            video_refresh(full.as_ptr() as *const c_void, 240, 160, 240 * 4);
            let small = vec![0x11u8; 160 * 144 * 4];
            video_refresh(small.as_ptr() as *const c_void, 160, 144, 160 * 4);
        }
        let corner = unsafe { with_host(|h| h.video[0]) }.unwrap();
        assert_eq!(corner, 0, "the previous picture is still in the margin");
    }

    fn gradient(w: usize, h: usize) -> Vec<u8> {
        let mut px = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let o = (y * w + x) * 4;
                px[o] = x as u8;
                px[o + 1] = y as u8;
                px[o + 2] = (x ^ y) as u8;
            }
        }
        px
    }

    #[test]
    fn a_full_size_picture_is_unmoved() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);
        let frame = gradient(GBA_W as usize, GBA_H as usize);
        unsafe {
            video_refresh(
                frame.as_ptr() as *const c_void,
                GBA_W,
                GBA_H,
                GBA_W as usize * 4,
            )
        };

        assert_eq!(unsafe { with_host(|h| h.video.clone()) }.unwrap(), frame);
    }

    #[test]
    fn a_tall_picture_fits_a_bigger_buffer_whole() {
        let mut host = host_with(HashMap::new(), false);
        host.video_w = 360;
        host.video_h = 240;
        host.video = vec![0; 360 * 240 * 4];
        let _active = Active::bind(&mut host);
        let frame = vec![0xffu8; 144 * 224 * 4];
        unsafe { video_refresh(frame.as_ptr() as *const c_void, 144, 224, 144 * 4) };

        let lit =
            |x: usize, y: usize| unsafe { with_host(|h| h.video[(y * 360 + x) * 4]) }.unwrap() != 0;
        assert!(lit(108, 8), "the top left is not at (108, 8)");
        assert!(lit(251, 231), "the bottom right is not at (251, 231)");
        assert!(!lit(107, 8), "the picture starts one column too early");
        assert!(!lit(108, 7), "the picture starts one row too early");
        assert!(!lit(108, 232), "the picture runs one row too long");
        assert_eq!(
            unsafe { with_host(|h| (h.shown_w, h.shown_h)) }.unwrap(),
            (144, 224)
        );
    }

    #[test]
    fn a_tall_picture_is_still_cropped_to_the_gba_buffer() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);
        let frame = vec![0xffu8; 144 * 224 * 4];
        unsafe { video_refresh(frame.as_ptr() as *const c_void, 144, 224, 144 * 4) };
        assert_eq!(
            unsafe { with_host(|h| (h.video.len(), h.shown_w, h.shown_h)) }.unwrap(),
            ((GBA_W * GBA_H * 4) as usize, 144, GBA_H)
        );
    }

    #[test]
    fn set_rotation_stores_the_quarter_turns_the_core_asks_for() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);
        let mut turns: c_uint = 1;
        let ok = unsafe { environment(SET_ROTATION, &mut turns as *mut c_uint as *mut c_void) };
        assert!(ok, "a core that asks to be turned has to be answered");
        assert_eq!(unsafe { with_host(|h| h.rotation) }.unwrap(), 1);

        let mut nonsense: c_uint = 4;
        let ok = unsafe { environment(SET_ROTATION, &mut nonsense as *mut c_uint as *mut c_void) };
        assert!(!ok, "there are only four quarter turns");
        assert_eq!(unsafe { with_host(|h| h.rotation) }.unwrap(), 1);
        assert!(!unsafe { environment(SET_ROTATION, ptr::null_mut()) });
    }

    /// A 3 wide, 2 tall frame whose pixels are numbered 1 to 6, row by row,
    /// drawn turned. Returns what lands at each place in the shown picture.
    fn turned(rotation: u32) -> (Vec<Vec<u8>>, (u32, u32)) {
        drawn(rotation, false)
    }

    /// The same frame, on a handheld held turned a quarter anticlockwise.
    fn drawn_on_turned_handheld(rotation: u32) -> (Vec<Vec<u8>>, (u32, u32)) {
        drawn_held(rotation, "ccw")
    }

    fn drawn(rotation: u32, _: bool) -> (Vec<Vec<u8>>, (u32, u32)) {
        drawn_held(rotation, "off")
    }

    fn drawn_held(rotation: u32, held: &str) -> (Vec<Vec<u8>>, (u32, u32)) {
        let mut host = host_with(HashMap::new(), false);
        host.rotation = rotation;
        host.held = Held::parse(held);
        let _active = Active::bind(&mut host);
        let mut frame = vec![0u8; 3 * 2 * 4];
        for n in 0..6 {
            frame[n * 4] = n as u8 + 1;
        }
        unsafe { video_refresh(frame.as_ptr() as *const c_void, 3, 2, 3 * 4) };
        let (w, h) = unsafe { with_host(|h| (h.shown_w, h.shown_h)) }.unwrap();
        let (ox, oy) = ((GBA_W - w) / 2, (GBA_H - h) / 2);
        let rows = (0..h)
            .map(|y| {
                (0..w)
                    .map(|x| {
                        let i = (((oy + y) * GBA_W + ox + x) * 4) as usize;
                        unsafe { with_host(|h| h.video[i]) }.unwrap()
                    })
                    .collect()
            })
            .collect();
        (rows, (w, h))
    }

    #[test]
    fn a_frame_turned_one_quarter_anticlockwise_swaps_its_size() {
        let (rows, size) = turned(1);
        assert_eq!(size, (2, 3));
        assert_eq!(rows, vec![vec![3, 6], vec![2, 5], vec![1, 4]]);
    }

    #[test]
    fn a_frame_turned_three_quarters_anticlockwise_is_a_clockwise_quarter() {
        let (rows, size) = turned(3);
        assert_eq!(size, (2, 3));
        assert_eq!(rows, vec![vec![4, 1], vec![5, 2], vec![6, 3]]);
    }

    #[test]
    fn a_frame_turned_a_half_keeps_its_size_and_reverses_it() {
        let (rows, size) = turned(2);
        assert_eq!(size, (3, 2));
        assert_eq!(rows, vec![vec![6, 5, 4], vec![3, 2, 1]]);
    }

    #[test]
    fn an_upright_frame_is_drawn_as_it_always_was() {
        let (rows, size) = turned(0);
        assert_eq!(size, (3, 2));
        assert_eq!(rows, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }

    #[test]
    fn a_game_the_core_turns_left_is_drawn_as_it_is_on_a_turned_handheld() {
        let (rows, size) = drawn_on_turned_handheld(1);
        assert_eq!(size, (3, 2));
        assert_eq!(rows, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }

    #[test]
    fn a_game_the_core_turns_right_is_drawn_upside_down_on_a_turned_handheld() {
        let (rows, size) = drawn_on_turned_handheld(3);
        assert_eq!(size, (3, 2));
        assert_eq!(rows, vec![vec![6, 5, 4], vec![3, 2, 1]]);
    }

    #[test]
    fn an_upright_game_on_a_turned_handheld_is_turned_back_to_upright() {
        let (rows, size) = drawn_on_turned_handheld(0);
        assert_eq!(size, (2, 3));
        assert_eq!(rows, vec![vec![4, 1], vec![5, 2], vec![6, 3]]);
    }

    #[test]
    fn each_hold_draws_the_turn_the_core_asks_less_its_own() {
        for (held, turns) in [("off", 0), ("ccw", 1), ("180", 2), ("cw", 3)] {
            for asked in 0..4 {
                let mut host = host_with(HashMap::new(), false);
                host.rotation = asked;
                host.held = Held::parse(held);
                assert_eq!(
                    drawn_turns(&host),
                    (asked + 4 - turns) % 4,
                    "{held}, asked {asked}"
                );
            }
        }
    }

    #[test]
    fn held_the_way_the_core_asks_the_picture_is_drawn_as_it_is() {
        for asked in 0..4 {
            let mut host = host_with(HashMap::new(), false);
            host.rotation = asked;
            host.held = Held::Native;
            assert_eq!(drawn_turns(&host), 0, "asked {asked}");
        }
        let (rows, _) = drawn_held(3, "native");
        assert_eq!(rows, vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let (rows, _) = drawn_held(3, "cw");
        assert_eq!(
            rows,
            vec![vec![1, 2, 3], vec![4, 5, 6]],
            "the same as cw for a right-turned game"
        );
    }

    #[test]
    fn the_pad_is_turned_a_quarter_to_follow_a_turned_handheld() {
        let pad = turned_pad;
        assert_eq!(pad(ButtonMask::RIGHT), ButtonMask::UP);
        assert_eq!(pad(ButtonMask::DOWN), ButtonMask::RIGHT);
        assert_eq!(pad(ButtonMask::LEFT), ButtonMask::DOWN);
        assert_eq!(pad(ButtonMask::UP), ButtonMask::LEFT);
        assert_eq!(
            pad(ButtonMask::UP | ButtonMask::RIGHT),
            ButtonMask::LEFT | ButtonMask::UP,
            "a diagonal turns with the pad"
        );
        let others = ButtonMask::A
            | ButtonMask::B
            | ButtonMask::L
            | ButtonMask::R
            | ButtonMask::START
            | ButtonMask::SELECT;
        assert_eq!(pad(others | ButtonMask::DOWN), others | ButtonMask::RIGHT);
    }

    fn pad_seen(handheld_turn: bool, rotation: u32, held: u16) -> (i16, i16) {
        pad_held(if handheld_turn { "ccw" } else { "off" }, rotation, held)
    }

    fn pad_held(hold: &str, rotation: u32, held: u16) -> (i16, i16) {
        let mut host = host_with(HashMap::new(), false);
        host.held = Held::parse(hold);
        host.rotation = rotation;
        host.inputs[0] = held;
        let _active = Active::bind(&mut host);
        unsafe {
            (
                input_state(0, DEVICE_JOYPAD, 0, JOYPAD_MASK),
                input_state(0, DEVICE_JOYPAD, 0, 4),
            )
        }
    }

    #[test]
    fn a_core_that_turns_its_game_sees_the_turned_pad_on_a_turned_handheld() {
        let held = ButtonMask::RIGHT | ButtonMask::A;
        let (mask, up) = pad_seen(true, 1, held);
        assert_eq!(mask as u16, ButtonMask::UP | ButtonMask::A);
        assert_eq!(up, 1, "pad right reads as up, one id at a time too");
    }

    #[test]
    fn the_pad_turns_with_the_handheld_and_not_otherwise() {
        let held = ButtonMask::RIGHT | ButtonMask::A;
        let (mask, up) = pad_seen(false, 1, held);
        assert_eq!(mask as u16, held, "an upright handheld");
        assert_eq!(up, 0);
        let (mask, _) = pad_held("180", 0, held);
        assert_eq!(
            mask as u16,
            ButtonMask::LEFT | ButtonMask::A,
            "upside down, right is left"
        );
        let (mask, _) = pad_held("cw", 0, held);
        assert_eq!(
            mask as u16,
            ButtonMask::DOWN | ButtonMask::A,
            "a quarter clockwise, right is down"
        );
        for asked in 0..4 {
            let (mask, _) = pad_held("native", asked, held);
            let (turned, _) = pad_held(["off", "ccw", "180", "cw"][asked as usize], asked, held);
            assert_eq!(
                mask, turned,
                "native is the hold the core asks for, {asked}"
            );
        }
    }

    #[test]
    fn a_turned_565_frame_is_widened_like_an_upright_one() {
        let mut host = host_with(HashMap::new(), false);
        host.rotation = 1;
        host.format = PixelFormat::Rgb565;
        let _active = Active::bind(&mut host);
        let frame: Vec<u8> = [0xffffu16, 0x0000]
            .iter()
            .flat_map(|p| p.to_le_bytes())
            .collect();
        unsafe { video_refresh(frame.as_ptr() as *const c_void, 2, 1, 2 * 2) };
        let (ox, oy) = ((GBA_W - 1) / 2, (GBA_H - 2) / 2);
        let at = |y: u32| {
            let i = (((oy + y) * GBA_W + ox) * 4) as usize;
            unsafe { with_host(|h| h.video[i..i + 4].to_vec()) }.unwrap()
        };
        assert_eq!(at(0), vec![0, 0, 0, 0], "the right pixel turns to the top");
        assert_eq!(at(1), vec![255, 255, 255, 0]);
    }

    #[test]
    fn get_can_dupe_tells_a_core_the_frontend_keeps_the_last_frame() {
        assert_eq!(GET_CAN_DUPE, 3);

        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);

        let mut can_dupe = false;
        let ok = unsafe { environment(GET_CAN_DUPE, &mut can_dupe as *mut bool as *mut c_void) };

        assert!(ok, "a core that asks has to be answered, not ignored");
        assert!(
            can_dupe,
            "the frontend does keep the last frame, so it must say so"
        );
    }

    #[test]
    fn get_can_dupe_refuses_a_null_pointer() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);

        assert!(!unsafe { environment(GET_CAN_DUPE, ptr::null_mut()) });
    }

    #[test]
    fn a_duplicate_frame_leaves_the_previous_picture_exactly_as_it_was() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);
        let frame = gradient(GBA_W as usize, GBA_H as usize);
        unsafe {
            video_refresh(
                frame.as_ptr() as *const c_void,
                GBA_W,
                GBA_H,
                GBA_W as usize * 4,
            );
            video_refresh(ptr::null(), GBA_W, GBA_H, GBA_W as usize * 4);
        }

        assert_eq!(
            unsafe { with_host(|h| h.video.clone()) }.unwrap(),
            frame,
            "a duplicate frame changed the picture instead of keeping it"
        );
    }

    #[test]
    fn a_duplicate_frame_keeps_a_centred_game_boy_picture_where_it_is() {
        let mut host = host_with(HashMap::new(), false);
        let _active = Active::bind(&mut host);
        let frame = gradient(160, 144);
        unsafe { video_refresh(frame.as_ptr() as *const c_void, 160, 144, 160 * 4) };
        let drawn = unsafe { with_host(|h| h.video.clone()) }.unwrap();

        unsafe {
            video_refresh(ptr::null(), 160, 144, 160 * 4);
            video_refresh(ptr::null(), 160, 144, 160 * 4);
        }

        let kept = unsafe { with_host(|h| h.video.clone()) }.unwrap();
        assert_eq!(kept, drawn, "the picture did not survive two duplicates");
        let lit = (58 * GBA_W as usize + 140) * 4;
        assert_eq!(
            &kept[lit..lit + 3],
            &[100u8, 50, 100 ^ 50],
            "the picture was blanked or moved where a duplicate should have kept it"
        );
    }
}
