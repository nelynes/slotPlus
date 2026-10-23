mod common;

use std::path::Path;

use common::clocked;
use slot::app::Phase;
use slot::hold_picker::Hold;
use slot::session::Session;
use slot_input::{Btn, RawEvent};
use slot_ui::Millis;

const FRAME: Millis = 16;

fn step(s: &mut Session, now: &mut Millis, events: &[RawEvent]) {
    *now += FRAME;
    s.feed(events.iter().copied(), *now);
    s.update(FRAME as f32 / 1000.0);
}

fn tap(s: &mut Session, now: &mut Millis, btn: Btn) {
    step(s, now, &[RawEvent::Down(btn)]);
    step(s, now, &[RawEvent::Up(btn)]);
    for _ in 0..25 {
        step(s, now, &[]);
    }
}

fn lynx_card() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    for sub in slot::root::DIRS {
        std::fs::create_dir_all(d.path().join(sub)).unwrap();
    }
    let dir = d.path().join("Games/Lynx");
    std::fs::create_dir_all(&dir).unwrap();
    // Two games, as a card with one boots straight into it.
    for stem in ["California Games (USA)", "Chip's Challenge (USA, Europe)"] {
        std::fs::write(dir.join(format!("{stem}.lnx")), vec![0u8; 0x200]).unwrap();
    }
    clocked(d.path());
    let mut state = slot_store::read_slot_state(d.path());
    state.cart = None;
    state.cart_platform = None;
    state.shelf = Some("lynx".into());
    slot_store::write_slot_state(d.path(), &state).unwrap();
    d
}

fn picking(root: &Path) -> (Session, Millis) {
    let mut s = Session::boot(root.to_path_buf());
    let mut now = 0;
    for _ in 0..30 {
        if matches!(s.app().phase(), Phase::Shelf) {
            break;
        }
        step(&mut s, &mut now, &[]);
    }
    tap(&mut s, &mut now, Btn::Start);
    // Off the device nothing builds the cart's faces, so the picker starts
    // once it has waited for them.
    for _ in 0..120 {
        step(&mut s, &mut now, &[]);
    }
    assert_eq!(
        s.app().hold_picker(),
        Some(Hold::Upright),
        "START opens the hold picker"
    );
    (s, now)
}

#[test]
fn each_direction_moves_the_chip_to_its_socket_and_a_saves_it() {
    let d = lynx_card();
    let (mut s, mut now) = picking(d.path());
    for (btn, hold) in [
        (Btn::Left, Hold::Left),
        (Btn::Down, Hold::Flipped),
        (Btn::Right, Hold::Right),
        (Btn::Up, Hold::Upright),
        (Btn::Down, Hold::Flipped),
    ] {
        tap(&mut s, &mut now, btn);
        assert_eq!(s.app().hold_picker(), Some(hold), "{btn:?}");
    }
    tap(&mut s, &mut now, Btn::A);
    let saved = std::fs::read_to_string(d.path().join("Config/lynx_hold.txt")).unwrap();
    assert!(
        saved.contains("California Games (USA) = flipped"),
        "{saved}"
    );
}
