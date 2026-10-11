use slot_retro::ButtonMask;

fn mapped(held: u16) -> u16 {
    ButtonMask(held).wonderswan().0
}

#[test]
fn without_a_modifier_the_dpad_is_the_x_pad_and_a_b_are_a_b() {
    let held =
        ButtonMask::UP | ButtonMask::LEFT | ButtonMask::A | ButtonMask::B | ButtonMask::START;
    assert_eq!(mapped(held), held);
}

#[test]
fn x_and_y_are_left_for_turbo_when_no_modifier_is_held() {
    let held = ButtonMask::X | ButtonMask::Y;
    assert_eq!(mapped(held), held);
}

#[test]
fn l_turns_the_dpad_into_the_y_pad() {
    let cases = [
        (ButtonMask::UP, ButtonMask::R2),
        (ButtonMask::RIGHT, ButtonMask::R),
        (ButtonMask::DOWN, ButtonMask::L2),
        (ButtonMask::LEFT, ButtonMask::L),
    ];
    for (dpad, y) in cases {
        assert_eq!(mapped(ButtonMask::L | dpad), y);
    }
}

#[test]
fn l_leaves_the_face_buttons_alone() {
    let held = ButtonMask::L | ButtonMask::A | ButtonMask::B | ButtonMask::START;
    assert_eq!(
        mapped(held),
        ButtonMask::A | ButtonMask::B | ButtonMask::START
    );
}

#[test]
fn r_turns_the_face_diamond_into_the_y_pad() {
    let cases = [
        (ButtonMask::X, ButtonMask::R2),
        (ButtonMask::A, ButtonMask::R),
        (ButtonMask::B, ButtonMask::L2),
        (ButtonMask::Y, ButtonMask::L),
    ];
    for (face, y) in cases {
        assert_eq!(mapped(ButtonMask::R | face), y);
    }
}

#[test]
fn r_leaves_the_dpad_on_the_x_pad() {
    let held = ButtonMask::R | ButtonMask::UP | ButtonMask::RIGHT;
    assert_eq!(mapped(held), ButtonMask::UP | ButtonMask::RIGHT);
}

#[test]
fn both_modifiers_drive_the_y_pad_from_both_clusters() {
    let held = ButtonMask::L | ButtonMask::R | ButtonMask::UP | ButtonMask::A;
    assert_eq!(mapped(held), ButtonMask::R2 | ButtonMask::R);
}

#[test]
fn the_shoulders_never_reach_the_core_as_themselves() {
    assert_eq!(mapped(ButtonMask::L), 0);
    assert_eq!(mapped(ButtonMask::R), 0);
    assert_eq!(mapped(ButtonMask::L | ButtonMask::R), 0);
}

#[test]
fn start_and_select_pass_through_with_a_modifier_held() {
    let held = ButtonMask::L | ButtonMask::R | ButtonMask::START | ButtonMask::SELECT;
    assert_eq!(mapped(held), ButtonMask::START | ButtonMask::SELECT);
}

#[test]
fn y_pad_bits_match_libretros_joypad_ids() {
    assert_eq!(ButtonMask::L2, 1 << 12);
    assert_eq!(ButtonMask::R2, 1 << 13);
}
