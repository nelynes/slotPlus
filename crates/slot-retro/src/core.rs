use std::fmt;
use std::path::Path;

use crate::link::Link;
use crate::rumble::Rumble;

pub const GBA_W: u32 = 240;
pub const GBA_H: u32 = 160;

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct ButtonMask(pub u16);

impl ButtonMask {
    pub const B: u16 = 1 << 0;
    pub const Y: u16 = 1 << 1;
    pub const SELECT: u16 = 1 << 2;
    pub const START: u16 = 1 << 3;
    pub const UP: u16 = 1 << 4;
    pub const DOWN: u16 = 1 << 5;
    pub const LEFT: u16 = 1 << 6;
    pub const RIGHT: u16 = 1 << 7;
    pub const A: u16 = 1 << 8;
    pub const X: u16 = 1 << 9;
    pub const L: u16 = 1 << 10;
    pub const R: u16 = 1 << 11;
    pub const L2: u16 = 1 << 12;
    pub const R2: u16 = 1 << 13;

    /// The same buttons with SELECT let go, for a game that never sees it.
    pub fn without_select(self) -> ButtonMask {
        ButtonMask(self.0 & !Self::SELECT)
    }

    /// WonderSwan has two four-way pads. The X pad is the D-pad as usual; the
    /// Y pad is reached through a shoulder-button modifier, because the SP has
    /// no spare buttons for it. Cygne reads the Y pad as R2 (up), R (right),
    /// L2 (down) and L (left).
    ///
    /// - L held: the D-pad drives the Y pad instead of the X pad.
    /// - R held: the face diamond (X up, A right, B down, Y left) drives the Y
    ///   pad instead of acting as A, B and the turbo buttons.
    ///
    /// L and R are modifiers only and never reach the core as themselves.
    pub fn wonderswan(self) -> ButtonMask {
        let held = |bit: u16| self.0 & bit != 0;
        let mut out = self.0 & !(Self::L | Self::R);
        if held(Self::L) {
            out &= !(Self::UP | Self::DOWN | Self::LEFT | Self::RIGHT);
            for (from, to) in [
                (Self::UP, Self::R2),
                (Self::RIGHT, Self::R),
                (Self::DOWN, Self::L2),
                (Self::LEFT, Self::L),
            ] {
                if held(from) {
                    out |= to;
                }
            }
        }
        if held(Self::R) {
            out &= !(Self::X | Self::A | Self::B | Self::Y);
            for (from, to) in [
                (Self::X, Self::R2),
                (Self::A, Self::R),
                (Self::B, Self::L2),
                (Self::Y, Self::L),
            ] {
                if held(from) {
                    out |= to;
                }
            }
        }
        ButtonMask(out)
    }

    /// The WonderSwan remap for a game laid out `layout`. Horizontal games
    /// use the shoulder-modifier scheme above and never see SELECT, so the
    /// core cannot rotate them. Vertical games drop the
    /// modifiers: the D-pad is the Y pad, the face diamond is the X pad and
    /// the shoulders are A (L) and B (R), as in the core's own portrait map.
    /// The core is told to read the landscape map, so this is the only mapping.
    pub fn wonderswan_for(self, layout: WsLayout) -> ButtonMask {
        let held = |bit: u16| self.0 & bit != 0;
        let sideways = match layout {
            // A horizontal game has nothing to gain from turning the picture, and
            // the buttons would not follow it, so SELECT does nothing.
            WsLayout::Horizontal => return ButtonMask(self.wonderswan().0 & !Self::SELECT),
            WsLayout::Upright => false,
            WsLayout::Sideways => true,
        };
        let mut out = self.0 & (Self::SELECT | Self::START);
        // Y pad is R2 (up), R (right), L2 (down), L (left) as the core sees it.
        // Turned upright, the game's up is the hardware's Y-right, and so on.
        let (dpad, face) = if sideways {
            (
                [
                    (Self::UP, Self::R2),
                    (Self::RIGHT, Self::R),
                    (Self::DOWN, Self::L2),
                    (Self::LEFT, Self::L),
                ],
                [
                    (Self::X, Self::UP),
                    (Self::A, Self::RIGHT),
                    (Self::B, Self::DOWN),
                    (Self::Y, Self::LEFT),
                ],
            )
        } else {
            (
                [
                    (Self::UP, Self::R),
                    (Self::RIGHT, Self::L2),
                    (Self::DOWN, Self::L),
                    (Self::LEFT, Self::R2),
                ],
                [
                    (Self::X, Self::RIGHT),
                    (Self::A, Self::DOWN),
                    (Self::B, Self::LEFT),
                    (Self::Y, Self::UP),
                ],
            )
        };
        for (from, to) in dpad.into_iter().chain(face) {
            if held(from) {
                out |= to;
            }
        }
        if held(Self::L) {
            out |= Self::A;
        }
        if held(Self::R) {
            out |= Self::B;
        }
        ButtonMask(out)
    }

