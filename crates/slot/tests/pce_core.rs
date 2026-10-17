mod common;

use slot_retro::{ButtonMask, LibretroCore, RetroCore};
use slot_store::{Core, Platform};

fn beetle_pce() -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor")
        .join(slot::core::dylib_name(Core::Pce));
    if path.exists() {
        Some(path)
    } else {
        eprintln!("no beetle-pce-fast dylib on this host, skipping");
        None
    }
}

/// An 8 KB HuCard that does nothing but spin: `bra -2` at $E000, which the
/// reset vector in the card's last two bytes points at. Enough for the core
/// to boot it and draw.
fn spinning_rom(dir: &std::path::Path) -> std::path::PathBuf {
    let mut rom = vec![0u8; 0x2000];
    rom[0] = 0x80;
    rom[1] = 0xfe;
    rom[0x1ffe] = 0x00;
    rom[0x1fff] = 0xe0;
    let path = dir.join("Spin.pce");
    std::fs::write(&path, rom).unwrap();
    path
}

fn opened(dir: &std::path::Path) -> Option<LibretroCore> {
    let path = beetle_pce()?;
    let mut core = LibretroCore::open_with(&path, dir, dir).expect("open beetle-pce-fast");
    slot::core::apply_core_options(&mut core, Core::Pce, Platform::Tg16, "", false, false, None);
    let (w, h) = Platform::Tg16.source_size();
    core.set_video_size(w, h);
    Some(core)
}

#[test]
fn a_hucard_boots_with_no_bios_and_draws_240_lines_that_fit_the_buffer() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = opened(d.path()) else {
        return;
    };
    core.load(&spinning_rom(d.path()))
        .expect("beetle-pce-fast refused the card");
    for _ in 0..30 {
        core.run_frame(ButtonMask::default());
    }
    let (w, h) = core.frame_size();
    assert_eq!(h, 240, "{w}x{h}");
    assert!((256..=512).contains(&w), "{w}x{h}");
    assert!(w <= Platform::Tg16.source_size().0);
    let fps = core.av_info().fps;
    assert!((59.0..61.0).contains(&fps), "{fps} frames a second");
    assert!(!core.take_audio().is_empty(), "no sound came out");
}

#[test]
fn a_hucard_game_saves_and_restores_a_state() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = opened(d.path()) else {
        return;
    };
    core.load(&spinning_rom(d.path()))
        .expect("beetle-pce-fast refused the card");
    core.run_frame(ButtonMask::default());
    let state = core.serialize().expect("no save state");
    assert!(!state.is_empty());
    for _ in 0..10 {
        core.run_frame(ButtonMask::default());
    }
    core.unserialize(&state).expect("state would not load back");
}
