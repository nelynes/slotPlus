use std::path::{Path, PathBuf};

use slot_retro::{LibretroCore, MockCore, RetroCore};
use slot_store::{Core, GbPalette, Platform};

use crate::root;

const CORE_ENV: &str = "SLOT_CORE";

pub fn dylib_name(core: Core) -> String {
    format!(
        "{}_libretro.{}",
        core.as_str(),
        std::env::consts::DLL_EXTENSION
    )
}

fn candidates(root: &Path, core: Core) -> Vec<PathBuf> {
    if let Some(named) = std::env::var_os(CORE_ENV) {
        return vec![PathBuf::from(named)];
    }
    let name = dylib_name(core);
    let mut paths = vec![root.join("System").join(&name)];
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        paths.push(dir.join(&name));
        paths.push(dir.join("vendor").join(&name));
    }
    paths.push(Path::new("vendor").join(&name));
    paths
}

pub struct Opened {
    pub core: Box<dyn RetroCore>,
    pub named: bool,
}

pub fn open_core(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    palette: Option<GbPalette>,
    link: Option<u8>,
) -> Opened {
    let paths = candidates(root, core);
    match open_named(root, core, serial, colour, palette, link, &paths) {
        Some(core) => Opened { core, named: true },
        None => {
            report_missing(core, &paths);
            Opened {
                core: Box::new(MockCore::new()),
                named: false,
            }
        }
    }
}

pub fn open_core_for(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    paths: &[PathBuf],
) -> Box<dyn RetroCore> {
    open_core_for_palette(root, core, serial, colour, None, paths)
}

pub fn open_core_for_palette(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    palette: Option<GbPalette>,
    paths: &[PathBuf],
) -> Box<dyn RetroCore> {
    open_named(root, core, serial, colour, palette, None, paths).unwrap_or_else(|| {
        report_missing(core, paths);
        Box::new(MockCore::new())
    })
}

fn open_named(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    palette: Option<GbPalette>,
    link: Option<u8>,
    paths: &[PathBuf],
) -> Option<Box<dyn RetroCore>> {
    let bios = root::bios_dir(root);
    let saves = root::saves_dir(root);
    for path in paths {
        if !path.exists() {
            continue;
        }
        match LibretroCore::open_with(path, &bios, &saves) {
            Ok(mut opened) => {
                apply_core_options(
                    &mut opened,
                    core,
                    serial,
                    root::has_real_bios(root),
                    colour,
                    palette,
                );
                if let Some(player) = link {
                    apply_link_options(&mut opened, core, player);
                }
                if core == Core::Cygne {
                    let (w, h) = Platform::Ws.source_size();
                    opened.set_video_size(w, h);
                }
                eprintln!("slot: core {}", path.display());
                return Some(Box::new(opened));
            }
            Err(e) => eprintln!("slot: {}: {e}", path.display()),
        }
    }
    None
}

pub fn palette_for(
    chosen: Option<GbPalette>,
    platform: Platform,
    rom: &Path,
    link: Option<u8>,
) -> Option<GbPalette> {
    chosen.filter(|_| {
        link.is_none() && matches!(platform, Platform::Gb | Platform::Gbc) && dmg_only(rom)
    })
}

pub fn dmg_only(rom: &Path) -> bool {
    use std::io::Read;
    let mut header = [0u8; 0x150];
    std::fs::File::open(rom)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_ok_and(|_| header[0x143] & 0x80 == 0)
}

