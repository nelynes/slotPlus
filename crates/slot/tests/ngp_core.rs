mod common;

use slot_retro::{ButtonMask, LibretroCore, RetroCore};
use slot_store::{Core, Platform};

fn neopop() -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor")
        .join(slot::core::dylib_name(Core::Ngp));
    if path.exists() {
        Some(path)
    } else {
        eprintln!("no ngp dylib on this host, skipping");
        None
    }
}

/// A 64 KB NeoGeo Pocket Color rom: the cart header, then `jr t,-2` where the
/// header says to start, so it spins forever. Enough for the core to boot it
/// and draw.
fn spinning_rom(dir: &std::path::Path) -> std::path::PathBuf {
    let mut rom = vec![0u8; 0x10000];
    rom[..28].copy_from_slice(b"COPYRIGHT BY SNK CORPORATION");
    rom[0x1c..0x20].copy_from_slice(&0x0020_0040u32.to_le_bytes());
    rom[0x23] = 0x10;
    rom[0x24..0x30].copy_from_slice(b"SPIN        ");
    rom[0x40] = 0x68;
    rom[0x41] = 0xfe;
    let path = dir.join("Spin.ngc");
    std::fs::write(&path, rom).unwrap();
    path
}

fn opened(dir: &std::path::Path) -> Option<LibretroCore> {
    let path = neopop()?;
    let saves = slot::core::save_dir_for(dir, Core::Ngp);
    std::fs::create_dir_all(&saves).unwrap();
    let mut core = LibretroCore::open_with(&path, dir, &saves).expect("open neopop");
    slot::core::apply_core_options(&mut core, Core::Ngp, Platform::Ngp, "", false, true, None);
    Some(core)
}

#[test]
fn a_neogeo_pocket_game_boots_with_no_bios_and_draws_its_160x152_screen() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = opened(d.path()) else {
        return;
    };
    let (w, h) = Platform::Ngp.source_size();
    core.set_video_size(w, h);
    core.load(&spinning_rom(d.path()))
        .expect("neopop refused the rom");
    for _ in 0..30 {
        core.run_frame(ButtonMask::default());
    }
    assert_eq!(core.frame_size(), Platform::Ngp.picture());
    let fps = core.av_info().fps;
    assert!((60.0..60.5).contains(&fps), "{fps} frames a second");
    assert!(!core.take_audio().is_empty(), "no sound came out");
    assert_eq!(core.option("ngp_language").as_deref(), Some("english"));
}

#[test]
fn a_neogeo_pocket_game_saves_and_restores_a_state() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = opened(d.path()) else {
        return;
    };
    core.load(&spinning_rom(d.path()))
        .expect("neopop refused the rom");
    core.run_frame(ButtonMask::default());
    let state = core.serialize().expect("no save state");
    assert!(!state.is_empty());
    for _ in 0..10 {
        core.run_frame(ButtonMask::default());
    }
    core.unserialize(&state).expect("state would not load back");
}

#[test]
fn neopop_is_told_to_save_in_saves_ngp() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(core) = opened(d.path()) else {
        return;
    };
    let reported = core.reported_save_dir();
    assert!(
        reported.ends_with("Saves/NGP"),
        "the core was told to save in {reported}"
    );
}
