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

mod vertical {
    use slot_retro::{ButtonMask, WsLayout};

    fn upright(held: u16) -> u16 {
        ButtonMask(held).wonderswan_for(WsLayout::Upright).0
    }

    fn sideways(held: u16) -> u16 {
        ButtonMask(held).wonderswan_for(WsLayout::Sideways).0
    }

    #[test]
    fn a_horizontal_game_keeps_the_modifier_scheme() {
        for held in [
            ButtonMask::UP | ButtonMask::A,
            ButtonMask::L | ButtonMask::LEFT,
            ButtonMask::R | ButtonMask::X,
            ButtonMask::X | ButtonMask::Y,
        ] {
            assert_eq!(
                ButtonMask(held).wonderswan_for(WsLayout::Horizontal),
                ButtonMask(held).wonderswan()
            );
        }
    }

    #[test]
    fn select_does_nothing_in_a_horizontal_game_so_it_cannot_be_turned() {
        let out = ButtonMask(ButtonMask::SELECT | ButtonMask::START | ButtonMask::A)
            .wonderswan_for(WsLayout::Horizontal);
        assert_eq!(out.0, ButtonMask::START | ButtonMask::A);
    }

    #[test]
    fn upright_the_dpad_is_the_y_pad_turned_so_up_is_the_games_up() {
        // The game's up is the hardware's Y-right, its right is Y-down, and so on.
        for (dpad, y) in [
            (ButtonMask::UP, ButtonMask::R),
            (ButtonMask::RIGHT, ButtonMask::L2),
            (ButtonMask::DOWN, ButtonMask::L),
            (ButtonMask::LEFT, ButtonMask::R2),
        ] {
            assert_eq!(upright(dpad), y);
        }
    }

    #[test]
    fn upright_the_face_diamond_is_the_x_pad_turned_the_same_way() {
        for (face, x) in [
            (ButtonMask::X, ButtonMask::RIGHT),
            (ButtonMask::A, ButtonMask::DOWN),
            (ButtonMask::B, ButtonMask::LEFT),
            (ButtonMask::Y, ButtonMask::UP),
        ] {
            assert_eq!(upright(face), x);
        }
    }

    #[test]
    fn sideways_the_dpad_and_face_diamond_keep_the_hardware_directions() {
        for (dpad, y) in [
            (ButtonMask::UP, ButtonMask::R2),
            (ButtonMask::RIGHT, ButtonMask::R),
            (ButtonMask::DOWN, ButtonMask::L2),
            (ButtonMask::LEFT, ButtonMask::L),
        ] {
            assert_eq!(sideways(dpad), y);
        }
        for (face, x) in [
            (ButtonMask::X, ButtonMask::UP),
            (ButtonMask::A, ButtonMask::RIGHT),
            (ButtonMask::B, ButtonMask::DOWN),
            (ButtonMask::Y, ButtonMask::LEFT),
        ] {
            assert_eq!(sideways(face), x);
        }
    }

    #[test]
    fn the_shoulders_are_a_and_b_whichever_way_up() {
        for f in [upright, sideways] {
            assert_eq!(f(ButtonMask::L), ButtonMask::A);
            assert_eq!(f(ButtonMask::R), ButtonMask::B);
            assert_eq!(
                f(ButtonMask::L | ButtonMask::R),
                ButtonMask::A | ButtonMask::B
            );
        }
    }

    #[test]
    fn start_and_select_pass_straight_through() {
        for f in [upright, sideways] {
            assert_eq!(f(ButtonMask::START), ButtonMask::START);
            assert_eq!(f(ButtonMask::SELECT), ButtonMask::SELECT);
        }
    }

    #[test]
    fn nothing_is_left_for_turbo_in_a_vertical_game() {
        for f in [upright, sideways] {
            let out = f(ButtonMask::X | ButtonMask::Y | ButtonMask::A | ButtonMask::B);
            assert_eq!(out & (ButtonMask::X | ButtonMask::Y), 0);
        }
    }
}
