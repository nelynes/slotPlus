use slot_store::{read_slot_state, write_slot_state, SlotState};

#[test]
fn titles_start_hidden() {
    assert!(!SlotState::default().show_titles);
}

#[test]
fn show_titles_survives_a_restart() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    let s = SlotState {
        show_titles: true,
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    assert!(read_slot_state(d.path()).show_titles);
}

#[test]
fn a_card_written_before_titles_reads_with_them_hidden() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    write_slot_state(d.path(), &SlotState::default()).unwrap();
    let path = d.path().join("Config/slot.state");
    let old: String = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("show_titles="))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, old).unwrap();
    assert!(!read_slot_state(d.path()).show_titles);
}
