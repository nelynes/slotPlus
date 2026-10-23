use slot_store::Platform;

#[test]
fn every_platform_has_a_directory() {
    let names: Vec<&str> = Platform::ALL.iter().map(|p| p.dir_name()).collect();
    assert_eq!(
        names,
        vec!["GBA", "GBC", "GB", "WS", "Lynx", "GG", "NGP", "TG16"]
    );
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
            Platform::Lynx,
            Platform::Gg,
            Platform::Ngp,
            Platform::Tg16
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
    assert!(Platform::Gg.accepts(std::path::Path::new("Sonic Chaos.gg")));
    assert!(Platform::Gg.accepts(std::path::Path::new("Shinobi.GG")));
    assert!(!Platform::Gg.accepts(std::path::Path::new("Tetris.gb")));
    assert!(!Platform::Gg.accepts(std::path::Path::new("Alex Kidd.sms")));
    assert!(!Platform::Gb.accepts(std::path::Path::new("Sonic Chaos.gg")));
}

#[test]
fn a_game_gear_picture_is_the_size_of_a_game_boy_one() {
    assert_eq!(Platform::Gg.picture(), (160, 144));
    assert_eq!(Platform::Gg.picture(), Platform::Gb.picture());
    assert_eq!(Platform::Gg.name(), "Game Gear");
    assert!(
        !Platform::Gg.turns(),
        "no Game Gear game is held on its side"
    );
}

#[test]
fn only_wonderswan_and_turbografx_get_a_bigger_video_buffer() {
    for p in [
        Platform::Gba,
        Platform::Gb,
        Platform::Gbc,
        Platform::Lynx,
        Platform::Gg,
        Platform::Ngp,
    ] {
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

#[test]
fn neogeo_pocket_and_pocket_color_games_share_one_shelf() {
    for name in [
        "Metal Slug.ngp",
        "SNK vs Capcom.ngc",
        "Sonic.NGPC",
        "Pac-Man.NGP",
    ] {
        assert!(Platform::Ngp.accepts(std::path::Path::new(name)), "{name}");
    }
    assert!(!Platform::Ngp.accepts(std::path::Path::new("Sonic Chaos.gg")));
    assert!(!Platform::Gg.accepts(std::path::Path::new("Metal Slug.ngp")));
    assert_eq!(Platform::Ngp.name(), "NeoGeo Pocket");
    assert_eq!(Platform::Ngp.dir_name(), "NGP");
}

#[test]
fn turbografx_hucards_take_pce_files_only() {
    assert!(Platform::Tg16.accepts(std::path::Path::new("Bonk's Adventure.pce")));
    assert!(Platform::Tg16.accepts(std::path::Path::new("R-Type.PCE")));
    // CD games are not HuCards, and there is no SuperGrafx support.
    for name in ["Ys.cue", "Ys.chd", "Ghouls.sgx"] {
        assert!(
            !Platform::Tg16.accepts(std::path::Path::new(name)),
            "{name}"
        );
    }
    assert_eq!(Platform::Tg16.name(), "TurboGrafx-16");
    assert_eq!(Platform::Tg16.dir_name(), "TG16");
    assert!(!Platform::Tg16.turns());
}

#[test]
fn a_turbografx_buffer_holds_a_512_wide_picture_and_lands_on_the_canvas_at_2x_down() {
    let (w, h) = Platform::Tg16.source_size();
    assert!(w >= 512, "room for the high resolution mode");
    assert_eq!((w, h * 2), (720, 480), "1x across, 2x down on the canvas");
    assert_eq!(Platform::Tg16.picture(), (256, 240));
    assert_eq!(Platform::Tg16.actual_window(), (360, 240), "2x both ways");
}

#[test]
fn a_neogeo_pocket_picture_fits_the_shared_buffer_at_3x() {
    assert_eq!(Platform::Ngp.picture(), (160, 152));
    assert_eq!(Platform::Ngp.source_size(), (240, 160));
    let (w, h) = Platform::Ngp.picture();
    assert!(w * 3 <= 720 && h * 3 <= 480, "160x152 at 3x is 480x456");
    assert!(!Platform::Ngp.turns());
}
