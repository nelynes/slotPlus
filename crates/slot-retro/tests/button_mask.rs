use slot_retro::ButtonMask;

#[test]
fn without_select_lets_go_of_select_and_nothing_else() {
    let held = ButtonMask::SELECT
        | ButtonMask::START
        | ButtonMask::UP
        | ButtonMask::A
        | ButtonMask::B
        | ButtonMask::L
        | ButtonMask::R;
    let rest = held & !ButtonMask::SELECT;
    assert_eq!(ButtonMask(held).without_select(), ButtonMask(rest));
    assert_eq!(ButtonMask(rest).without_select(), ButtonMask(rest));
    assert_eq!(ButtonMask::default().without_select(), ButtonMask(0));
}
