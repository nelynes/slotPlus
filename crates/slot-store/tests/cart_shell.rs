use slot_store::cart_shell::choices;
use slot_store::{Outline, ShellChoice, ShellFinish};

#[test]
fn a_choice_reads_back_as_itself() {
    for outline in [Outline::Auto, Outline::Notched, Outline::Rounded] {
        for finish in [ShellFinish::Solid, ShellFinish::Clear, ShellFinish::Glitter] {
            for colour in [[0, 0, 0], [0xff, 0xff, 0xff], [0x86, 0xb9, 0xbf]] {
                for back in [None, Some([0xe2, 0xdb, 0xc8])] {
                    let c = ShellChoice {
                        outline,
                        colour,
                        finish,
                        back,
                    };
                    assert_eq!(ShellChoice::parse(&c.to_value()), Some(c));
                }
            }
        }
    }
    assert_eq!(
        ShellChoice::parse("rounded 86B9BF glitter")
            .unwrap()
            .to_value(),
        "rounded 86b9bf glitter"
    );
}

#[test]
fn a_two_tone_choice_keeps_its_back() {
    let c = ShellChoice::parse("auto 8e1b2b solid E2DBC8").unwrap();
    assert_eq!(c.colour, [0x8e, 0x1b, 0x2b]);
    assert_eq!(c.back, Some([0xe2, 0xdb, 0xc8]));
    assert_eq!(c.to_value(), "auto 8e1b2b solid e2dbc8");
    assert_eq!(ShellChoice::parse("auto 8e1b2b solid").unwrap().back, None);
}

#[test]
fn anything_else_is_no_choice() {
    for bad in [
        "",
        "auto",
        "auto 86b9bf",
        "auto 86b9bf solid extra",
        "auto 86b9bf solid e2dbc8 extra",
        "auto 86b9bf solid e2dbc",
        "square 86b9bf solid",
        "auto 86b9b solid",
        "auto 86b9bg solid",
        "auto 86b9bf matte",
        "Auto 86b9bf solid",
    ] {
        assert_eq!(ShellChoice::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn choices_keeps_the_lines_that_parse() {
    let got = choices("# c\nA = rounded 112233 solid\nB = nonsense\nC = auto aabbcc clear\n");
    assert_eq!(got.len(), 2);
    assert_eq!(got["A"].outline, Outline::Rounded);
    assert_eq!(got["C"].finish, ShellFinish::Clear);
}

#[test]
fn scan_gives_each_cart_its_line_and_the_rest_none() {
    let d = tempfile::tempdir().unwrap();
    for dir in ["Games/GB", "System", "Config"] {
        std::fs::create_dir_all(d.path().join(dir)).unwrap();
    }
    for stem in ["Chosen", "Plain", "Garbled"] {
        std::fs::write(d.path().join(format!("Games/GB/{stem}.gb")), [0u8; 0x150]).unwrap();
    }
    std::fs::write(
        d.path().join(slot_store::CART_SHELL_FILE),
        "Chosen = rounded 112233 clear\nGarbled = rounded\n",
    )
    .unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    let shell = |stem: &str| carts.iter().find(|c| c.stem == stem).unwrap().shell;
    assert_eq!(shell("Chosen"), ShellChoice::parse("rounded 112233 clear"));
    assert_eq!(shell("Plain"), None);
    assert_eq!(shell("Garbled"), None);

    std::fs::remove_file(d.path().join(slot_store::CART_SHELL_FILE)).unwrap();
    assert!(slot_store::scan(d.path())
        .unwrap()
        .iter()
        .all(|c| c.shell.is_none()));
}

#[test]
fn the_labels_file_lies_over_the_system_one_line_by_line() {
    let d = tempfile::tempdir().unwrap();
    for dir in ["Games/GB", "System", "Config", "Labels"] {
        std::fs::create_dir_all(d.path().join(dir)).unwrap();
    }
    for stem in ["Both", "SystemOnly", "LabelsOnly", "BackToAuto", "Neither"] {
        std::fs::write(d.path().join(format!("Games/GB/{stem}.gb")), [0u8; 0x150]).unwrap();
    }
    std::fs::write(
        d.path().join(slot_store::CART_SHELL_FILE),
        "Both = rounded 111111 solid\nSystemOnly = notched 222222 clear\nBackToAuto = rounded 333333 glitter\n",
    )
    .unwrap();
    std::fs::write(
        d.path().join(slot_store::LABELS_SHELL_FILE),
        "Both = auto 444444 clear\nLabelsOnly = auto 555555 solid\nBackToAuto = auto\n",
    )
    .unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    let shell = |stem: &str| carts.iter().find(|c| c.stem == stem).unwrap().shell;
    assert_eq!(
        shell("Both"),
        ShellChoice::parse("auto 444444 clear"),
        "Labels wins"
    );
    assert_eq!(
        shell("SystemOnly"),
        ShellChoice::parse("notched 222222 clear")
    );
    assert_eq!(shell("LabelsOnly"), ShellChoice::parse("auto 555555 solid"));
    assert_eq!(
        shell("BackToAuto"),
        None,
        "`auto` in Labels puts the cart back on slot's pick"
    );
    assert_eq!(shell("Neither"), None);
}

#[test]
fn a_card_with_only_the_labels_file_reads_it() {
    let d = tempfile::tempdir().unwrap();
    for dir in ["Games/GBA", "Labels"] {
        std::fs::create_dir_all(d.path().join(dir)).unwrap();
    }
    std::fs::write(d.path().join("Games/GBA/Pak.gba"), [0u8; 0x100]).unwrap();
    std::fs::write(
        d.path().join(slot_store::LABELS_SHELL_FILE),
        "Pak = auto c2332e clear\n",
    )
    .unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    assert_eq!(carts[0].shell, ShellChoice::parse("auto c2332e clear"));
}

#[test]
fn a_line_matches_its_cart_whichever_way_the_accent_is_spelled() {
    let composed = "Pok\u{e9}mon";
    let decomposed = "Poke\u{301}mon";
    for (file, key) in [(composed, decomposed), (decomposed, composed)] {
        let d = tempfile::tempdir().unwrap();
        for dir in ["Games/GBA", "Labels"] {
            std::fs::create_dir_all(d.path().join(dir)).unwrap();
        }
        std::fs::write(d.path().join(format!("Games/GBA/{file}.gba")), [0u8; 0x100]).unwrap();
        std::fs::write(
            d.path().join(slot_store::LABELS_SHELL_FILE),
            format!("{key} = auto e2b413 clear\n"),
        )
        .unwrap();
        let carts = slot_store::scan(d.path()).unwrap();
        assert_eq!(
            carts[0].shell,
            ShellChoice::parse("auto e2b413 clear"),
            "a {:?} line missed a {:?} rom",
            key.as_bytes(),
            file.as_bytes()
        );
    }
}
