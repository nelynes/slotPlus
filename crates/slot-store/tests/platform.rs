use slot_store::Platform;

#[test]
fn every_platform_has_a_directory() {
    let names: Vec<&str> = Platform::ALL.iter().map(|p| p.dir_name()).collect();
    assert_eq!(names, vec!["GBA", "GBC", "GB", "WS", "Lynx"]);
}

#[test]
fn every_platform_is_a_shelf_of_its_own() {
    assert_eq!(
        Platform::ALL,
        [
            Platform::Gba,
            Platform::Gbc,
            Platform::Gb,
            Platform::Ws,
            Platform::Lynx
        ]
    );
    for (i, p) in Platform::ALL.iter().enumerate() {
        assert!(
            !Platform::ALL[..i].contains(p),
            "{p:?} is named twice in the ring, so two shelves would hold one platform"
        );
    }
}

#[test]
fn each_platform_takes_only_its_own_extensions() {
    assert!(Platform::Gba.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(!Platform::Gba.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gbc")));
    assert!(!Platform::Gb.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(Platform::Gba.accepts(std::path::Path::new("Shrek.GBA")));
    assert!(Platform::Lynx.accepts(std::path::Path::new("California Games.lnx")));
    assert!(Platform::Lynx.accepts(std::path::Path::new("Klax.LYX")));
    assert!(!Platform::Lynx.accepts(std::path::Path::new("Tetris.gb")));
    assert!(!Platform::Ws.accepts(std::path::Path::new("Klax.lnx")));
}

#[test]
fn only_wonderswan_gets_a_bigger_video_buffer() {
    for p in [Platform::Gba, Platform::Gb, Platform::Gbc, Platform::Lynx] {
        assert_eq!(p.source_size(), (240, 160), "{p:?} must keep its buffer");
    }
    let (w, h) = Platform::Ws.source_size();
    let (pw, ph) = Platform::Ws.picture();
    assert!(
        w >= ph.max(pw) && h >= ph.max(pw),
        "a turned frame must fit"
    );
    assert_eq!((w * 2, h * 2), (720, 480), "the buffer is the canvas at 2x");
}

fn ws_rom(dir: &std::path::Path, name: &str, len: usize, flags: u8) -> std::path::PathBuf {
    let mut rom = vec![0u8; len];
    rom[len - 4] = flags;
    let path = dir.join(name);
    std::fs::write(&path, rom).unwrap();
    path
}

#[test]
fn the_rom_footer_says_whether_a_game_is_vertical() {
    let d = tempfile::tempdir().unwrap();
    let vertical = ws_rom(d.path(), "Tetris.wsc", 128 * 1024, 0x05);
    let horizontal = ws_rom(d.path(), "Klonoa.wsc", 128 * 1024, 0x04);
    assert!(slot_store::ws_vertical(&vertical));
    assert!(!slot_store::ws_vertical(&horizontal));
}

#[test]
fn a_rom_that_is_too_small_or_missing_is_not_vertical() {
    let d = tempfile::tempdir().unwrap();
    let tiny = ws_rom(d.path(), "Tiny.ws", 1024, 0x01);
    assert!(!slot_store::ws_vertical(&tiny));
    assert!(!slot_store::ws_vertical(&d.path().join("Missing.ws")));
}
