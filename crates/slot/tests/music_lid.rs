mod common;

use std::time::{Duration, Instant};

use slot::app::Phase;
use slot::session::Session;
use slot_input::{Action, Btn, RawEvent};
use std::sync::atomic::Ordering;

fn song(path: &std::path::Path) {
    // A second and a half of quiet WAV.
    let (rate, frames) = (22_050u32, 33_075u32);
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

fn until(s: &mut Session, what: &str, done: impl Fn(&Session) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !done(s) {
        assert!(Instant::now() < deadline, "{what} never happened");
        s.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn shutting_the_lid_on_a_playing_song_only_turns_the_screen_off() {
    let d = common::tmp_root_with_carts(&[]);
    song(&d.path().join("Music/01 Song.wav"));
    common::clocked(d.path());
    let mut s = Session::boot(d.path().to_path_buf());
    let (power, backlight) = common::panel(d.path(), Duration::from_secs(300));
    s.app_mut().set_power(power);
    s.feed([RawEvent::Down(Btn::A)], 16);
    s.feed([RawEvent::Up(Btn::A)], 32);
    until(&mut s, "seating", |s| {
        matches!(s.app().phase(), Phase::Playing { .. })
    });
    // A plays the first song.
    s.feed([RawEvent::Down(Btn::A)], 300);
    let root = d.path().to_path_buf();
    until(&mut s, "playing", |_| slot::music::playing(&root));
    s.feed([RawEvent::Up(Btn::A)], 400);
    let lit = backlight.load(Ordering::Relaxed);

    s.app_mut().apply(Action::LidClose);
    assert!(s.app().music_dark());
    assert!(
        matches!(s.app().phase(), Phase::Playing { .. }),
        "still playing"
    );
    assert_eq!(backlight.load(Ordering::Relaxed), 0, "the screen is off");
    s.app_mut().apply(Action::LidOpen);
    assert!(!s.app().music_dark());
    assert_eq!(backlight.load(Ordering::Relaxed), lit, "and back on");

    s.app_mut().apply(Action::LidClose);
    assert!(s.app().music_dark());
    until(&mut s, "sleep once the music ends", |s| {
        matches!(s.app().phase(), Phase::Doze { .. })
    });
    assert!(!s.app().music_dark());
    s.app_mut().apply(Action::LidOpen);
    assert!(matches!(s.app().phase(), Phase::Playing { .. }));
}

#[test]
fn shutting_the_lid_with_no_song_playing_sleeps_as_ever() {
    let d = common::tmp_root_with_carts(&[]);
    song(&d.path().join("Music/01 Song.wav"));
    common::clocked(d.path());
    let mut s = Session::boot(d.path().to_path_buf());
    s.feed([RawEvent::Down(Btn::A)], 16);
    s.feed([RawEvent::Up(Btn::A)], 32);
    until(&mut s, "seating", |s| {
        matches!(s.app().phase(), Phase::Playing { .. })
    });
    s.app_mut().apply(Action::LidClose);
    assert!(matches!(s.app().phase(), Phase::Doze { .. }));
}
