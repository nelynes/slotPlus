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
    /// The Pokémon mini: a 96x64 black-and-white screen, run by PokeMini.
    Pm,
    /// slotPlus: the PokeDaisy shelf, `Games/PokeDaisy/`. Its Game Boy Advance, Game Boy and
    /// Game Boy Color carts share one folder and one shelf, and play on PokeDaisy's cores; each
    /// is its own system's cart in every other way (`hardware`).
    PdGba,
    PdGb,
    PdGbc,
    #[cfg(feature = "nds")]
    Nds,
}

impl Platform {
    #[cfg(not(feature = "nds"))]
    pub const ALL: [Platform; 10] = [
        Platform::Gba,
        Platform::Gbav,
        Platform::Gbc,
        Platform::Gb,
        Platform::Ws,
        Platform::Lynx,
        Platform::Pm,
        Platform::PdGba,
        Platform::PdGb,
        Platform::PdGbc,
    ];
    #[cfg(feature = "nds")]
    pub const ALL: [Platform; 7] = [
        Platform::Nds,
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
            Platform::Pm => "Pokémon mini",
            Platform::PdGba | Platform::PdGb | Platform::PdGbc => "PokeDaisy",
            #[cfg(feature = "nds")]
            Platform::Nds => "Nintendo DS",
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
            Platform::Pm => "PokeMini",
            Platform::PdGba | Platform::PdGb | Platform::PdGbc => "PokeDaisy",
            #[cfg(feature = "nds")]
            Platform::Nds => "NDS",
        }
    }

    pub fn hardware(self) -> Platform {
        match self {
            Platform::Gbav | Platform::PdGba => Platform::Gba,
            Platform::PdGb => Platform::Gb,
            Platform::PdGbc => Platform::Gbc,
            p => p,
        }
    }

    /// What names the platform in Slot's own files (recents, favorites, the last cart): its
    /// folder, lowercased, except the PokeDaisy shelf's Game Boy and Game Boy Color games, which
    /// share `PokeDaisy` with its Game Boy Advance ones and so take a key of their own.
    pub fn key(self) -> String {
        match self {
            Platform::PdGb => "pokedaisy-gb".into(),
            Platform::PdGbc => "pokedaisy-gbc".into(),
            p => p.dir_name().to_ascii_lowercase(),
        }
    }

    /// slotPlus: a PokeDaisy cart (`Games/PokeDaisy/`), of any of its three systems.
    pub fn is_pokedaisy(self) -> bool {
        matches!(self, Platform::PdGba | Platform::PdGb | Platform::PdGbc)
    }

    /// slotPlus: the platform whose shelf a cart sits on. The PokeDaisy shelf gathers its three
    /// systems' carts on one shelf, which goes by `PdGba`; every other platform is its own.
    pub fn shelf(self) -> Platform {
        if self.is_pokedaisy() {
            Platform::PdGba
        } else {
            self
        }
    }

    /// slotPlus: the folder under `Saves/` that holds a game's battery save. A PokeDaisy game
    /// shares its system's (`Saves/GBA/` for a Game Boy Advance game), so a game moved between
    /// the PokeDaisy shelf and its own keeps its save. Save states stay apart, per core.
    pub fn save_dir_name(self) -> &'static str {
        if self.is_pokedaisy() {
            self.hardware().dir_name()
        } else {
            self.dir_name()
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba | Platform::Gbav | Platform::PdGba => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            // One folder: the extension says which system a PokeDaisy game is.
            Platform::PdGb => &["gb"],
            Platform::PdGbc => &["gbc"],
            Platform::Ws => &["ws", "wsc"],
            Platform::Lynx => &["lnx", "lyx"],
            Platform::Pm => &["min"],
            #[cfg(feature = "nds")]
            Platform::Nds => &["nds"],
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba | Platform::Gbav | Platform::PdGba => (240, 160),
            Platform::Gb | Platform::Gbc | Platform::PdGb | Platform::PdGbc => (160, 144),
            #[cfg(feature = "nds")]
            Platform::Nds => (256, 192),
            Platform::Ws => (224, 144),
            Platform::Lynx => (160, 102),
            Platform::Pm => (96, 64),
        }
    }

    /// The part of the video buffer an "actual size" picture is drawn into,
    /// centred. Most platforms fill the whole 240x160 buffer, which the
    /// canvas shows at 3x. A Lynx picture is only 160x102, so it gets a
    /// 180x120 window and lands at 4x instead of floating in a 3x frame.
    pub fn actual_window(self) -> (u32, u32) {
        match self {
            Platform::Lynx => (180, 120),
            // 96x64 lands at 6x (576x384) in a 120x80 window.
            Platform::Pm => (120, 80),
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
            Platform::Gba
            | Platform::Gbav
            | Platform::Gb
            | Platform::Gbc
            | Platform::Lynx
            | Platform::Pm => (240, 160),
            // PokeDaisy draws its own canvas round the game: 240x160 while playing, 360x240 for its
            // companion and menus, 480x320 for its GBA quick menu (`pokedaisy_panel` is the
            // 720x480 canvas). Each frame is centred in this buffer and fills the screen.
            Platform::PdGba | Platform::PdGb | Platform::PdGbc => (480, 320),
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
