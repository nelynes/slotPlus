use slot_store::Platform;

#[test]
fn every_platform_has_a_directory() {
    let names: Vec<&str> = Platform::ALL.iter().map(|p| p.dir_name()).collect();
    assert_eq!(names, vec!["GBA", "GB", "GBC", "WS"]);
}

#[test]
fn every_platform_is_a_shelf_of_its_own() {
    assert_eq!(
        Platform::ALL,
        [Platform::Gba, Platform::Gb, Platform::Gbc, Platform::Ws]
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
}

#[test]
fn only_wonderswan_gets_a_bigger_video_buffer() {
    for p in [Platform::Gba, Platform::Gb, Platform::Gbc] {
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
