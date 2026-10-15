#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelTurn {
    None,
    Cw,
    Ccw,
}

pub fn panel_turn(
    panel: (u32, u32),
    baseos_ccw: Option<&str>,
    slot_rotate: Option<&str>,
) -> PanelTurn {
    let baseos = match baseos_ccw.map(str::trim) {
        Some("0") => Some(PanelTurn::None),
        Some("90") => Some(PanelTurn::Ccw),
        Some("270") => Some(PanelTurn::Cw),
        _ => None,
    };
    let turn = baseos.unwrap_or(if panel.0 < panel.1 {
        PanelTurn::Cw
    } else {
        PanelTurn::None
    });
    if turn == PanelTurn::None {
        return turn;
    }
    match slot_rotate
        .map(|v| v.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("ccw") => PanelTurn::Ccw,
        Some("cw") => PanelTurn::Cw,
        _ => turn,
    }
}
