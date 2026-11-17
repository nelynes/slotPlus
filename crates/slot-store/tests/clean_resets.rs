//! slotPlus Shelf Life: Clean Button Resets, what holding B to wipe a cart takes off it.

use slot_store::{read_slot_state, write_slot_state, CleanResets, Platform, PlayLog, SlotState};

#[test]
fn the_clean_button_starts_by_taking_off_both() {
    assert_eq!(SlotState::default().clean_resets, CleanResets::Both);
}

#[test]
fn each_setting_survives_a_restart_and_a_card_without_it_reads_both() {
    for resets in CleanResets::ALL {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("Config")).unwrap();
        let s = SlotState {
            clean_resets: resets,
            ..SlotState::default()
        };
        write_slot_state(d.path(), &s).unwrap();
        assert_eq!(read_slot_state(d.path()).clean_resets, resets);
    }
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    std::fs::write(d.path().join("Config/slot.state"), "dust=1\n").unwrap();
    assert_eq!(read_slot_state(d.path()).clean_resets, CleanResets::Both);
}

#[test]
fn the_choices_go_wear_dust_both_disabled_and_round() {
    let mut c = CleanResets::Wear;
    let mut seen = vec![c];
    for _ in 0..3 {
        c = c.step(true);
        seen.push(c);
    }
    assert_eq!(
        seen,
        [
            CleanResets::Wear,
            CleanResets::Dust,
            CleanResets::Both,
            CleanResets::Disabled
        ]
    );
    assert_eq!(c.step(true), CleanResets::Wear);
    assert_eq!(CleanResets::Wear.step(false), CleanResets::Disabled);
}

#[test]
fn a_wipe_of_one_leaves_the_other_where_it_was() {
    let mut log = PlayLog::default();
    log.add_play(Platform::Gba, "Emerald", 40 * 3600, 1_000);
    log.clean_only(Platform::Gba, "Emerald", 5_000, false, true);
    let r = log.record(Platform::Gba, "Emerald");
    assert_eq!(
        (r.cleaned, r.worn_seconds(), r.idle_since()),
        (None, 0, Some(1_000))
    );
    log.add_play(Platform::Gba, "Emerald", 3600, 6_000);
    log.clean_only(Platform::Gba, "Emerald", 9_000, true, false);
    let r = log.record(Platform::Gba, "Emerald");
    assert_eq!((r.idle_since(), r.worn_seconds()), (Some(9_000), 3600));
}
