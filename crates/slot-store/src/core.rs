use std::collections::HashMap;
use std::path::Path;

use crate::Platform;

pub const SELECTED_CORE_FILE: &str = "Config/selected_core.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Core {
    #[default]
    Mgba,
    Gpsp,
    Cygne,
    Lynx,
    Gearsystem,
    Ngp,
    Pce,
    Dsperate,
}

impl Core {
    pub const ALL: [Core; 8] = [
        Core::Mgba,
        Core::Gpsp,
        Core::Cygne,
        Core::Lynx,
        Core::Gearsystem,
        Core::Ngp,
        Core::Pce,
        Core::Dsperate,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Core::Mgba => "mgba",
            Core::Gpsp => "gpsp",
            Core::Cygne => "cygne",
            Core::Lynx => "lynx",
            Core::Gearsystem => "gearsystem",
            Core::Ngp => "ngp",
            Core::Pce => "pce",
            Core::Dsperate => "dsperate",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn choices(platform: Platform) -> Vec<Core> {
        Core::ALL.into_iter().filter(|c| c.runs(platform)).collect()
    }

    pub fn text(self) -> &'static str {
        match self {
            Core::Mgba => "mGBA",
            Core::Gpsp => "gpSP",
            Core::Cygne => "Beetle Cygne",
            Core::Lynx => "Beetle Lynx",
            Core::Gearsystem => "Gearsystem",
            Core::Ngp => "Beetle NeoPop",
            Core::Pce => "Beetle PCE Fast",
            Core::Dsperate => "DSperate",
        }
    }

    pub fn parse(s: &str) -> Option<Core> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mgba" => Some(Core::Mgba),
            "gpsp" => Some(Core::Gpsp),
            "cygne" => Some(Core::Cygne),
            "lynx" => Some(Core::Lynx),
            "gearsystem" => Some(Core::Gearsystem),
            "ngp" => Some(Core::Ngp),
            "pce" => Some(Core::Pce),
            "dsperate" => Some(Core::Dsperate),
            _ => None,
        }
    }

    pub fn runs(self, platform: Platform) -> bool {
        match self {
            Core::Mgba => matches!(
                platform,
                Platform::Gba | Platform::Gbav | Platform::Gb | Platform::Gbc
            ),
            Core::Gpsp => platform.hardware() == Platform::Gba,
            Core::Cygne => platform == Platform::Ws,
            Core::Lynx => platform == Platform::Lynx,
            Core::Gearsystem => platform == Platform::Gg,
            Core::Ngp => platform == Platform::Ngp,
            Core::Pce => platform == Platform::Tg16,
            Core::Dsperate => platform == Platform::Nds,
        }
    }

    pub fn default_for(platform: Platform) -> Core {
        match platform {
            Platform::Gba | Platform::Gb | Platform::Gbc | Platform::Gbav => Core::Mgba,
            Platform::Ws => Core::Cygne,
            Platform::Lynx => Core::Lynx,
            Platform::Gg => Core::Gearsystem,
            Platform::Ngp => Core::Ngp,
            Platform::Tg16 => Core::Pce,
            Platform::Nds => Core::Dsperate,
        }
    }
}

pub fn read_selected_cores(root: &Path) -> HashMap<String, Core> {
    crate::ini::read(root, SELECTED_CORE_FILE)
        .into_iter()
        .filter_map(|(stem, name)| Core::parse(&name).map(|core| (stem, core)))
        .collect()
}

pub fn core_for(root: &Path, stem: &str) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .unwrap_or_default()
}

pub fn core_for_platform(root: &Path, stem: &str, platform: Platform) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .filter(|core| core.runs(platform))
        .unwrap_or_else(|| Core::default_for(platform))
}

pub fn write_selected_core(root: &Path, stem: &str, core: Core) -> std::io::Result<()> {
    crate::ini::write(root, SELECTED_CORE_FILE, stem, core.as_str())
}
