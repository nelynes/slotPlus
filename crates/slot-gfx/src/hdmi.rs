//! HDMI output on the H700, through the sunxi disp2 display driver of the 4.9
//! BSP kernel BaseOS runs.
//!
//! The driver has no clean way to move the picture to a TV. Its only control
//! is the dispdbg debugfs interface, which BaseOS mounts for this: it flips
//! disp0 between the panel and the HDMI transmitter. The framebuffer then has
//! to be resized to match, and because the driver's fbdev glue never rebuilds
//! the scanout layer itself, the layer is rewritten through /dev/disp too.
//!
//! The sequence, its timings and its parameters follow the H700 port of
//! NextUI (pvaibhav/NextUI, workspace/h700/libmsettings/msettings.c), which
//! worked them out on this hardware: a 1280x720 framebuffer that the display
//! engine scales up to a 1080p60 signal.
//!
//! The Mali EGL driver fixes the window size when it starts, so the switch
//! must happen before any EGL surface exists. Slot switches at start-up, and
//! when the cable changes it saves and exits for BaseOS to start it again.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The framebuffer Slot draws into while on HDMI.
pub const HDMI_FB: (u32, u32) = (1280, 720);

/// The signal the display engine scales that framebuffer up to.
const HDMI_SIGNAL: (u32, u32) = (1920, 1080);

/// dispdbg switch parameters: output type 4 (HDMI), mode 10 (1080p60), then
/// the colour format and depth the H700 NextUI port uses.
const SWITCH_TO_HDMI: &str = "4 10 0 0 0x4 0x101 0 0 0 8";

/// Output type 1: the panel.
const SWITCH_TO_PANEL: &str = "1 0";

/// Where the kernel's files are, so tests can point at a made-up tree.
#[derive(Clone, Debug)]
pub struct Paths {
    pub sys: PathBuf,
    pub dev: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Paths {
            sys: PathBuf::from("/sys"),
            dev: PathBuf::from("/dev"),
        }
    }
}

impl Paths {
    fn dispdbg(&self) -> PathBuf {
        self.sys.join("kernel/debug/dispdbg")
    }

    /// Whether this kernel has the display driver Slot knows how to switch.
    pub fn switchable(&self) -> bool {
        self.dispdbg().join("command").exists() && self.dev.join("disp").exists()
    }

    /// Whether an HDMI cable is plugged in.
    pub fn cable_in(&self) -> bool {
        let extcon = self.sys.join("class/extcon/hdmi");
        ["state", "cable.0/state", "cable.1/state"]
            .iter()
            .any(|f| reads_on(&extcon.join(f)))
    }

    /// Whether the display driver is sending the picture to HDMI right now.
    pub fn on_hdmi(&self) -> bool {
        std::fs::read_to_string(self.sys.join("class/disp/disp/attr/sys"))
            .is_ok_and(|s| s.contains("hdmi output"))
    }
}

/// An extcon state file reads "1", or "HDMI=1", when the cable is in.
fn reads_on(path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    text.lines().any(|line| {
        let value = line.rsplit('=').next().unwrap_or("").trim();
        value.parse::<u32>().is_ok_and(|v| v > 0)
    })
}

/// The panel's own framebuffer size, for switching back to it. BaseOS names
/// the device in BASEOS_TARGET; the sizes are the ones its device list gives.
pub fn panel_for(target: Option<&str>) -> (u32, u32) {
    match target.unwrap_or("") {
        "rgcubexx" => (720, 720),
        "rg34xx" | "rg34xxsp" => (720, 480),
        // A portrait panel, turned by Slot.
        "rg28xx" => (480, 640),
        _ => (640, 480),
    }
}

/// Put the picture where the cable says it should be: on HDMI if one is
/// plugged in, on the panel if not. Returns whether the picture is on HDMI
/// afterwards. Does nothing on a kernel without the sunxi display driver.
pub fn sync(paths: &Paths, panel: (u32, u32)) -> bool {
    if !paths.switchable() {
        return false;
    }
    let want = paths.cable_in();
    if paths.on_hdmi() != want {
        eprintln!(
            "slot: hdmi: cable {}, switching to the {}",
            if want { "in" } else { "out" },
            if want { "TV" } else { "panel" }
        );
        switch(paths, want, panel);
    }
    let on = paths.on_hdmi();
    eprintln!(
        "slot: hdmi: cable {}, picture on the {}",
        if paths.cable_in() { "in" } else { "out" },
        if on { "TV" } else { "panel" }
    );
    on
}

