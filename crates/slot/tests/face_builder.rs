use std::time::{Duration, Instant};

use slot::face_builder::{BuiltFaces, FaceBuilder};
use slot::label_cache::label_art;
use slot_store::{Cart, Platform};
use slot_ui::{CART_H, CART_W, TURN_PAD};

fn cart(stem: &str) -> Cart {
    Cart {
        platform: Platform::Gba,
        stem: stem.into(),
        rom: format!("Games/GBA/{stem}.gba").into(),
        label: None,
        title: stem.to_uppercase(),
        code: String::new(),
        shell: None,
        tags: Vec::new(),
    }
}

fn collect(builder: &FaceBuilder, want: usize) -> Vec<BuiltFaces> {
    let mut got = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while got.len() < want && Instant::now() < deadline {
        if let Some(faces) = builder.take() {
            got.push(faces);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    got
}

#[test]
fn a_request_comes_back_as_the_open_carts_faces() {
    let builder = FaceBuilder::spawn(std::env::temp_dir());
    builder.request(cart("Metroid Fusion"));
    let got = collect(&builder, 1);
    assert_eq!(got.len(), 1, "the worker never answered");
    assert_eq!(got[0].stem, "Metroid Fusion");
    assert_eq!((got[0].board.w, got[0].board.h), (CART_W, CART_H));
    assert_eq!(
        (got[0].lid.w, got[0].lid.h),
        (CART_W + 2 * TURN_PAD, CART_H + 2 * TURN_PAD)
    );
}

#[test]
fn the_newest_request_of_a_burst_is_the_last_built() {
    let builder = FaceBuilder::spawn(std::env::temp_dir());
    for stem in ["Advance Wars", "Drill Dozer", "Metroid Fusion"] {
        builder.request(cart(stem));
    }
    let mut seen = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Some(faces) = builder.take() {
            let done = faces.stem == "Metroid Fusion";
            seen.push(faces.stem);
            if done {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(seen.last().map(String::as_str), Some("Metroid Fusion"));
    assert!(
        seen.len() <= 2,
        "the burst did not collapse to its newest: {seen:?} came back"
    );
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        builder.take().is_none(),
        "a build older than the newest came back after it"
    );
}

fn red_label(path: &std::path::Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(file, 640, 330);
    enc.set_color(png::ColorType::Rgb);
    let px: Vec<u8> = (0..640 * 330).flat_map(|_| [200u8, 10, 10]).collect();
    enc.write_header().unwrap().write_image_data(&px).unwrap();
}

#[test]
fn the_selected_carts_lid_comes_from_the_label_cache() {
    let d = tempfile::tempdir().unwrap();
    let label = d.path().join("Labels/GBA/Emerald.png");
    red_label(&label);
    let cart = Cart {
        platform: Platform::Gba,
        stem: "Emerald".into(),
        rom: d.path().join("Games/GBA/Emerald.gba"),
        label: Some(label.clone()),
        title: String::new(),
        code: "BPRE".into(),
        shell: None,
        tags: Vec::new(),
    };
    label_art(d.path(), &cart).unwrap();
    let meta = std::fs::metadata(&label).unwrap();
    let junk = vec![0u8; meta.len() as usize];
    std::fs::write(&label, junk).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&label)
        .unwrap()
        .set_modified(meta.modified().unwrap())
        .unwrap();
    let builder = FaceBuilder::spawn(d.path().to_path_buf());
    builder.request(cart);
    let mut built = None;
    for _ in 0..200 {
        built = builder.take();
        if built.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let lid = built.expect("no lid was built").lid;
    let (x, y) = (
        slot_ui::TURN_PAD + slot_ui::LABEL_X + slot_ui::LABEL_W / 2,
        slot_ui::TURN_PAD + slot_ui::LABEL_Y + slot_ui::LABEL_H / 2,
    );
    let i = ((y * lid.w + x) * 4) as usize;
    assert_eq!(
        &lid.rgba[i..i + 3],
        &[200, 10, 10],
        "the lid was not drawn from the cached label"
    );
}

/// slotPlus: a game with a downloaded set shows how far it's got, beside the trophy and as
/// a gold sticker on its board; one without a set shows neither.
#[test]
fn a_game_with_a_set_shows_its_progress() {
    let root = tempfile::tempdir().unwrap();
    let r = root.path();
    std::fs::create_dir_all(r.join("Config")).unwrap();
    std::fs::write(
        r.join("Config/retroachievements.toml"),
        "enabled = true\nusername = 'Player'\n",
    )
    .unwrap();
    std::fs::create_dir_all(r.join("Games/GBA")).unwrap();
    let rom = r.join("Games/GBA/Metroid Fusion.gba");
    std::fs::write(&rom, b"a rom").unwrap();
    let meta = rom.metadata().unwrap();
    let modified_ns = meta
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // md5 of the lowercased username, as the achievements crate names an account.
    let account = "912af0dff974604f1321254ca8ff38b6";
    let dir = r.join("Saves/RetroAchievements").join(account);
    std::fs::create_dir_all(&dir).unwrap();
    let hash = "0123456789abcdef0123456789abcdef";
    let index = serde_json::json!({
        rom.to_string_lossy(): { "size": meta.len(), "modified_ns": modified_ns, "hash": hash }
    });
    std::fs::write(dir.join("library.json"), index.to_string()).unwrap();
    let achievement = |id: u32| {
        serde_json::json!({ "ID": id, "Title": "t", "Description": "d", "Points": 5,
                            "Flags": 3, "MemAddr": "0xH0=1", "BadgeName": "1" })
    };
    let game = serde_json::json!([
        { "ID": 1, "ConsoleID": 5, "Title": "Metroid Fusion",
          "Achievements": [achievement(1), achievement(2), achievement(3)] },
        [2]
    ]);
    std::fs::write(dir.join(format!("{hash}.json")), game.to_string()).unwrap();

    let mut with_set = cart("Metroid Fusion");
    with_set.rom = rom;
    let builder = FaceBuilder::spawn(r.to_path_buf());
    builder.request(cart("Drill Dozer"));
    let plain = collect(&builder, 1).remove(0);
    assert!(plain.progress.is_none());
    builder.request(with_set);
    let got = collect(&builder, 1).remove(0);
    let progress = got.progress.expect("a set, so progress");
    assert_eq!(progress.h, slot::face_builder::PROGRESS_H);
    assert!(progress.rgba.chunks(4).any(|p| p[3] > 200));
    assert_ne!(got.board.rgba, plain.board.rgba, "no gold sticker on the board");
}
