use slot_store::{core_for_platform, read_selected_cores, write_selected_core, Core, Platform};
use tempfile::tempdir;

fn core_for(root: &std::path::Path, stem: &str) -> Core {
    core_for_platform(root, stem, Platform::Gba, Core::Mgba)
}

fn root_with(ini: Option<&str>) -> tempfile::TempDir {
    let d = tempdir().unwrap();
    std::fs::create_dir(d.path().join("Config")).unwrap();
    if let Some(text) = ini {
        std::fs::write(d.path().join("Config/selected_core.txt"), text).unwrap();
    }
    d
}

#[test]
fn absent_file_means_everything_defaults() {
    let d = root_with(None);
    assert!(read_selected_cores(d.path()).is_empty());
    assert_eq!(core_for(d.path(), "Emerald"), Core::Mgba);
}

#[test]
fn a_listed_stem_gets_its_core() {
    let d = root_with(Some("Pokemon - Emerald Version (USA, Europe) = gpsp\n"));
    assert_eq!(
        core_for(d.path(), "Pokemon - Emerald Version (USA, Europe)"),
        Core::Gpsp
    );
}

#[test]
fn an_unlisted_stem_defaults_to_mgba() {
    let d = root_with(Some("Emerald = gpsp\n"));
    assert_eq!(core_for(d.path(), "Metroid Fusion"), Core::Mgba);
}

#[test]
fn malformed_lines_are_ignored_rather_than_fatal() {
    let d = root_with(Some(concat!(
        "\n",
        "# a comment\n",
        "; another comment\n",
        "[cores]\n",
        "no equals sign here\n",
        "Emerald = notacore\n",
        "  Spaced Out   =   gpsp  \n",
        "= gpsp\n",
        "Trailing =\n",
    )));
    let map = read_selected_cores(d.path());
    assert_eq!(
        map.get("Spaced Out"),
        Some(&Core::Gpsp),
        "whitespace not trimmed"
    );
    assert_eq!(
        map.get("Emerald"),
        None,
        "an unknown core name must not be stored"
    );
    assert_eq!(core_for(d.path(), "Emerald"), Core::Mgba);
    assert_eq!(map.len(), 1, "only the one good line should survive");
}

#[test]
fn a_later_duplicate_wins() {
    let d = root_with(Some("Emerald = mgba\nEmerald = gpsp\n"));
    assert_eq!(core_for(d.path(), "Emerald"), Core::Gpsp);
}

#[test]
fn the_last_line_for_a_cart_wins_even_when_it_is_the_typo() {
    let d = root_with(Some("Emerald = gpsp\nEmerald = notacore\n"));
    assert_eq!(
        core_for(d.path(), "Emerald"),
        Core::Mgba,
        "a line the later one superseded is still in force"
    );
    assert_eq!(
        read_selected_cores(d.path()).get("Emerald"),
        None,
        "the map and core_for read the same two lines differently"
    );

    let d = root_with(Some("Emerald = notacore\nEmerald = gpsp\n"));
    assert_eq!(core_for(d.path(), "Emerald"), Core::Gpsp);
    assert_eq!(
        read_selected_cores(d.path()).get("Emerald"),
        Some(&Core::Gpsp)
    );
}

#[test]
fn core_names_round_trip() {
    for c in Core::ALL {
        assert_eq!(Core::parse(c.as_str()), Some(c));
    }
    assert_eq!(
        Core::parse("MGBA"),
        Some(Core::Mgba),
        "case is not the user's problem"
    );
    assert_eq!(Core::parse("nonsense"), None);
    assert_eq!(Core::default(), Core::Mgba);
}

#[test]
fn writing_a_core_creates_the_file_when_absent() {
    let d = root_with(None);
    slot_store::write_selected_core(d.path(), "Emerald", Core::Gpsp).unwrap();
    assert_eq!(core_for(d.path(), "Emerald"), Core::Gpsp);
}

#[test]
fn writing_a_core_replaces_that_carts_line_and_leaves_the_rest_alone() {
    let d = root_with(Some(concat!(
        "# my notes\n",
        "\n",
        "Emerald = mgba\n",
        "Metroid Fusion = gpsp\n",
    )));
    slot_store::write_selected_core(d.path(), "Emerald", Core::Gpsp).unwrap();

    let text = std::fs::read_to_string(d.path().join("Config/selected_core.txt")).unwrap();
    assert!(
        text.contains("# my notes"),
        "a hand-written comment was destroyed"
    );
    assert!(
        text.contains("Metroid Fusion = gpsp"),
        "another cart's entry was lost"
    );
    assert_eq!(core_for(d.path(), "Emerald"), Core::Gpsp);
    assert_eq!(core_for(d.path(), "Metroid Fusion"), Core::Gpsp);
    assert_eq!(
        text.matches("Emerald").count(),
        1,
        "the old line was left behind"
    );
}