/// Move the picture to HDMI or back to the panel. Must run while no EGL
/// surface exists.
pub fn switch(paths: &Paths, to_hdmi: bool, panel: (u32, u32)) {
    let began = Instant::now();
    let blank = paths.sys.join("class/graphics/fb0/blank");
    put(&blank, "4");
    clear_framebuffer(&paths.dev.join("fb0"));
    let dbg = paths.dispdbg();
    put(&dbg.join("name"), "disp0");
    put(&dbg.join("command"), "switch");
    put(
        &dbg.join("param"),
        if to_hdmi {
            SWITCH_TO_HDMI
        } else {
            SWITCH_TO_PANEL
        },
    );
    put(&dbg.join("start"), "1");
    // The switch runs in the background, and bringing the panel back alone
    // takes most of a second. The framebuffer must not be resized until the
    // new output is attached, or the layer is rebuilt against the old one.
    let attached = wait_for(|| paths.on_hdmi() == to_hdmi, Duration::from_secs(3));
    std::thread::sleep(Duration::from_millis(150));
    if !attached {
        eprintln!("slot: hdmi: the driver did not report the new output in 3 s");
    }
    let (fb, out) = match to_hdmi {
        true => (HDMI_FB, HDMI_SIGNAL),
        false => (panel, panel),
    };
    let fb0 = paths.dev.join("fb0");
    set_framebuffer_size(&fb0, fb);
    commit_layer(&paths.dev.join("disp"), fb, out);
    std::thread::sleep(Duration::from_millis(250));
    put(&blank, "0");
    // Unblanking can make the driver restore an older layer; set it again.
    commit_layer(&paths.dev.join("disp"), fb, out);
    eprintln!(
        "slot: hdmi: on the {} at {}x{} in {} ms",
        if to_hdmi { "TV" } else { "panel" },
        fb.0,
        fb.1,
        began.elapsed().as_millis()
    );
}

fn put(path: &Path, value: &str) {
    if let Err(e) = std::fs::write(path, value) {
        eprintln!("slot: hdmi: {}: {e}", path.display());
    }
}

