use slot::panel_turn::{panel_turn, PanelTurn};

const RG28XX: (u32, u32) = (480, 640);
const RG35XXSP: (u32, u32) = (640, 480);

#[test]
fn baseos_turns_the_rg28xx_counter_clockwise() {
    assert_eq!(panel_turn(RG28XX, Some("90"), None), PanelTurn::Ccw);
}

#[test]
fn baseos_can_name_the_other_quarter_turn() {
    assert_eq!(panel_turn(RG28XX, Some("270"), None), PanelTurn::Cw);
}

#[test]
fn a_landscape_panel_is_not_turned() {
    assert_eq!(panel_turn(RG35XXSP, Some("0"), None), PanelTurn::None);
    assert_eq!(panel_turn(RG35XXSP, None, None), PanelTurn::None);
}

#[test]
fn an_older_baseos_guesses_from_the_panel_as_before() {
    assert_eq!(panel_turn(RG28XX, None, None), PanelTurn::Cw);
    assert_eq!(panel_turn(RG28XX, Some(""), None), PanelTurn::Cw);
    assert_eq!(panel_turn(RG28XX, Some("180"), None), PanelTurn::Cw);
}

#[test]
fn slot_rotate_still_picks_the_direction() {
    assert_eq!(panel_turn(RG28XX, None, Some("ccw")), PanelTurn::Ccw);
    assert_eq!(panel_turn(RG28XX, Some("90"), Some(" CW ")), PanelTurn::Cw);
    assert_eq!(panel_turn(RG28XX, Some("90"), Some("ccw")), PanelTurn::Ccw);
    assert_eq!(
        panel_turn(RG28XX, Some("90"), Some("sideways")),
        PanelTurn::Ccw
    );
}

#[test]
fn slot_rotate_does_not_turn_a_landscape_panel() {
    assert_eq!(
        panel_turn(RG35XXSP, Some("0"), Some("ccw")),
        PanelTurn::None
    );
}
