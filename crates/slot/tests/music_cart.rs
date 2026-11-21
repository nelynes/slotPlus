mod common;

use std::time::{Duration, Instant};

use slot::app::Phase;
use slot::session::Session;
use slot_input::{Btn, RawEvent};
use slot_store::music::MUSIC_STEM;
use slot_store::{Core, Platform};

fn song(path: &std::path::Path) {
    // A short, quiet WAV.
    let (rate, frames) = (22_050u32, 2_205u32);
    let mut b = Vec::new();
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + frames * 2).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    for v in [
        16u32.to_le_bytes().to_vec(),
        1u16.to_le_bytes().to_vec(),
        1u16.to_le_bytes().to_vec(),
    ] {
        b.extend_from_slice(&v);
    }
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(frames * 2).to_le_bytes());
    b.extend(std::iter::repeat_n(0u8, (frames * 2) as usize));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b).unwrap();
}

#[test]
fn the_cart_sits_on_the_gba_shelf_only_when_there_is_music() {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    let carts = slot_store::scan_shelf(d.path(), Platform::Gba).unwrap();
    assert!(!carts.iter().any(|c| c.stem == MUSIC_STEM));
    song(&d.path().join("Music/Album/01 Song.wav"));
    let carts = slot_store::scan_shelf(d.path(), Platform::Gba).unwrap();
    let music = carts
        .iter()
        .find(|c| c.stem == MUSIC_STEM)
        .expect("the cart");
    assert!(slot_store::music::is_music(music));
    assert_eq!(carts.len(), 2, "beside the games");
    assert!(!slot_store::scan_shelf(d.path(), Platform::Gb)
        .unwrap()
        .iter()
        .any(|c| c.stem == MUSIC_STEM));
}

#[test]
fn it_plays_on_its_own_player_and_no_core_is_needed() {
    let d = common::tmp_root_with_carts(&[]);
    song(&d.path().join("Music/01 Song.wav"));
    common::clocked(d.path());
    let mut s = Session::boot(d.path().to_path_buf());
    assert_eq!(s.app().shelf_stems(), vec![MUSIC_STEM]);
    s.feed([RawEvent::Down(Btn::A)], 16);
    s.feed([RawEvent::Up(Btn::A)], 32);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "the cart never seated");
        s.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(s.app().core(), Core::Music);
    assert!(s.emu().is_some(), "the player is running");
}
