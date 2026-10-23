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
}

/// The hold a game is played in: the one chosen for it in
/// `Config/lynx_hold.txt`, or else the one slotPlus has always used, turned
/// a quarter anticlockwise for a game made to be held on its side.
pub fn hold_for(root: &Path, stem: &str, rom: &Path) -> Hold {
    if let Some(asked) = slot_store::ini::value(root, LYNX_HOLD_FILE, stem) {
        if let Some(hold) = Hold::parse(&asked) {
            return hold;
        }
        eprintln!("slot: lynx: {LYNX_HOLD_FILE}: {asked:?} for {stem} is not a hold");
    }
    let turn = crate::core::lynx_rotation(root, stem);
    let sideways = matches!(turn, "90" | "270") || (turn == "auto" && header_turns(rom));
    match sideways {
        true => Hold::Left,
        false => Hold::Upright,
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

/// Whether a `.lnx` file's header says the game is played turned: byte 58
/// is 1 for a game turned left and 2 for one turned right.
fn header_turns(rom: &Path) -> bool {
    use std::io::Read;
    let mut head = [0u8; 64];
    let read = std::fs::File::open(rom).and_then(|mut f| f.read_exact(&mut head));
    read.is_ok() && &head[..4] == b"LYNX" && matches!(head[58], 1 | 2)
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
            Hold::Left,
            "a known sideways game"
        );
        assert_eq!(
            hold_for(d.path(), "California Games (USA)", &rom),
            Hold::Upright
        );
        write_hold(d.path(), "Klax (World)", Hold::Right).unwrap();
        assert_eq!(hold_for(d.path(), "Klax (World)", &rom), Hold::Right);
    }
}
