use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Platform {
    #[default]
    Gba,
    Gb,
    Gbc,
    Ws,
}

impl Platform {
    pub const ALL: [Platform; 4] = [Platform::Gba, Platform::Gb, Platform::Gbc, Platform::Ws];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Gba => "Game Boy Advance",
            Platform::Gb => "Game Boy",
            Platform::Gbc => "Game Boy Color",
            Platform::Ws => "WonderSwan",
        }
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::Gba => "GBA",
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Ws => "WS",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            Platform::Ws => &["ws", "wsc"],
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba => (240, 160),
            Platform::Gb | Platform::Gbc => (160, 144),
            Platform::Ws => (224, 144),
        }
    }

    /// Size of the video buffer the host keeps for this platform. GBA and
    /// Game Boy keep the 240x160 buffer; WonderSwan needs room for a
    /// 144x224 vertical frame, so it gets 360x240 (the canvas at 2x).
    pub fn source_size(self) -> (u32, u32) {
        match self {
            Platform::Gba | Platform::Gb | Platform::Gbc => (240, 160),
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
