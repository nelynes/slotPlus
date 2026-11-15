//! slotPlus: a save being replaced is kept, a few to a game.

use slot::persist::{back_up, backups, write_sav, BACKUPS_KEPT, BACKUP_EVERY_SECS};
use slot_store::Platform;

#[test]
fn the_first_save_has_nothing_to_back_up() {
    let d = tempfile::tempdir().unwrap();
    write_sav(d.path(), Platform::Gba, "Emerald", &[1; 8]).unwrap();
    assert!(backups(d.path(), Platform::Gba, "Emerald").is_empty());
}

#[test]
fn replacing_a_save_keeps_the_old_one() {
    let d = tempfile::tempdir().unwrap();
    write_sav(d.path(), Platform::Gba, "Emerald", &[1; 8]).unwrap();
    write_sav(d.path(), Platform::Gba, "Emerald", &[2; 8]).unwrap();
    let kept = backups(d.path(), Platform::Gba, "Emerald");
    assert_eq!(kept.len(), 1);
    assert_eq!(std::fs::read(&kept[0].1).unwrap(), vec![1; 8]);
    assert!(kept[0].1.starts_with(d.path().join("Saves/Backups/GBA")));
}

#[test]
fn a_save_written_again_soon_after_isnt_backed_up_again() {
    let d = tempfile::tempdir().unwrap();
    let r = d.path();
    assert!(back_up(r, Platform::Gb, "Tetris", &[1], 1_000_000).unwrap());
    assert!(!back_up(r, Platform::Gb, "Tetris", &[2], 1_000_000 + 60).unwrap());
    assert!(back_up(
        r,
        Platform::Gb,
        "Tetris",
        &[3],
        1_000_000 + BACKUP_EVERY_SECS
    )
    .unwrap());
    assert_eq!(backups(r, Platform::Gb, "Tetris").len(), 2);
}

#[test]
fn only_the_newest_few_are_kept_and_other_games_are_left_alone() {
    let d = tempfile::tempdir().unwrap();
    let r = d.path();
    back_up(r, Platform::Gb, "Tetris DX", &[9], 5).unwrap();
    for i in 0..6u64 {
        back_up(
            r,
            Platform::Gb,
            "Tetris",
            &[i as u8],
            10_000 + i * BACKUP_EVERY_SECS,
        )
        .unwrap();
    }
    let kept = backups(r, Platform::Gb, "Tetris");
    assert_eq!(kept.len(), BACKUPS_KEPT);
    assert_eq!(std::fs::read(&kept[0].1).unwrap(), vec![5]);
    assert_eq!(backups(r, Platform::Gb, "Tetris DX").len(), 1);
}

#[test]
fn the_same_save_isnt_kept_twice() {
    let d = tempfile::tempdir().unwrap();
    let r = d.path();
    back_up(r, Platform::Gba, "Emerald", &[7; 4], 100).unwrap();
    assert!(!back_up(
        r,
        Platform::Gba,
        "Emerald",
        &[7; 4],
        100 + 2 * BACKUP_EVERY_SECS
    )
    .unwrap());
}