#[test]
fn writing_a_core_appends_a_cart_the_file_has_never_seen() {
    let d = root_with(Some("Emerald = gpsp\n"));
    slot_store::write_selected_core(d.path(), "Drill Dozer", Core::Gpsp).unwrap();
    assert_eq!(core_for(d.path(), "Emerald"), Core::Gpsp);
    assert_eq!(core_for(d.path(), "Drill Dozer"), Core::Gpsp);
}

#[test]
fn writing_the_default_still_records_it() {
    let d = root_with(Some("Emerald = gpsp\n"));
    slot_store::write_selected_core(d.path(), "Emerald", Core::Mgba).unwrap();
    assert_eq!(core_for(d.path(), "Emerald"), Core::Mgba);
    let text = std::fs::read_to_string(d.path().join("Config/selected_core.txt")).unwrap();
    assert!(text.contains("Emerald = mgba"));
}

#[test]
fn an_unlisted_gb_stem_takes_the_card_default() {
    let d = tempdir().unwrap();
    assert_eq!(
        core_for_platform(d.path(), "Tetris", Platform::Gb, Core::Gambatte),
        Core::Gambatte
    );
    assert_eq!(
        core_for_platform(d.path(), "Tetris Chromatic", Platform::Gbc, Core::Gambatte),
        Core::Gambatte
    );
    assert_eq!(
        core_for_platform(d.path(), "Tetris", Platform::Gb, Core::Mgba),
        Core::Mgba
    );
}

#[test]
fn an_unlisted_gba_stem_stays_mgba() {
    let d = tempdir().unwrap();
    assert_eq!(
        core_for_platform(d.path(), "Emerald", Platform::Gba, Core::Gambatte),
        Core::Mgba
    );
    assert_eq!(
        core_for_platform(d.path(), "Emerald", Platform::Gbav, Core::Gambatte),
        Core::Mgba
    );
}

#[test]
fn a_gb_stem_listed_gpsp_takes_the_card_default() {
    let d = tempdir().unwrap();
    write_selected_core(d.path(), "Tetris", Core::Gpsp).unwrap();
    assert_eq!(
        core_for_platform(d.path(), "Tetris", Platform::Gb, Core::Gambatte),
        Core::Gambatte
    );
}

#[test]
fn a_gb_stem_listed_mgba_keeps_mgba() {
    let d = tempdir().unwrap();
    write_selected_core(d.path(), "Tetris", Core::Mgba).unwrap();
    assert_eq!(
        core_for_platform(d.path(), "Tetris", Platform::Gb, Core::Gambatte),
        Core::Mgba
    );
}

#[test]
fn gambatte_round_trips() {
    let d = root_with(None);
    write_selected_core(d.path(), "Tetris", Core::Gambatte).unwrap();
    assert_eq!(
        read_selected_cores(d.path()).get("Tetris"),
        Some(&Core::Gambatte)
    );
    assert_eq!(Core::parse("gambatte"), Some(Core::Gambatte));
    assert_eq!(Core::Gambatte.as_str(), "gambatte");
    assert_eq!(Core::Gambatte.text(), "Gambatte");
    assert!(Core::Gambatte.runs(Platform::Gb));
    assert!(Core::Gambatte.runs(Platform::Gbc));
    assert!(!Core::Gambatte.runs(Platform::Gba));
    assert!(!Core::Gambatte.runs(Platform::Gbav));
}

#[test]
fn pairs_by_platform() {
    assert_eq!(Core::pair(Platform::Gba), [Core::Mgba, Core::Gpsp]);
    assert_eq!(Core::pair(Platform::Gbav), [Core::Mgba, Core::Gpsp]);
    assert_eq!(Core::pair(Platform::Gb), [Core::Mgba, Core::Gambatte]);
    assert_eq!(Core::pair(Platform::Gbc), [Core::Mgba, Core::Gambatte]);
}

#[test]
fn a_cart_can_ask_for_a_non_default_core_by_hand() {
    let d = tempfile::tempdir().unwrap();
    write_selected_core(d.path(), "Emerald", Core::Gpsp).unwrap();
    assert_ne!(Core::Mgba, Core::Gpsp);
    assert_eq!(
        core_for_platform(d.path(), "Emerald", Platform::Gba, Core::Mgba),
        Core::Gpsp
    );
}

