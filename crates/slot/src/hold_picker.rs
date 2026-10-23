//! The Lynx hold picker: START on a Lynx cart turns it over to show a socket
//! for each way the handheld can be held, and the D-pad moves a chip between
//! them. The socket the chip is in says which way the top of the handheld
//! points while the game is played.

use std::path::Path;

use slot_ui::{ease, Millis, Refusal, CHIP_TIP};

/// How long the cart takes to come up off the shelf and turn over.
pub const OPEN_MS: Millis = 420;
pub const CLOSE_MS: Millis = 320;
pub const HOP_MS: Millis = 200;

pub const LYNX_HOLD_FILE: &str = "Config/lynx_hold.txt";

/// Which way the top of the handheld points while a Lynx game is played.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Hold {
    /// Held the usual way.
    Upright,
    /// Turned a quarter anticlockwise, its top to the left.
    Left,
    /// Turned a quarter clockwise, its top to the right.
    Right,
    /// Upside down.
    Flipped,
}

impl Hold {
    pub const ALL: [Hold; 4] = [Hold::Upright, Hold::Left, Hold::Right, Hold::Flipped];

    /// The socket's place on the back of the cart: north, west, east, south.
    pub fn index(self) -> usize {
        match self {
            Hold::Upright => 0,
            Hold::Left => 1,
            Hold::Right => 2,
            Hold::Flipped => 3,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Hold::Upright => "upright",
            Hold::Left => "left",
            Hold::Right => "right",
            Hold::Flipped => "flipped",
        }
    }

    pub fn parse(text: &str) -> Option<Hold> {
        let text = text.trim().to_ascii_lowercase();
        Hold::ALL.into_iter().find(|h| h.key() == text)
    }

    /// The hold a turn of the Lynx core's picture is upright for: Beetle Lynx's
    /// `90` is a game made for a Lynx turned a quarter anticlockwise, `270`
    /// one turned clockwise.
    fn for_turn(turn: &str) -> Hold {
        match turn {
            "90" => Hold::Left,
            "180" => Hold::Flipped,
            "270" => Hold::Right,
            _ => Hold::Upright,
        }
    }

    /// The host's `slot_handheld_turn` for a handheld held this way.
    pub fn held(self) -> &'static str {
        match self {
            Hold::Upright => "off",
            Hold::Left => "ccw",
            Hold::Flipped => "180",
            Hold::Right => "cw",
        }
    }
}

/// How a Lynx game is started: the turn Beetle Lynx gives its picture and
/// pad, and how the handheld is held for it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LynxSetup {
    pub core_turn: &'static str,
    pub held: &'static str,
}

/// The core works out which way the game is made to be held, from the cart,
/// its own list or `Config/lynx_rotation.txt`. The rotation chosen for the
/// game says how the handheld is held to play it, and Slot turns the picture
/// and the pad so the game is upright to the player held that way. Without a
/// choice, the handheld is held the way the game is made for, so the picture
/// fills the screen as it is. With `manual` the core turns the picture at each
/// SELECT, so the handheld is taken as upright.
pub fn lynx_setup(root: &Path, stem: &str) -> LynxSetup {
    let core_turn = crate::core::lynx_rotation(root, stem);
    let held = match (core_turn, chosen_hold(root, stem)) {
        ("manual", _) => "off",
        (_, Some(hold)) => hold.held(),
        (_, None) => "native",
    };
    LynxSetup { core_turn, held }
}