fn wait_for(mut done: impl FnMut() -> bool, limit: Duration) -> bool {
    let began = Instant::now();
    while began.elapsed() < limit {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    done()
}

fn clear_framebuffer(fb0: &Path) {
    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(fb0) else {
        return;
    };
    let zeros = [0u8; 4096];
    while file.write(&zeros).is_ok_and(|n| n > 0) {}
}

#[cfg(unix)]
extern "C" {
    fn ioctl(fd: i32, request: std::ffi::c_ulong, ...) -> i32;
}

#[cfg(unix)]
const FBIOGET_VSCREENINFO: std::ffi::c_ulong = 0x4600;
#[cfg(unix)]
const FBIOPUT_VSCREENINFO: std::ffi::c_ulong = 0x4601;
/// FB_ACTIVATE_NOW (0) with FB_ACTIVATE_FORCE, so the driver applies the
/// geometry even when it looks unchanged.
#[cfg(unix)]
const FB_ACTIVATE_FORCE: u32 = 128;

/// Indices into struct fb_var_screeninfo, read as u32s.
const XRES: usize = 0;
const YRES: usize = 1;
const XRES_VIRTUAL: usize = 2;
const YRES_VIRTUAL: usize = 3;
const XOFFSET: usize = 4;
const YOFFSET: usize = 5;
const BITS_PER_PIXEL: usize = 6;
const ACTIVATE: usize = 21;

/// Size the framebuffer, double buffered at 32 bits, its page offset back at
/// zero. Coming off another size the old pan offset is not on a page of the
/// new one, and left there it stops the EGL driver ever flipping.
#[cfg(unix)]
fn set_framebuffer_size(fb0: &Path, (w, h): (u32, u32)) {
    use std::os::unix::io::AsRawFd;
    let Ok(file) = std::fs::OpenOptions::new().read(true).write(true).open(fb0) else {
        eprintln!("slot: hdmi: no {}", fb0.display());
        return;
    };
    let fd = file.as_raw_fd();
    let mut var = [0u32; 40];
    if unsafe { ioctl(fd, FBIOGET_VSCREENINFO, var.as_mut_ptr()) } != 0 {
        eprintln!("slot: hdmi: FBIOGET_VSCREENINFO failed");
        return;
    }
    // Drop the old allocation before asking for the new one.
    var[XRES] = 0;
    var[YRES] = 0;
    var[XRES_VIRTUAL] = 0;
    var[YRES_VIRTUAL] = 0;
    var[BITS_PER_PIXEL] = 32;
    unsafe { ioctl(fd, FBIOPUT_VSCREENINFO, var.as_mut_ptr()) };
    std::thread::sleep(Duration::from_millis(250));
    var[XRES] = w;
    var[YRES] = h;
    var[XRES_VIRTUAL] = w;
    var[YRES_VIRTUAL] = h * 2;
    var[XOFFSET] = 0;
    var[YOFFSET] = 0;
    var[BITS_PER_PIXEL] = 32;
    var[ACTIVATE] = FB_ACTIVATE_FORCE;
    if unsafe { ioctl(fd, FBIOPUT_VSCREENINFO, var.as_mut_ptr()) } != 0 {
        eprintln!("slot: hdmi: FBIOPUT_VSCREENINFO {w}x{h} failed");
    }
}

#[cfg(not(unix))]
fn set_framebuffer_size(_: &Path, _: (u32, u32)) {}

/// The parts of the sunxi disp2 UAPI (include/video/sunxi_display2.h in the
/// 4.9 sun50iw9 BSP kernel) needed to rewrite the fb0 scanout layer. Their
/// layout must match the kernel's exactly; the tests pin every size and
/// offset to what the C compiler gives for that header.
pub mod disp {
    pub const LAYER_SET_CONFIG: std::ffi::c_ulong = 0x47;
    pub const LAYER_GET_CONFIG: std::ffi::c_ulong = 0x48;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Rect {
        pub x: i32,
        pub y: i32,
        pub width: u32,
        pub height: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct RectSize {
        pub width: u32,
        pub height: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Rect64 {
        pub x: i64,
        pub y: i64,
        pub width: i64,
        pub height: i64,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct FbInfo {
        pub addr: [u64; 3],
        pub size: [RectSize; 3],
        pub align: [u32; 3],
        pub format: i32,
        pub color_space: i32,
        pub trd_right_addr: [u32; 3],
        /// A C bool.
        pub pre_multiply: u8,
        /// Fixed point, 32 bits of fraction.
        pub crop: Rect64,
        pub flags: i32,
        pub scan: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct LayerInfo {
        pub mode: i32,
        pub zorder: u8,
        pub alpha_mode: u8,
        pub alpha_value: u8,
        pub screen_win: Rect,
        /// A C bool.
        pub b_trd_out: u8,
        pub out_trd_mode: i32,
        /// In C a union of this and a colour; the colour is never used here.
        pub fb: FbInfo,
        pub id: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct LayerConfig {
        pub info: LayerInfo,
        /// A C bool.
        pub enable: u8,
        pub channel: u32,
        pub layer_id: u32,
    }
}

/// Point the fb0 scanout layer (channel 1, layer 0) at a framebuffer of
/// `fb` and show it over `out` of the output. The driver can restore an older
/// layer a moment after a switch, so write, read back and try again until it
/// sticks.
#[cfg(unix)]
fn commit_layer(disp_dev: &Path, fb: (u32, u32), out: (u32, u32)) {
    use std::os::unix::io::AsRawFd;
    let Ok(file) = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(disp_dev)
    else {
        eprintln!("slot: hdmi: no {}", disp_dev.display());
        return;
    };
    let fd = file.as_raw_fd();
    let get = |config: &mut disp::LayerConfig| -> bool {
        *config = disp::LayerConfig {
            channel: 1,
            layer_id: 0,
            ..Default::default()
        };
        let args: [std::ffi::c_ulong; 4] = [0, config as *mut _ as std::ffi::c_ulong, 1, 0];
        unsafe { ioctl(fd, disp::LAYER_GET_CONFIG, args.as_ptr()) >= 0 }
    };
    for attempt in 0..20 {
        let mut config = disp::LayerConfig::default();
        if !get(&mut config) {
            eprintln!("slot: hdmi: DISP_LAYER_GET_CONFIG failed");
            return;
        }
        config.enable = 1;
        for size in &mut config.info.fb.size {
            size.width = fb.0;
            size.height = fb.1 * 2;
        }
        config.info.fb.crop = disp::Rect64 {
            x: 0,
            y: 0,
            width: i64::from(fb.0) << 32,
            height: i64::from(fb.1) << 32,
        };
        config.info.screen_win = disp::Rect {
            x: 0,
            y: 0,
            width: out.0,
            height: out.1,
        };
        let args: [std::ffi::c_ulong; 4] = [0, &mut config as *mut _ as std::ffi::c_ulong, 1, 0];
        if unsafe { ioctl(fd, disp::LAYER_SET_CONFIG, args.as_ptr()) } < 0 {
            eprintln!("slot: hdmi: DISP_LAYER_SET_CONFIG failed");
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
        let mut back = disp::LayerConfig::default();
        if !get(&mut back) {
            return;
        }
        if back.info.fb.size[0].width == fb.0
            && back.info.screen_win.width == out.0
            && back.info.fb.crop.y == 0
        {
            if attempt > 0 {
                eprintln!("slot: hdmi: layer set after {} tries", attempt + 1);
            }
            return;
        }
    }
    eprintln!(
        "slot: hdmi: layer {}x{} on {}x{} did not stick",
        fb.0, fb.1, out.0, out.1
    );
}

#[cfg(not(unix))]
fn commit_layer(_: &Path, _: (u32, u32), _: (u32, u32)) {}

/// Watches the cable while Slot runs, and says when it has changed for long
/// enough to act on: the same new state on two checks in a row.
pub struct CableWatch {
    paths: Paths,
    on: bool,
    seen: Option<bool>,
    next: Instant,
}

const WATCH_EVERY: Duration = Duration::from_millis(500);

impl CableWatch {
    pub fn new(paths: Paths, on: bool) -> Self {
        CableWatch {
            paths,
            on,
            seen: None,
            next: Instant::now() + WATCH_EVERY,
        }
    }

    /// Whether the cable now says the picture belongs somewhere else.
    pub fn changed(&mut self, now: Instant) -> bool {
        if now < self.next || !self.paths.switchable() {
            return false;
        }
        self.next = now + WATCH_EVERY;
        let cable = self.paths.cable_in();
        if cable == self.on {
            self.seen = None;
            return false;
        }
        let settled = self.seen == Some(cable);
        self.seen = Some(cable);
        settled
    }
}

#[cfg(test)]
mod tests {
    use super::disp::*;
    use super::*;
    use std::mem::{offset_of, size_of};

    // Every size and offset below is what gcc and clang give for the C
    // header, on x86_64 and on aarch64 alike.
    #[test]
    fn the_layer_config_matches_the_kernel_layout() {
        assert_eq!(size_of::<LayerConfig>(), 184);
        assert_eq!(std::mem::align_of::<LayerConfig>(), 8);
        assert_eq!(size_of::<LayerInfo>(), 168);
        assert_eq!(size_of::<FbInfo>(), 128);
        assert_eq!(offset_of!(LayerConfig, info), 0);
        assert_eq!(offset_of!(LayerConfig, enable), 168);
        assert_eq!(offset_of!(LayerConfig, channel), 172);
        assert_eq!(offset_of!(LayerConfig, layer_id), 176);
        assert_eq!(offset_of!(LayerInfo, mode), 0);
        assert_eq!(offset_of!(LayerInfo, zorder), 4);
        assert_eq!(offset_of!(LayerInfo, alpha_mode), 5);
        assert_eq!(offset_of!(LayerInfo, alpha_value), 6);
        assert_eq!(offset_of!(LayerInfo, screen_win), 8);
        assert_eq!(offset_of!(LayerInfo, b_trd_out), 24);
        assert_eq!(offset_of!(LayerInfo, out_trd_mode), 28);
        assert_eq!(offset_of!(LayerInfo, fb), 32);
        assert_eq!(offset_of!(LayerInfo, id), 160);
        assert_eq!(offset_of!(FbInfo, addr), 0);
        assert_eq!(offset_of!(FbInfo, size), 24);
        assert_eq!(offset_of!(FbInfo, align), 48);
        assert_eq!(offset_of!(FbInfo, format), 60);
        assert_eq!(offset_of!(FbInfo, color_space), 64);
        assert_eq!(offset_of!(FbInfo, trd_right_addr), 68);
        assert_eq!(offset_of!(FbInfo, pre_multiply), 80);
        assert_eq!(offset_of!(FbInfo, crop), 88);
        assert_eq!(offset_of!(FbInfo, flags), 120);
        assert_eq!(offset_of!(FbInfo, scan), 124);
    }

    fn fake_kernel() -> (tempfile::TempDir, Paths) {
        let d = tempfile::tempdir().unwrap();
        let paths = Paths {
            sys: d.path().join("sys"),
            dev: d.path().join("dev"),
        };
        std::fs::create_dir_all(paths.sys.join("kernel/debug/dispdbg")).unwrap();
        std::fs::write(paths.sys.join("kernel/debug/dispdbg/command"), "").unwrap();
        std::fs::create_dir_all(&paths.dev).unwrap();
        std::fs::write(paths.dev.join("disp"), "").unwrap();
        std::fs::create_dir_all(paths.sys.join("class/extcon/hdmi")).unwrap();
        std::fs::create_dir_all(paths.sys.join("class/disp/disp/attr")).unwrap();
        (d, paths)
    }

    fn plug(paths: &Paths, state: &str) {
        std::fs::write(paths.sys.join("class/extcon/hdmi/state"), state).unwrap();
    }

    #[test]
    fn the_cable_reads_in_from_either_form_of_the_state_file() {
        let (_d, paths) = fake_kernel();
        assert!(!paths.cable_in(), "no state file at all");
        for (state, want) in [
            ("1\n", true),
            ("HDMI=1\n", true),
            ("0\n", false),
            ("HDMI=0\n", false),
        ] {
            plug(&paths, state);
            assert_eq!(paths.cable_in(), want, "{state:?}");
        }
        plug(&paths, "0");
        std::fs::create_dir_all(paths.sys.join("class/extcon/hdmi/cable.0")).unwrap();
        std::fs::write(paths.sys.join("class/extcon/hdmi/cable.0/state"), "1").unwrap();
        assert!(paths.cable_in(), "a cable state file counts too");
    }

    #[test]
    fn the_driver_says_when_the_picture_is_on_hdmi() {
        let (_d, paths) = fake_kernel();
        let sys = paths.sys.join("class/disp/disp/attr/sys");
        std::fs::write(
            &sys,
            "screen 0:\nde_rate 696000000 hz, ref_fps:60\nlcd output\n",
        )
        .unwrap();
        assert!(!paths.on_hdmi());
        std::fs::write(&sys, "screen 0:\nhdmi output mode(10)\n").unwrap();
        assert!(paths.on_hdmi());
    }

    #[test]
    fn nothing_is_switched_without_the_sunxi_display_driver() {
        let d = tempfile::tempdir().unwrap();
        let paths = Paths {
            sys: d.path().join("sys"),
            dev: d.path().join("dev"),
        };
        assert!(!paths.switchable());
        assert!(!sync(&paths, (720, 480)));
    }

    #[test]
    fn each_panel_has_its_own_size_to_come_back_to() {
        assert_eq!(panel_for(Some("rgsp")), (640, 480));
        assert_eq!(panel_for(Some("rg34xxsp")), (720, 480));
        assert_eq!(panel_for(Some("rgcubexx")), (720, 720));
        assert_eq!(panel_for(Some("rg28xx")), (480, 640));
        assert_eq!(panel_for(Some("rg35xxsp")), (640, 480));
        assert_eq!(panel_for(None), (640, 480));
    }

    #[test]
    fn a_cable_change_counts_once_it_has_held_for_two_checks() {
        let (_d, paths) = fake_kernel();
        plug(&paths, "0");
        let mut watch = CableWatch::new(paths.clone(), false);
        let t0 = Instant::now();
        assert!(!watch.changed(t0 + WATCH_EVERY), "nothing changed");
        plug(&paths, "1");
        assert!(!watch.changed(t0 + WATCH_EVERY * 2), "seen once");
        assert!(
            !watch.changed(t0 + WATCH_EVERY * 2),
            "too soon to look again"
        );
        assert!(watch.changed(t0 + WATCH_EVERY * 3), "seen twice");
        // A blip that goes away again is not a change.
        let mut watch = CableWatch::new(paths.clone(), true);
        plug(&paths, "0");
        assert!(!watch.changed(t0 + WATCH_EVERY));
        plug(&paths, "1");
        assert!(!watch.changed(t0 + WATCH_EVERY * 2));
        assert!(!watch.changed(t0 + WATCH_EVERY * 3));
    }
}