fn report_missing(core: Core, paths: &[PathBuf]) {
    eprintln!(
        "slot: no {} core found, running the mock test pattern instead. Looked in: {}",
        core.as_str(),
        paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

pub fn colour_option(which: Core, on: bool) -> Option<(&'static str, &'static str)> {
    match which {
        Core::Mgba => Some(("mgba_color_correction", if on { "Auto" } else { "OFF" })),
        Core::Cygne | Core::Lynx => None,
        Core::Gpsp => Some((
            "gpsp_color_correction",
            if on { "enabled" } else { "disabled" },
        )),
    }
}

pub fn apply_link_options(core: &mut LibretroCore, which: Core, player: u8) {
    if which != Core::Mgba {
        return;
    }
    core.set_option("mgba_link", "on");
    core.set_option("mgba_link_player", &player.to_string());
    eprintln!("slot: core: link mode on, player {player}");
}

/// The prefix a core puts on its option keys. It is not always the core's own
/// name: Cygne is built from beetle-wswan, whose keys all start `wswan_`.
pub fn option_prefix(which: Core) -> &'static str {
    match which {
        Core::Cygne => "wswan",
        other => other.as_str(),
    }
}

/// Whether the core has a `<prefix>_frameskip` option to set. Beetle Lynx
/// does not: it is cheap to run and has no frameskip setting.
fn has_frameskip(which: Core) -> bool {
    which != Core::Lynx
}

pub const LYNX_ROTATION_FILE: &str = "Config/lynx_rotation.txt";

/// Lynx games made to be held on their side, by title, with the turn Beetle
/// Lynx asks for. The core finds these from its own list of checksums, which
/// knows only some dumps of each: a "(World)" Klax is not on it, so it stays
/// upright. Matching on the title catches every dump of the same game.
const LYNX_SIDEWAYS: [(&str, &str); 4] = [
    ("klax", "270"),
    ("gauntlet - the third encounter", "270"),
    ("lexis", "90"),
    ("nfl football", "90"),
];

fn lynx_title(stem: &str) -> String {
    let cut = stem
        .find(" (")
        .into_iter()
        .chain(stem.find(" ["))
        .min()
        .unwrap_or(stem.len());
    stem[..cut].trim().to_ascii_lowercase()
}

/// What to give Beetle Lynx's `lynx_rot_screen` for a game: `auto` lets the
/// core decide from the cart, `0`, `90`, `180` and `270` turn the picture that
/// many degrees anticlockwise, and `manual` turns it a quarter each time
/// SELECT is pressed. `Config/lynx_rotation.txt` can set any of these for a
/// game by name; without an entry, a game known to be played on its side is
/// turned and everything else is left to the core.
pub fn lynx_rotation(root: &Path, stem: &str) -> &'static str {
    const CHOICES: [&str; 6] = ["auto", "manual", "0", "90", "180", "270"];
    if let Some(asked) = slot_store::ini::value(root, LYNX_ROTATION_FILE, stem) {
        let asked = asked.trim().to_ascii_lowercase();
        if let Some(choice) = CHOICES.iter().find(|c| **c == asked) {
            return choice;
        }
        eprintln!(
            "slot: lynx: {LYNX_ROTATION_FILE}: {asked:?} for {stem} is not one of {CHOICES:?}"
        );
    }
    let title = lynx_title(stem);
    LYNX_SIDEWAYS
        .iter()
        .find(|(name, _)| *name == title)
        .map_or("auto", |(_, turn)| *turn)
}

