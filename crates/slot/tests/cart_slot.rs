mod common;

use slot::app::App;
use slot_store::Platform;
use slot_ui::{Draw, QuickRow, QuickValue, TexId};

fn on_ws_shelf(bespoke: bool) -> (tempfile::TempDir, App) {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    let ws = d.path().join("Games/WS");
    std::fs::create_dir_all(&ws).unwrap();
    for stem in ["Swan", "Swan Two"] {
        std::fs::write(ws.join(format!("{stem}.ws")), vec![0u8; 0x10000]).unwrap();
    }
    let mut state = slot_store::read_slot_state(d.path());
    state.clock_set = true;
    state.bespoke_slot = bespoke;
    slot_store::write_slot_state(d.path(), &state).unwrap();
    let mut a = App::boot(d.path());
    a.set_slot_faces(TexId::from_raw(1), TexId::from_raw(2));
    a.set_plugged_face(TexId::from_raw(3));
    a.set_bespoke_slot(
        Platform::Ws,
        TexId::from_raw(11),
        TexId::from_raw(12),
        TexId::from_raw(13),
        true,
    );
    a.set_bolt_faces([TexId::from_raw(21), TexId::from_raw(22)]);
    for _ in 0..30 {
        a.update(1.0 / 60.0);
    }
    for _ in 0..8 {
        if a.shelf_platform_name() == Some(Platform::Ws.name()) {
            break;
        }
        a.apply(slot_input::Action::NextShelf);
        for _ in 0..60 {
            a.update(1.0 / 60.0);
        }
    }
    assert_eq!(
        a.shelf_platform_name(),
        Some(Platform::Ws.name()),
        "on the WonderSwan shelf"
    );
    (d, a)
}

fn drawn(a: &App, id: usize) -> bool {
    let mut out = Vec::new();
    a.draw(&mut out);
    out.iter().any(|d| match *d {
        Draw::Tex { tex, .. } | Draw::Part { tex, .. } => tex == TexId::from_raw(id),
        _ => false,
    })
}

#[test]
fn the_wonderswan_shelf_keeps_slots_slot_until_cart_slot_is_bespoke() {
    let (_d, a) = on_ws_shelf(false);
    assert_eq!(
        a.quick_value(QuickRow::CartSlot),
        Some(QuickValue::SlotDefault)
    );
    assert!(drawn(&a, 1) && !drawn(&a, 11), "Default draws Slot's slot");
    let (_d, a) = on_ws_shelf(true);
    assert_eq!(
        a.quick_value(QuickRow::CartSlot),
        Some(QuickValue::SlotBespoke)
    );
    assert!(
        drawn(&a, 11) && drawn(&a, 12) && !drawn(&a, 1),
        "Bespoke draws the WonderSwan's own"
    );
}

#[test]
fn a_system_without_a_slot_of_its_own_keeps_slots_when_bespoke() {
    let (_d, mut a) = on_ws_shelf(true);
    a.apply(slot_input::Action::PreviousShelf);
    for _ in 0..60 {
        a.update(1.0 / 60.0);
    }
    assert_ne!(a.shelf_platform_name(), Some(Platform::Ws.name()));
    assert!(drawn(&a, 1) && !drawn(&a, 11));
}

#[test]
fn cart_slot_is_kept_on_the_card() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    assert!(
        !slot_store::read_slot_state(d.path()).bespoke_slot,
        "Default to start"
    );
    let state = slot_store::SlotState {
        bespoke_slot: true,
        ..Default::default()
    };
    slot_store::write_slot_state(d.path(), &state).unwrap();
    assert!(slot_store::read_slot_state(d.path()).bespoke_slot);
}

#[test]
fn the_battery_and_clock_take_the_pale_slots_ink_in_the_dark_theme() {
    let (_d, a) = on_ws_shelf(false);
    assert!(
        !a.band_light(),
        "Slot's dark slot keeps the dark theme's ink"
    );
    assert_eq!(a.status_ink(), slot_ui::footer_ink(false));
    let (_d, a) = on_ws_shelf(true);
    assert!(a.band_light(), "the pearl WonderSwan slot is pale");
    assert_eq!(a.status_ink(), slot_ui::footer_ink(true));
}

#[test]
fn the_wonderswan_has_a_pale_slot_of_its_own_and_the_game_boys_keep_slots() {
    use slot_ui::{bespoke_slot, bespoke_slot_light, SLOT_H, SLOT_W};
    let strips = bespoke_slot(Platform::Ws).expect("the WonderSwan's own");
    for s in strips {
        assert_eq!(s.len(), (SLOT_W * SLOT_H * 4) as usize);
    }
    assert!(bespoke_slot_light(Platform::Ws));
    for p in [Platform::Gba, Platform::Gbc, Platform::Gb] {
        assert!(bespoke_slot(p).is_none(), "{p:?} keeps Slot's");
    }
}

#[test]
fn the_game_gear_has_a_dark_slot_of_its_own() {
    use slot_ui::{bespoke_slot, bespoke_slot_light};
    assert!(bespoke_slot(Platform::Gg).is_some());
    assert!(
        !bespoke_slot_light(Platform::Gg),
        "grey plastic keeps light ink"
    );
}

#[test]
fn the_neo_geo_pocket_has_a_dark_slot_of_its_own() {
    use slot_ui::{bespoke_slot, bespoke_slot_light};
    assert!(bespoke_slot(Platform::Ngp).is_some());
    assert!(!bespoke_slot_light(Platform::Ngp), "carbon keeps light ink");
}

#[test]
fn the_turbografx_has_the_turboexpresss_dark_slot() {
    use slot_ui::{bespoke_slot, bespoke_slot_light};
    assert!(bespoke_slot(Platform::Tg16).is_some());
    assert!(
        !bespoke_slot_light(Platform::Tg16),
        "charcoal keeps light ink"
    );
}

#[test]
fn the_lynx_has_a_dark_slot_of_its_own() {
    use slot_ui::{bespoke_slot, bespoke_slot_light};
    assert!(bespoke_slot(Platform::Lynx).is_some());
    assert!(!bespoke_slot_light(Platform::Lynx), "grey keeps light ink");
}

#[test]
fn the_ds_has_the_silver_ds_lite_s_slot_with_dark_ink_over_it() {
    use slot_ui::{bespoke_slot, bespoke_slot_light};
    assert!(bespoke_slot(Platform::Nds).is_some());
    assert!(bespoke_slot_light(Platform::Nds), "silver is pale");
}
