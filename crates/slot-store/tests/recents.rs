mod common;

use common::tmp_root;
use slot_store::{read_slot_state, write_slot_state, Platform, Recent, SlotState, RECENTS_MAX};

fn recent(platform: Platform, stem: &str) -> Recent {
    Recent {
        platform,
        stem: stem.into(),
    }
}

fn clocked() -> SlotState {
    SlotState {
        clock_set: true,
        ..SlotState::default()
    }
}

#[test]
fn recents_are_off_and_empty_until_something_is_played() {
    let s = SlotState::default();
    assert!(!s.show_recents);
    assert!(s.recents.is_empty());
}

#[test]
fn the_game_just_played_goes_to_the_front() {
    let mut s = clocked();
    s.played(Platform::Gba, "Emerald");
    s.played(Platform::Gb, "Tetris");
    assert_eq!(
        s.recents,
        [
            recent(Platform::Gb, "Tetris"),
            recent(Platform::Gba, "Emerald")
        ]
    );
}

#[test]
fn playing_a_game_again_moves_it_rather_than_listing_it_twice() {
    let mut s = clocked();
    s.played(Platform::Gba, "Emerald");
    s.played(Platform::Gba, "Fusion");
    s.played(Platform::Gba, "Emerald");
    assert_eq!(
        s.recents,
        [
            recent(Platform::Gba, "Emerald"),
            recent(Platform::Gba, "Fusion")
        ]
    );
}

#[test]
fn the_same_name_on_two_systems_is_two_games() {
    let mut s = clocked();
    s.played(Platform::Gb, "Tetris");
    s.played(Platform::Gbc, "Tetris");
    assert_eq!(s.recents.len(), 2);
}

#[test]
fn only_the_last_ten_are_kept() {
    let mut s = clocked();
    for n in 0..15 {
        s.played(Platform::Gba, &format!("Game {n}"));
    }
    assert_eq!(s.recents.len(), RECENTS_MAX);
    assert_eq!(s.recents[0].stem, "Game 14");
    assert_eq!(s.recents[RECENTS_MAX - 1].stem, "Game 5");
}

#[test]
fn the_setting_and_the_list_survive_a_restart() {
    let d = tmp_root();
    let mut s = clocked();
    s.show_recents = true;
    s.played(Platform::Gba, "Pokemon - Emerald Version (USA, Europe)");
    s.played(Platform::Gbc, "Tetris DX");
    write_slot_state(d.path(), &s).unwrap();
    assert_eq!(read_slot_state(d.path()), s);
}

#[test]
fn a_card_written_before_recents_reads_with_them_off() {
    let d = tmp_root();
    write_slot_state(d.path(), &clocked()).unwrap();
    let path = d.path().join("Config/slot.state");
    let text = std::fs::read_to_string(&path).unwrap();
    let old: String = text
        .lines()
        .filter(|l| !l.starts_with("show_recents="))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, old).unwrap();
    let s = read_slot_state(d.path());
    assert!(s.clock_set, "the old settings did not read back");
    assert!(!s.show_recents);
    assert!(s.recents.is_empty());
}

#[test]
fn a_garbled_recent_line_is_skipped_and_the_rest_kept() {
    let d = tmp_root();
    let mut s = clocked();
    s.played(Platform::Gba, "Emerald");
    write_slot_state(d.path(), &s).unwrap();
    let path = d.path().join("Config/recents.txt");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("n64/Mario\ngba/\nnothing\n");
    std::fs::write(&path, text).unwrap();
    assert_eq!(
        read_slot_state(d.path()).recents,
        [recent(Platform::Gba, "Emerald")]
    );
}
