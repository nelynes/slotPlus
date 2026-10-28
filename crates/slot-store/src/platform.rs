use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Platform {
    #[default]
    Gba,
    Gb,
    Gbc,
    Ws,
    Lynx,
    Gbav,
}

impl Platform {
    pub const ALL: [Platform; 6] = [
        Platform::Gba,
        Platform::Gbav,
        Platform::Gbc,
        Platform::Gb,
        Platform::Ws,
        Platform::Lynx,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Gba => "Game Boy Advance",
            Platform::Gb => "Game Boy",
            Platform::Gbc => "Game Boy Color",
            Platform::Ws => "WonderSwan",
            Platform::Lynx => "Atari Lynx",
            Platform::Gbav => "Game Boy Advance Video",
        }
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::Gba => "GBA",
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Ws => "WS",
            Platform::Lynx => "Lynx",
            Platform::Gbav => "GBAV",
        }
    }

    pub fn hardware(self) -> Platform {
        match self {
            Platform::Gbav => Platform::Gba,
            p => p,
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba | Platform::Gbav => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            Platform::Ws => &["ws", "wsc"],
            Platform::Lynx => &["lnx", "lyx"],
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba | Platform::Gbav => (240, 160),
            Platform::Gb | Platform::Gbc => (160, 144),
            Platform::Ws => (224, 144),
            Platform::Lynx => (160, 102),
        }
    }

    /// The part of the video buffer an "actual size" picture is drawn into,
    /// centred. Most platforms fill the whole 240x160 buffer, which the
    /// canvas shows at 3x. A Lynx picture is only 160x102, so it gets a
    /// 180x120 window and lands at 4x instead of floating in a 3x frame.
    pub fn actual_window(self) -> (u32, u32) {
        match self {
            Platform::Lynx => (180, 120),
            _ => (240, 160),
        }
    }

    /// Whether a game on this platform can turn its picture a quarter turn
    /// (a vertical WonderSwan game, or a Lynx game held on its side), which
    /// makes the frame taller than it is wide.
    pub fn turns(self) -> bool {
        matches!(self, Platform::Ws | Platform::Lynx)
    }

    /// Size of the video buffer the host keeps for this platform. GBA, Game
    /// Boy and Lynx keep the 240x160 buffer (a turned Lynx frame is 102x160,
    /// which fits and lands at 3x); WonderSwan needs room for a 144x224
    /// vertical frame, so it gets 360x240 (the canvas at 2x).
    pub fn source_size(self) -> (u32, u32) {
        match self {
            Platform::Gba | Platform::Gbav | Platform::Gb | Platform::Gbc | Platform::Lynx => {
                (240, 160)
            }
            Platform::Ws => (360, 240),
        }
    }

    pub fn accepts(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                self.extensions()
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }
}