pub fn apply_core_options(
    core: &mut LibretroCore,
    which: Core,
    serial: &str,
    bios: bool,
    colour: bool,
    palette: Option<GbPalette>,
) {
    if has_frameskip(which) {
        core.set_option(&format!("{}_frameskip", option_prefix(which)), "auto");
    }
    if which == Core::Mgba {
        core.set_option("mgba_sgb_borders", "OFF");
        core.set_option("mgba_force_gbp", "ON");
        match palette {
            Some(p) => {
                core.set_option("mgba_gb_model", "Game Boy");
                core.set_option("mgba_gb_colors_preset", "0");
                core.set_option("mgba_gb_colors", p.core_name());
            }
            None => {
                core.set_option("mgba_gb_model", "Autodetect");
                core.set_option("mgba_gb_colors_preset", "1");
                core.set_option("mgba_gb_colors", "GBC Dark Green →A");
            }
        }
        if let Some((key, value)) = colour_option(which, colour) {
            core.set_option(key, value);
        }
    }
    if which == Core::Cygne {
        // Slot does all the button mapping, so the core must always read the
        // landscape map whatever way up it draws the picture.
        core.set_option("wswan_rotate_keymap", "disabled");
    }
    if which == Core::Lynx {
        // The core turns the picture itself to the way the cart was made to be
        // held, and asks Slot to draw it that way; it also turns the D-pad to
        // match. SELECT is left unused so nothing else can fight it.
        core.set_option("lynx_rot_screen", "auto");
    }
    if which == Core::Gpsp {
        core.set_option("gpsp_serial", serial);
        core.set_option("gpsp_rtc_time_source", "system");
        if bios {
            core.set_option("gpsp_boot_mode", "bios");
        }
        if let Some((key, value)) = colour_option(which, colour) {
            core.set_option(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn a_lynx_game_known_to_be_played_on_its_side_is_turned_whatever_its_dump() {
        let root = tempfile::tempdir().unwrap();
        for stem in [
            "Klax (World)",
            "Klax (USA, Europe)",
            "Klax",
            "Gauntlet - The Third Encounter (World)",
            "KLAX (World) [!]",
        ] {
            assert_eq!(lynx_rotation(root.path(), stem), "270", "{stem}");
        }
        for stem in ["Lexis (USA)", "NFL Football (USA, Europe)"] {
            assert_eq!(lynx_rotation(root.path(), stem), "90", "{stem}");
        }
    }

    #[test]
    fn other_lynx_games_are_left_to_the_core() {
        let root = tempfile::tempdir().unwrap();
        for stem in [
            "Double Dragon (USA, Europe)",
            "Klaxon (World)",
            "Super Klax (World)",
            "Gauntlet (World)",
        ] {
            assert_eq!(lynx_rotation(root.path(), stem), "auto", "{stem}");
        }
    }

    #[test]
    fn a_game_named_in_the_rotation_file_gets_what_the_file_says() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("Config")).unwrap();
        std::fs::write(
            root.path().join(LYNX_ROTATION_FILE),
            "Klax (World) = auto\nDouble Dragon (USA, Europe) = 90\nLexis (USA) = Manual\nRaiden = sideways\n",
        )
        .unwrap();
        assert_eq!(lynx_rotation(root.path(), "Klax (World)"), "auto");
        assert_eq!(
            lynx_rotation(root.path(), "Double Dragon (USA, Europe)"),
            "90"
        );
        assert_eq!(lynx_rotation(root.path(), "Lexis (USA)"), "manual");
        assert_eq!(lynx_rotation(root.path(), "Raiden"), "auto");
    }

    #[test]
    fn only_beetle_lynx_has_no_frameskip_option() {
        assert!(has_frameskip(Core::Mgba));
        assert!(has_frameskip(Core::Gpsp));
        assert!(has_frameskip(Core::Cygne));
        assert!(!has_frameskip(Core::Lynx));
    }

    #[test]
    fn each_core_prefixes_its_option_keys_the_way_its_source_does() {
        assert_eq!(option_prefix(Core::Mgba), "mgba");
        assert_eq!(option_prefix(Core::Gpsp), "gpsp");
        // beetle-wswan names every key wswan_*, not cygne_*, and a key it does
        // not know is dropped without a word.
        assert_eq!(option_prefix(Core::Cygne), "wswan");
        assert_eq!(option_prefix(Core::Lynx), "lynx");
    }

    #[test]
    fn each_core_spells_colour_correction_its_own_way() {
        assert_eq!(
            colour_option(Core::Mgba, true),
            Some(("mgba_color_correction", "Auto"))
        );
        assert_eq!(
            colour_option(Core::Mgba, false),
            Some(("mgba_color_correction", "OFF"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, true),
            Some(("gpsp_color_correction", "enabled"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, false),
            Some(("gpsp_color_correction", "disabled"))
        );
    }

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn candidates_search_the_named_cores_own_filename_only() {
        let _g = lock();
        std::env::remove_var(CORE_ENV);

        let root = Path::new("/root");
        let mgba = candidates(root, Core::Mgba);
        let gpsp = candidates(root, Core::Gpsp);
        assert_ne!(mgba, gpsp, "the two cores searched the same paths");
        assert!(!mgba.is_empty());
        assert!(!gpsp.is_empty());
        for path in &mgba {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert!(name.starts_with("mgba_libretro"), "{name} is not mGBA's");
        }
        for path in &gpsp {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert!(name.starts_with("gpsp_libretro"), "{name} is not gpSP's");
        }
    }

    #[test]
    fn candidates_search_the_roots_own_system_directory() {
        let _g = lock();
        std::env::remove_var(CORE_ENV);
        let root = Path::new("/some/content/root");
        assert_eq!(
            candidates(root, Core::Gpsp)[0],
            root.join("System").join(dylib_name(Core::Gpsp)),
        );
    }

    #[test]
    fn the_env_override_ignores_which_core_was_asked_for() {
        let _g = lock();
        std::env::set_var(CORE_ENV, "/dev/null/named-core");
        let root = Path::new("/root");
        let mgba = candidates(root, Core::Mgba);
        let gpsp = candidates(root, Core::Gpsp);
        std::env::remove_var(CORE_ENV);
        assert_eq!(mgba, vec![PathBuf::from("/dev/null/named-core")]);
        assert_eq!(
            mgba, gpsp,
            "the override stopped winning for one of the cores"
        );
    }
}