#[test]
fn a_line_naming_a_core_the_platform_cannot_run_is_dropped() {
    let d = tempfile::tempdir().unwrap();
    write_selected_core(d.path(), "Tetris", Core::Gpsp).unwrap();
    assert_eq!(
        core_for_platform(d.path(), "Tetris", Platform::Gb, Core::Mgba),
        Core::Mgba
    );
    assert_eq!(
        core_for_platform(d.path(), "Tetris", Platform::Gbc, Core::Mgba),
        Core::Mgba
    );
}

#[test]
fn the_core_list_has_nine_entries() {
    // slotPlus: Slot's three, then slotPlus's six.
    assert_eq!(Core::ALL.len(), 9);
}

#[test]
fn slotplus_systems_have_one_core_each() {
    assert_eq!(Core::choices(Platform::Gba), vec![Core::Mgba, Core::Gpsp]);
    assert_eq!(Core::choices(Platform::Gb), vec![Core::Mgba, Core::Gambatte]);
    assert_eq!(Core::choices(Platform::Ws), vec![Core::Cygne]);
    assert_eq!(Core::choices(Platform::Lynx), vec![Core::Lynx]);
    assert_eq!(Core::choices(Platform::Gg), vec![Core::Gearsystem]);
    assert_eq!(Core::pair(Platform::Ws), [Core::Cygne, Core::Cygne]);
}

#[test]
fn a_lynx_cart_always_gets_beetle_lynx_whatever_the_card_says() {
    let d = root_with(Some("Chips Challenge = mgba\n"));
    assert_eq!(
        core_for_platform(d.path(), "Chips Challenge", Platform::Lynx, Core::Gambatte),
        Core::Lynx
    );
    assert_eq!(Core::default_for(Platform::Lynx), Core::Lynx);
    assert!(!Core::Mgba.runs(Platform::Lynx));
    assert!(!Core::Gambatte.runs(Platform::Ws));
    assert!(!Core::Lynx.runs(Platform::Gba));
}

#[test]
fn a_game_gear_cart_always_gets_gearsystem_whatever_the_card_says() {
    let d = root_with(Some("Sonic Chaos = mgba\n"));
    assert_eq!(
        core_for_platform(d.path(), "Sonic Chaos", Platform::Gg, Core::Gambatte),
        Core::Gearsystem
    );
    assert_eq!(Core::default_for(Platform::Gg), Core::Gearsystem);
    assert!(!Core::Mgba.runs(Platform::Gg));
    assert!(!Core::Gearsystem.runs(Platform::Gb));
    assert!(!Core::Gearsystem.runs(Platform::Gba));
}

#[test]
fn gearsystem_is_named_and_parsed_the_way_its_library_is() {
    assert_eq!(Core::Gearsystem.as_str(), "gearsystem");
    assert_eq!(Core::parse("Gearsystem"), Some(Core::Gearsystem));
    assert_eq!(Core::Gearsystem.text(), "Gearsystem");
    assert_eq!(Core::ALL[Core::Gearsystem.index()], Core::Gearsystem);
}

#[test]
fn a_hucard_always_gets_beetle_pce_fast() {
    let d = root_with(Some("Bonk = mgba\n"));
    assert_eq!(
        core_for_platform(d.path(), "Bonk", Platform::Tg16, Core::Gambatte),
        Core::Pce
    );
    assert_eq!(Core::default_for(Platform::Tg16), Core::Pce);
    assert_eq!(Core::choices(Platform::Tg16), vec![Core::Pce]);
    assert!(!Core::Pce.runs(Platform::Gba));
    assert_eq!(Core::Pce.as_str(), "pce");
    assert_eq!(Core::parse("PCE"), Some(Core::Pce));
    assert_eq!(Core::Pce.text(), "Beetle PCE Fast");
    assert_eq!(Core::ALL[Core::Pce.index()], Core::Pce);
}

#[test]
fn a_neogeo_pocket_cart_always_gets_beetle_neopop() {
    let d = root_with(Some("Metal Slug = mgba\n"));
    assert_eq!(
        core_for_platform(d.path(), "Metal Slug", Platform::Ngp, Core::Gambatte),
        Core::Ngp
    );
    assert_eq!(Core::default_for(Platform::Ngp), Core::Ngp);
    assert_eq!(Core::choices(Platform::Ngp), vec![Core::Ngp]);
    assert!(!Core::Ngp.runs(Platform::Gg));
    assert!(!Core::Gearsystem.runs(Platform::Ngp));
    assert_eq!(Core::Ngp.as_str(), "ngp");
    assert_eq!(Core::parse("NGP"), Some(Core::Ngp));
    assert_eq!(Core::Ngp.text(), "Beetle NeoPop");
    assert_eq!(Core::ALL[Core::Ngp.index()], Core::Ngp);
}
