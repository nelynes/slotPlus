//! The bezels bundled for slotPlus's own systems.

use slot_gfx::{OUT_H, OUT_W};
use slot_store::Platform;
use slot_ui::{default_game_bezel_face, has_default_game_bezel};

fn at(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * OUT_W + x) * 4) as usize;
    [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
}

#[test]
fn the_game_gear_has_a_black_bezel_with_a_green_power_light_over_its_screen() {
    assert!(has_default_game_bezel(Platform::Gg));
    let face = default_game_bezel_face(Platform::Gg).expect("a Game Gear bezel");
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
fn only_the_game_boys_and_the_game_gear_have_a_bundled_bezel() {
    for platform in Platform::ALL {
        let bundled = matches!(platform, Platform::Gb | Platform::Gbc | Platform::Gg);
        assert_eq!(has_default_game_bezel(platform), bundled, "{platform:?}");
        assert_eq!(
            default_game_bezel_face(platform).is_some(),
            bundled,
            "{platform:?}"
        );
    }
}