/// What SELECT switches to in a game: a turned handheld to upright, with the
/// picture stood up and bars at its sides, and an upright one to the way the
/// game is made to be held. Pressed again it goes back.
pub fn select_held(held: &'static str) -> &'static str {
    match held {
        "off" => "native",
        _ => "off",
    }
}

fn chosen_hold(root: &Path, stem: &str) -> Option<Hold> {
    let asked = slot_store::ini::value(root, LYNX_HOLD_FILE, stem)?;
    let hold = Hold::parse(&asked);
    if hold.is_none() {
        eprintln!("slot: lynx: {LYNX_HOLD_FILE}: {asked:?} for {stem} is not a hold");
    }
    hold
}

/// The rotation the picker starts on: the one chosen for the game in
/// `Config/lynx_hold.txt`, or else the way it is made to be held, from
/// `Config/lynx_rotation.txt`, slotPlus's own list or the cart's header.
pub fn hold_for(root: &Path, stem: &str, rom: &Path) -> Hold {
    if let Some(hold) = chosen_hold(root, stem) {
        return hold;
    }
    match crate::core::lynx_rotation(root, stem) {
        "auto" => header_hold(rom),
        turn => Hold::for_turn(turn),
    }
}

/// A socket's place across the board: west -1, north and south 0, east 1.
fn across(hold: Hold) -> f32 {
    match hold {
        Hold::Left => -1.0,
        Hold::Right => 1.0,
        Hold::Upright | Hold::Flipped => 0.0,
    }
}

pub fn write_hold(root: &Path, stem: &str, hold: Hold) -> std::io::Result<()> {
    slot_store::ini::write(root, LYNX_HOLD_FILE, stem, hold.key())
}

/// The way a `.lnx` file's header says the game is held: byte 58 is 1 for a
/// game turned left, a quarter anticlockwise, and 2 for one turned right.
fn header_hold(rom: &Path) -> Hold {
    use std::io::Read;
    let mut head = [0u8; 64];
    let read = std::fs::File::open(rom).and_then(|mut f| f.read_exact(&mut head));
    match (read.is_ok() && &head[..4] == b"LYNX", head[58]) {
        (true, 1) => Hold::Left,
        (true, 2) => Hold::Right,
        _ => Hold::Upright,
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Press {
    Toward(Hold),
    Keep,
    Back,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Nothing,
    Refused,
    Write(Hold),
}

/// Where the chip is drawn: between two sockets while it hops, `t` of the
/// way from `from` to `to`, lifted off the board by `lift`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Chip {
    pub from: Hold,
    pub to: Hold,
    pub t: f32,
    pub lift: f32,
    pub tip: f32,
    pub shake: f32,
}

#[derive(Copy, Clone)]
struct Hop {
    from: Hold,
    started: Millis,
}

#[derive(Copy, Clone)]
struct Close {
    started: Millis,
    from: f32,
}

#[derive(Copy, Clone)]
pub struct HoldPicker {
    seat: Hold,
    opened: Millis,
    started: Option<Millis>,
    hop: Option<Hop>,
    close: Option<Close>,
    refusal: Option<Refusal>,
}

impl HoldPicker {
    pub fn open(seat: Hold, now: Millis) -> Self {
        HoldPicker {
            seat,
            opened: now,
            started: None,
            hop: None,
            close: None,
            refusal: None,
        }
    }

    pub fn start(&mut self, now: Millis) {
        self.started.get_or_insert(now);
    }

    pub fn waiting(&self) -> bool {
        self.started.is_none()
    }

    pub fn waited(&self, now: Millis) -> Millis {
        now.saturating_sub(self.opened)
    }

    pub fn seat(&self) -> Hold {
        self.seat
    }

    pub fn closing(&self) -> bool {
        self.close.is_some()
    }

    /// 0 on the shelf, 1 turned over and in place.
    pub fn openness(&self, now: Millis) -> f32 {
        match (self.close, self.started) {
            (Some(Close { started, from }), _) => {
                (from - now.saturating_sub(started) as f32 / CLOSE_MS as f32).max(0.0)
            }
            (None, Some(started)) => (now.saturating_sub(started) as f32 / OPEN_MS as f32).min(1.0),
            (None, None) => 0.0,
        }
    }

    pub fn finished(&self, now: Millis) -> bool {
        self.close.is_some() && self.openness(now) <= 0.0
    }

    pub fn press(&mut self, press: Press, now: Millis) -> Outcome {
        if self.close.is_some() {
            return Outcome::Nothing;
        }
        let target = match press {
            Press::Keep => {
                self.begin_close(now);
                return Outcome::Write(self.seat);
            }
            Press::Back => {
                self.begin_close(now);
                return Outcome::Nothing;
            }
            Press::Toward(hold) => hold,
        };
        if self.waiting() {
            return Outcome::Nothing;
        }
        if target == self.seat && self.hop_progress(now).is_none() {
            self.refusal = Some(Refusal::started(now));
            return Outcome::Refused;
        }
        if target == self.seat {
            return Outcome::Nothing;
        }
        // A hop that is still in the air starts again from wherever the chip
        // was headed, so a quick second press never jumps it.
        self.hop = Some(Hop {
            from: self.seat,
            started: now,
        });
        self.seat = target;
        self.refusal = None;
        Outcome::Nothing
    }

    pub fn chip(&self, now: Millis) -> Chip {
        let shake = self.refusal.map_or(0.0, |r| r.offset(now));
        match (self.hop, self.hop_progress(now)) {
            (Some(hop), Some(q)) => {
                let arc = (std::f32::consts::PI * q).sin();
                // Leans the way it travels across; straight up or down, not at all.
                let dx = across(self.seat) - across(hop.from);
                let lean = if dx == 0.0 { 0.0 } else { dx.signum() };
                Chip {
                    from: hop.from,
                    to: self.seat,
                    t: ease(q),
                    lift: arc,
                    tip: lean * CHIP_TIP * arc,
                    shake,
                }
            }
            _ => Chip {
                from: self.seat,
                to: self.seat,
                t: 1.0,
                lift: 0.0,
                tip: 0.0,
                shake,
            },
        }
    }

    fn begin_close(&mut self, now: Millis) {
        self.close = Some(Close {
            started: now,
            from: self.openness(now),
        });
    }

    fn hop_progress(&self, now: Millis) -> Option<f32> {
        let hop = self.hop?;
        let q = now.saturating_sub(hop.started) as f32 / HOP_MS as f32;
        (q < 1.0).then_some(q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opened(seat: Hold) -> HoldPicker {
        let mut p = HoldPicker::open(seat, 0);
        p.start(0);
        p
    }

    #[test]
    fn each_direction_sends_the_chip_to_its_own_socket() {
        let mut p = opened(Hold::Upright);
        for (hold, at) in [
            (Hold::Left, 500),
            (Hold::Flipped, 1000),
            (Hold::Right, 1500),
            (Hold::Upright, 2000),
        ] {
            assert_eq!(p.press(Press::Toward(hold), at), Outcome::Nothing);
            assert_eq!(p.seat(), hold);
        }
    }

    #[test]
    fn pressing_toward_the_socket_the_chip_is_in_is_refused() {
        let mut p = opened(Hold::Left);
        assert_eq!(p.press(Press::Toward(Hold::Left), 500), Outcome::Refused);
    }

    #[test]
    fn a_keep_writes_the_seat_and_a_back_writes_nothing() {
        let mut p = opened(Hold::Upright);
        p.press(Press::Toward(Hold::Right), 500);
        assert_eq!(p.press(Press::Keep, 1000), Outcome::Write(Hold::Right));
        let mut p = opened(Hold::Upright);
        p.press(Press::Toward(Hold::Right), 500);
        assert_eq!(p.press(Press::Back, 1000), Outcome::Nothing);
        assert!(p.finished(1000 + CLOSE_MS));
    }

    #[test]
    fn a_hop_carries_the_chip_from_one_socket_to_the_other() {
        let mut p = opened(Hold::Upright);
        p.press(Press::Toward(Hold::Flipped), 1000);
        let mid = p.chip(1000 + HOP_MS / 2);
        assert_eq!((mid.from, mid.to), (Hold::Upright, Hold::Flipped));
        assert!(mid.lift > 0.9 && mid.t > 0.3 && mid.t < 0.7);
        let done = p.chip(1000 + HOP_MS);
        assert_eq!(
            (done.from, done.to, done.lift),
            (Hold::Flipped, Hold::Flipped, 0.0)
        );
    }

    #[test]
    fn a_hold_reads_back_as_it_was_written() {
        let d = tempfile::tempdir().unwrap();
        let rom = d.path().join("Games/Lynx/Klax (World).lnx");
        assert_eq!(
            hold_for(d.path(), "Klax (World)", &rom),
            Hold::Right,
            "made for a Lynx turned right"
        );
        assert_eq!(
            hold_for(d.path(), "Lexis (USA)", &rom),
            Hold::Left,
            "made for one turned left"
        );
        assert_eq!(
            hold_for(d.path(), "California Games (USA)", &rom),
            Hold::Upright
        );
        write_hold(d.path(), "Klax (World)", Hold::Flipped).unwrap();
        assert_eq!(hold_for(d.path(), "Klax (World)", &rom), Hold::Flipped);
    }

    #[test]
    fn a_header_says_which_way_a_game_is_held() {
        let d = tempfile::tempdir().unwrap();
        for (byte, hold) in [(0u8, Hold::Upright), (1, Hold::Left), (2, Hold::Right)] {
            let rom = d.path().join(format!("{byte}.lnx"));
            let mut head = vec![0u8; 64];
            head[..4].copy_from_slice(b"LYNX");
            head[58] = byte;
            std::fs::write(&rom, head).unwrap();
            assert_eq!(hold_for(d.path(), "Some Game (USA)", &rom), hold);
        }
    }

    #[test]
    fn a_chosen_rotation_holds_the_handheld_that_way_for_the_game_the_core_finds() {
        let d = tempfile::tempdir().unwrap();
        for (hold, held) in [
            (Hold::Upright, "off"),
            (Hold::Left, "ccw"),
            (Hold::Flipped, "180"),
            (Hold::Right, "cw"),
        ] {
            write_hold(d.path(), "Klax (World)", hold).unwrap();
            assert_eq!(
                lynx_setup(d.path(), "Klax (World)"),
                LynxSetup {
                    core_turn: "270",
                    held
                }
            );
        }
    }

    #[test]
    fn without_a_choice_the_handheld_is_held_the_way_the_game_is_made_for() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            lynx_setup(d.path(), "California Games (USA)"),
            LynxSetup {
                core_turn: "auto",
                held: "native"
            }
        );
        assert_eq!(
            lynx_setup(d.path(), "Klax (World)"),
            LynxSetup {
                core_turn: "270",
                held: "native"
            }
        );
        slot_store::ini::write(
            d.path(),
            crate::core::LYNX_ROTATION_FILE,
            "Lexis (USA)",
            "manual",
        )
        .unwrap();
        write_hold(d.path(), "Lexis (USA)", Hold::Right).unwrap();
        assert_eq!(
            lynx_setup(d.path(), "Lexis (USA)"),
            LynxSetup {
                core_turn: "manual",
                held: "off"
            },
            "a manual turn is the core's to make"
        );
    }

    #[test]
    fn select_switches_between_upright_and_the_way_it_was_held() {
        for held in ["ccw", "180", "cw", "native"] {
            assert_eq!(select_held(held), "off");
        }
        assert_eq!(select_held("off"), "native");
    }
}
