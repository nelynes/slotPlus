//! The bezels bundled for slotPlus's own systems.

use slot_gfx::{OUT_H, OUT_W};
use slot_store::Platform;
use slot_ui::{
    bundled_game_bezel_face, bundled_game_bezels, default_game_bezel_face, has_default_game_bezel,
    BezelArt,
};

fn at(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * OUT_W + x) * 4) as usize;
    [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
}

#[test]
fn the_game_gear_has_a_black_bezel_with_a_green_power_light_over_its_screen() {
    assert!(has_default_game_bezel(Platform::Gg));
    let face = default_game_bezel_face(Platform::Gg, BezelArt::Default, OUT_W, OUT_H)
        .expect("a Game Gear bezel");
    assert_eq!(face.len(), (OUT_W * OUT_H * 4) as usize);
    // The 3x picture, 480x432 from (120, 24), shows through.
    for (x, y) in [(360, 240), (130, 34), (590, 446)] {
        assert_eq!(at(&face, x, y)[3], 0, "the window is covered at {x},{y}");
    }
    // The face around it is black and opaque.
    for (x, y) in [(40, 240), (680, 240), (360, 470), (40, 10)] {
        let p = at(&face, x, y);
        assert_eq!(p[3], 255, "the face is see-through at {x},{y}");
        assert!(
            p[0].max(p[1]).max(p[2]) < 30,
            "the face is not black at {x},{y}: {p:?}"
        );
    }
    // The power light glows green, centred above the screen.
    let lamp = at(&face, 360, 11);
    assert!(
        lamp[1] > 200 && lamp[1] > lamp[0] && lamp[1] > lamp[2],
        "no green light: {lamp:?}"
    );
    let side = at(&face, 300, 11);
    assert!(side[1] > side[0] + 20, "the glow does not spread: {side:?}");
}

#[test]
fn the_neogeo_pocket_bezel_leaves_its_whole_3x_screen_clear() {
    let face = default_game_bezel_face(Platform::Ngp, BezelArt::Default, OUT_W, OUT_H)
        .expect("a NeoGeo Pocket bezel");
    // The 160x152 picture at 3x: 480x456 from (120, 12).
    for (x, y) in [(120, 12), (599, 12), (120, 467), (599, 467), (360, 240)] {
        assert_eq!(at(&face, x, y)[3], 0, "the window is covered at {x},{y}");
    }
    for (x, y) in [(119, 240), (600, 240), (360, 11), (360, 468), (680, 240)] {
        assert_eq!(
            at(&face, x, y)[3],
            255,
            "the face is see-through at {x},{y}"
        );
    }
}

#[test]
fn only_the_game_boys_the_game_gear_and_the_neogeo_pocket_have_a_bundled_bezel() {
    for platform in Platform::ALL {
        let bundled = matches!(
            platform,
            Platform::Gb | Platform::Gbc | Platform::Gg | Platform::Ngp
        );
        assert_eq!(has_default_game_bezel(platform), bundled, "{platform:?}");
        assert_eq!(
            default_game_bezel_face(platform, BezelArt::Default, OUT_W, OUT_H).is_some(),
            bundled,
            "{platform:?}"
        );
    }
}

#[test]
fn the_turbografx_16_bundles_tg16_and_pce_bezels_round_its_2x_picture() {
    assert_eq!(
        bundled_game_bezels(Platform::Tg16),
        ["TG16", "TG16 Wide", "PCE", "PCE Wide"]
    );
    assert!(!has_default_game_bezel(Platform::Tg16), "no plain Default");
    for name in ["TG16", "PCE"] {
        let face = bundled_game_bezel_face(Platform::Tg16, name, OUT_W, OUT_H).expect(name);
        assert_eq!(face.len(), (OUT_W * OUT_H * 4) as usize);
        // A 256-wide picture at 2x: 512x480 from (104, 0), all of it clear.
        for (x, y) in [(104, 0), (615, 0), (104, 479), (615, 479), (360, 240)] {
            assert_eq!(
                at(&face, x, y)[3],
                0,
                "{name}: the window is covered at {x},{y}"
            );
        }
        for (x, y) in [(103, 240), (616, 240), (20, 20), (700, 460)] {
            assert_eq!(at(&face, x, y)[3], 255, "{name}: see-through at {x},{y}");
        }
    }
    assert!(bundled_game_bezel_face(Platform::Tg16, "Express", OUT_W, OUT_H).is_none());
}

#[test]
fn only_the_turbografx_16_and_the_ds_have_bezels_bundled_under_their_own_names() {
    for platform in Platform::ALL {
        assert_eq!(
            bundled_game_bezels(platform).is_empty(),
            !matches!(platform, Platform::Tg16 | Platform::Nds),
            "{platform:?}"
        );
    }
}

#[test]
fn the_wide_turbografx_16_bezels_leave_a_320_wide_picture_clear() {
    for name in ["TG16 Wide", "PCE Wide"] {
        let face = bundled_game_bezel_face(Platform::Tg16, name, OUT_W, OUT_H).expect(name);
        // A 320-wide picture at 2x: 640x480 from (40, 0), every pixel clear.
        for y in 0..OUT_H {
            for x in 40..680 {
                assert_eq!(at(&face, x, y)[3], 0, "{name}: covered at {x},{y}");
            }
        }
        // And the face is solid right up to it on both sides.
        for (x, y) in [
            (39, 0),
            (39, 479),
            (680, 0),
            (680, 479),
            (10, 10),
            (710, 470),
        ] {
            assert_eq!(at(&face, x, y)[3], 255, "{name}: see-through at {x},{y}");
        }
    }
}

#[test]
fn the_game_gear_and_pocket_bezels_frame_every_one_of_slots_pictures_the_same() {
    for platform in [Platform::Gg, Platform::Ngp] {
        let default = default_game_bezel_face(platform, BezelArt::Default, OUT_W, OUT_H);
        for kind in [BezelArt::Fit3x2, BezelArt::Integer4x3, BezelArt::Fit4x3] {
            assert_eq!(
                default_game_bezel_face(platform, kind, OUT_W, OUT_H),
                default,
                "{platform:?} {kind:?}"
            );
        }
    }
}

#[test]
fn the_ds_lite_set_draws_each_layout_with_its_screens_clear() {
    use slot_ui::bundled_game_bezel_face_in;
    assert_eq!(bundled_game_bezels(Platform::Nds), ["DS Lite"]);
    let at = |f: &[u8], x: u32, y: u32| f[((y * OUT_W + x) * 4 + 3) as usize];
    for (layout, clear, solid) in [
        ("both-screens.png", (360, 120), (100, 240)),
        ("one-screen.png", (360, 240), (50, 240)),
        ("full-screen.png", (360, 240), (20, 240)),
        ("side-by-side.png", (256, 240), (600, 400)),
        ("picture-in-picture.png", (500, 400), (20, 240)),
    ] {
        let face = bundled_game_bezel_face_in(Platform::Nds, "DS Lite", Some(layout), OUT_W, OUT_H)
            .unwrap_or_else(|| panic!("{layout}"));
        assert_eq!(at(&face, clear.0, clear.1), 0, "{layout}: screen covered");
        assert_eq!(
            at(&face, solid.0, solid.1),
            255,
            "{layout}: bezel see-through"
        );
    }
    assert!(
        bundled_game_bezel_face_in(Platform::Nds, "DS Lite", None, OUT_W, OUT_H).is_none(),
        "no layout, no face"
    );
}