    pub fn without_turbo(self) -> ButtonMask {
        ButtonMask(self.0 & !(Self::X | Self::Y))
    }

    pub fn turbo(self, frame: u32) -> ButtonMask {
        let mut mask = self.0 & !(Self::X | Self::Y);
        if (frame / 3).is_multiple_of(2) {
            if self.0 & Self::X != 0 {
                mask |= Self::A;
            }
            if self.0 & Self::Y != 0 {
                mask |= Self::B;
            }
        }
        ButtonMask(mask)
    }
}

/// How a WonderSwan game wants to be played.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WsLayout {
    /// A horizontal game.
    Horizontal,
    /// A vertical game shown upright on a device that stays upright.
    Upright,
    /// A vertical game shown sideways on a device turned 90 degrees
    /// counter-clockwise.
    Sideways,
}

#[derive(Copy, Clone, Debug)]
pub struct AvInfo {
    pub fps: f64,
    pub sample_rate: f64,
}

#[derive(Debug)]
pub enum CoreError {
    Io(std::io::Error),
    Load(String),
    Unsupported(String),
    State(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Io(e) => write!(f, "io: {e}"),
            CoreError::Load(m) => write!(f, "load: {m}"),
            CoreError::Unsupported(m) => write!(f, "unsupported: {m}"),
            CoreError::State(m) => write!(f, "state: {m}"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e)
    }
}

pub trait RetroCore: Send {
    fn load(&mut self, rom: &Path) -> Result<(), CoreError>;
    fn run_frame(&mut self, input: ButtonMask);
    fn run_frame_linked(&mut self, p1: ButtonMask, _p2: ButtonMask) {
        self.run_frame(p1);
    }
    fn set_option(&mut self, _key: &str, _value: &str) {}
    fn set_frame_skip(&mut self, _skip: bool) {}
    fn video_xrgb8888(&self) -> &[u8];
    /// Width and height of the buffer `video_xrgb8888` returns.
    fn video_size(&self) -> (u32, u32) {
        (GBA_W, GBA_H)
    }
    /// Width and height of the picture the core last drew inside that buffer.
    fn frame_size(&self) -> (u32, u32) {
        self.video_size()
    }
    fn take_audio(&mut self) -> Vec<i16>;
    fn serialize(&mut self) -> Result<Vec<u8>, CoreError>;
    fn unserialize(&mut self, data: &[u8]) -> Result<(), CoreError>;
    fn save_ram(&self) -> Option<Vec<u8>>;
    fn load_save_ram(&mut self, data: &[u8]) -> Result<(), CoreError>;
    fn av_info(&self) -> AvInfo;
    fn rumble(&self) -> Rumble {
        Rumble::default()
    }
    fn net(&self) -> Link {
        Link::default()
    }
    fn start_link(&mut self, _client_id: u16) {}
    fn pump_link(&mut self) {}
    fn stop_link(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Recorder(Vec<ButtonMask>);

    impl RetroCore for Recorder {
        fn load(&mut self, _rom: &Path) -> Result<(), CoreError> {
            Ok(())
        }
        fn run_frame(&mut self, input: ButtonMask) {
            self.0.push(input);
        }
        fn video_xrgb8888(&self) -> &[u8] {
            &[]
        }
        fn take_audio(&mut self) -> Vec<i16> {
            Vec::new()
        }
        fn serialize(&mut self) -> Result<Vec<u8>, CoreError> {
            Ok(Vec::new())
        }
        fn unserialize(&mut self, _data: &[u8]) -> Result<(), CoreError> {
            Ok(())
        }
        fn save_ram(&self) -> Option<Vec<u8>> {
            None
        }
        fn load_save_ram(&mut self, _data: &[u8]) -> Result<(), CoreError> {
            Ok(())
        }
        fn av_info(&self) -> AvInfo {
            AvInfo {
                fps: 60.0,
                sample_rate: 48_000.0,
            }
        }
    }

    #[test]
    fn a_core_without_link_mode_runs_player_1_alone() {
        let mut core = Recorder::default();
        core.run_frame_linked(ButtonMask(ButtonMask::A), ButtonMask(ButtonMask::B));
        assert_eq!(core.0, vec![ButtonMask(ButtonMask::A)]);
    }
}
