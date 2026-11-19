//! slotPlus: the PokeDaisy shelf. `Games/PokeDaisy/` holds Game Boy Advance, Game Boy and Game Boy
//! Color games side by side; they share one shelf and play on PokeDaisy's cores.

mod common;

use std::path::Path;

use common::tmp_root_with_carts;
use slot::app::App;
use slot_store::{
    core_for_platform, read_slot_state, scan, write_slot_state, Core, Platform, Recent, SlotState,
};

fn put(root: &Path, dir: &str, name: &str) {
    let dir = root.join(dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), [0u8; 512]).unwrap();
}

fn pokedaisy_card() -> tempfile::TempDir {
    let d = tmp_root_with_carts(&["Drill Dozer"]);
    for name in [
        "Pokemon Emerald.gba",
        "Pokemon Crystal.gbc",
        "Pokemon Red.gb",
    ] {
        put(d.path(), "Games/PokeDaisy", name);
    }
    d
}

#[test]
fn each_game_on_the_shelf_is_its_own_system_by_its_extension() {
    let d = pokedaisy_card();
    let carts: Vec<_> = scan(d.path())
        .unwrap()
        .into_iter()
        .filter(|c| c.platform.is_pokedaisy())
        .map(|c| (c.stem, c.platform))
        .collect();
    assert_eq!(
        carts,
        [
            ("Pokemon Crystal".to_string(), Platform::PdGbc),
            ("Pokemon Emerald".to_string(), Platform::PdGba),
            ("Pokemon Red".to_string(), Platform::PdGb),
        ],
        "sorted by name together, not by system"
    );
    for (_, p) in carts {
        assert_eq!(
            p.shelf(),
            Platform::PdGba,
            "{p:?} is on the PokeDaisy shelf"
        );
    }
    assert_eq!(Platform::PdGb.hardware(), Platform::Gb);
    assert_eq!(Platform::PdGbc.hardware(), Platform::Gbc);
    assert_eq!(Platform::PdGba.hardware(), Platform::Gba);
}

#[test]
fn the_three_systems_share_one_shelf() {
    let d = pokedaisy_card();
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            shelf: Some("pokedaisy".into()),
            shelf_cart: Some(Recent {
                platform: Platform::PdGb,
                stem: "Pokemon Red".into(),
            }),
            ..SlotState::default()
        },
    )
    .unwrap();
    let a = App::boot(d.path());
    assert_eq!(a.shelf_key(), "pokedaisy");
    assert_eq!(
        a.shelf_stems(),
        ["Pokemon Crystal", "Pokemon Emerald", "Pokemon Red"]
    );
}

#[test]
fn a_game_without_pokedaisy_art_wears_its_systems_label() {
    let d = pokedaisy_card();
    put(d.path(), "Labels/GBA", "Pokemon Emerald.png");
    put(d.path(), "Labels/PokeDaisy", "Pokemon Red.png");
    put(d.path(), "Labels/GB", "Pokemon Red.png");
    let carts = scan(d.path()).unwrap();
    let label = |stem: &str| {
        carts
            .iter()
            .find(|c| c.stem == stem && c.platform.is_pokedaisy())
            .and_then(|c| c.label.clone())
    };
    assert_eq!(
        label("Pokemon Emerald"),
        Some(d.path().join("Labels/GBA/Pokemon Emerald.png"))
    );
    assert_eq!(
        label("Pokemon Red"),
        Some(d.path().join("Labels/PokeDaisy/Pokemon Red.png")),
        "the PokeDaisy folder's own art comes first"
    );
    assert_eq!(label("Pokemon Crystal"), None);
}

#[test]
fn a_game_keeps_its_systems_save_so_moving_it_keeps_the_save() {
    let d = pokedaisy_card();
    put(d.path(), "Saves/GBA", "Pokemon Emerald.sav");
    assert!(slot::persist::has_save(
        d.path(),
        Platform::PdGba,
        "Pokemon Emerald"
    ));
    assert!(!slot::persist::has_save(
        d.path(),
        Platform::PdGbc,
        "Pokemon Crystal"
    ));
    assert_eq!(Platform::PdGbc.save_dir_name(), "GBC");
    assert_eq!(Platform::PdGb.save_dir_name(), "GB");
}

#[test]
fn every_game_on_the_shelf_plays_on_pokedaisy() {
    let d = pokedaisy_card();
    let core = |p| core_for_platform(d.path(), "Pokemon Emerald", p, Core::Gambatte);
    assert_eq!(core(Platform::PdGba), Core::PokedaisyGba);
    assert_eq!(core(Platform::PdGb), Core::PokedaisyGbc);
    assert_eq!(core(Platform::PdGbc), Core::PokedaisyGbc);
    assert_eq!(Core::choices(Platform::PdGba), [Core::PokedaisyGba]);
    assert!(
        !Core::Mgba.runs(Platform::PdGba),
        "never the GBA shelf's cores"
    );
    assert_eq!(
        slot::core::dylib_name(Core::PokedaisyGbc),
        format!("pokedaisy_gbc_libretro.{}", std::env::consts::DLL_EXTENSION)
    );
}

#[test]
fn a_pokedaisy_game_boy_game_is_remembered_as_one() {
    let d = pokedaisy_card();
    let s = SlotState {
        cart: Some("Pokemon Red".into()),
        cart_platform: Some(Platform::PdGb),
        recents: vec![Recent {
            platform: Platform::PdGbc,
            stem: "Pokemon Crystal".into(),
        }],
        clock_set: true,
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    let back = read_slot_state(d.path());
    assert_eq!(back.cart_platform, Some(Platform::PdGb));
    assert_eq!(back.recents, s.recents);
}
