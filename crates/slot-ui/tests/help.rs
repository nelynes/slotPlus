use slot_ui::{quick_description_face, QuickRow};

#[test]
fn every_row_has_a_sentence_of_help_that_fits_on_one_line() {
    let mut long = Vec::new();
    for row in QuickRow::ALL {
        let help = row.help();
        assert!(help.ends_with('.'), "{row:?}: {help}");
        assert!(
            help.chars().next().is_some_and(char::is_uppercase),
            "{row:?}: {help}"
        );
        let face = quick_description_face(&[help]);
        // One line, with room either side on the 720px screen.
        if face.w > 640 || face.h > 23 {
            long.push(format!("{row:?} is {}px: {help}", face.w));
        }
    }
    assert!(long.is_empty(), "{}", long.join("\n"));
}

#[test]
fn shelf_life_help_names_only_its_three_steps() {
    // Shelf Life's hidden step stays out of everything a player reads.
    for row in [
        QuickRow::ShelfLife,
        QuickRow::Wear,
        QuickRow::Dust,
        QuickRow::ShrinkWrap,
    ] {
        let help = row.help().to_lowercase();
        for word in ["100", "stain", "level 4", "secret"] {
            assert!(!help.contains(word), "{row:?}: {help}");
        }
    }
}
