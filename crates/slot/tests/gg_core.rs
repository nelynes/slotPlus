mod common;

use slot_retro::{ButtonMask, LibretroCore, RetroCore};
use slot_store::{Core, Platform};

fn gearsystem() -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor")
        .join(slot::core::dylib_name(Core::Gearsystem));
    if path.exists() {
        Some(path)
    } else {
        eprintln!("no gearsystem dylib on this host, skipping");
        None
    }
}

/// A 32 KB Game Gear rom that does nothing but spin: `jr -2` at the reset
/// vector. Enough for the core to boot it as a Game Gear game and draw.
fn spinning_rom(dir: &std::path::Path) -> std::path::PathBuf {
    let mut rom = vec![0u8; 0x8000];
    rom[0] = 0x18;
    rom[1] = 0xfe;
    let path = dir.join("Spin.gg");
    std::fs::write(&path, rom).unwrap();
    path
}

fn opened(dir: &std::path::Path) -> Option<LibretroCore> {
    let path = gearsystem()?;
    let mut core = LibretroCore::open_with(&path, dir, dir).expect("open gearsystem");
    slot::core::apply_core_options(
        &mut core,
        Core::Gearsystem,
        Platform::Gg,
        "",
        false,
        true,
        None,
    );
    Some(core)
}

#[test]
fn a_game_gear_game_boots_with_no_bios_and_draws_the_bare_160x144_screen() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = opened(d.path()) else {
        return;
    };
    core.set_video_size(Platform::Gg.source_size().0, Platform::Gg.source_size().1);
    core.load(&spinning_rom(d.path()))
        .expect("gearsystem refused the rom");
    for _ in 0..30 {
        core.run_frame(ButtonMask::default());
    }
    assert_eq!(core.frame_size(), Platform::Gg.picture());
    assert!(
        !d.path().join("bios.gg").exists(),
        "the test must not depend on a bios"
    );
    let fps = core.av_info().fps;
    assert!((59.0..61.0).contains(&fps), "{fps} frames a second");
    assert!(!core.take_audio().is_empty(), "no sound came out");
}

#[test]
fn a_game_gear_game_saves_and_restores_a_state() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = opened(d.path()) else {
        return;
    };
    core.load(&spinning_rom(d.path()))
        .expect("gearsystem refused the rom");
    core.run_frame(ButtonMask::default());
    let state = core.serialize().expect("no save state");
    assert!(!state.is_empty());
    for _ in 0..10 {
        core.run_frame(ButtonMask::default());
    }
    core.unserialize(&state).expect("state would not load back");
}
