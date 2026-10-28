use std::path::{Path, PathBuf};

use slot_retro::{LibretroCore, MockCore, RetroCore};
use slot_store::{Blending, Core, GbColors, Platform};

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

#[allow(clippy::too_many_arguments)]
pub fn open_core(
    root: &Path,
    core: Core,
    platform: Platform,
    serial: &str,
    colour: bool,
    blending: Blending,
    palette: Option<GbColors>,
    link: Option<u8>,
) -> Opened {
    let paths = candidates(root, core);
    match open_named(
        root, core, platform, serial, colour, blending, palette, link, &paths,
    ) {
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
    open_core_for_palette(root, core, Platform::Gba, serial, colour, None, paths)
}

pub fn open_core_for_palette(
    root: &Path,
    core: Core,
    platform: Platform,
    serial: &str,
    colour: bool,
    palette: Option<GbColors>,
    paths: &[PathBuf],
) -> Box<dyn RetroCore> {
    open_named(
        root,
        core,
        platform,
        serial,
        colour,
        Blending::Off,
        palette,
        None,
        paths,
    )
    .unwrap_or_else(|| {
        report_missing(core, paths);
        Box::new(MockCore::new())
    })
}

#[allow(clippy::too_many_arguments)]
fn open_named(
    root: &Path,
    core: Core,
    platform: Platform,
    serial: &str,
    colour: bool,
    blending: Blending,
    palette: Option<GbColors>,
    link: Option<u8>,
    paths: &[PathBuf],
) -> Option<Box<dyn RetroCore>> {
    let bios = root::bios_dir(root);
    let saves = save_dir_for(root, core);
    for path in paths {
        if !path.exists() {
            continue;
        }
        match LibretroCore::open_with(path, &bios, &saves) {
            Ok(mut opened) => {
                apply_core_options(
                    &mut opened,
                    core,
                    platform,
                    serial,
                    root::has_real_bios(root),
                    colour,
                    palette,
                );
                apply_blending(&mut opened, core, blending);
                if let Some(player) = link {
                    apply_link_options(&mut opened, core, player);
                }
                // Bigger than the 240x160 default: a WonderSwan picture has
                // room to turn at 2x, a TurboGrafx-16 one to be 512 wide.
                let big = match core {
                    Core::Cygne => Some(Platform::Ws),
                    Core::Pce => Some(Platform::Tg16),
                    _ => None,
                };
                if let Some(big) = big {
                    let (w, h) = big.source_size();
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
    chosen: GbColors,
    platform: Platform,
    rom: &Path,
    link: Option<u8>,
) -> Option<GbColors> {
    // Only a Game Boy game takes a palette: slotPlus's other systems aren't GBA either.
    (link.is_none() && matches!(platform.hardware(), Platform::Gb | Platform::Gbc) && dmg_only(rom))
        .then_some(chosen)
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

pub fn colour_option(
    which: Core,
    platform: Platform,
    on: bool,
) -> Option<(&'static str, &'static str)> {
    match which {
        Core::Mgba => Some((
            "mgba_color_correction",
            if on && platform == Platform::Gb {
                "GBC"
            } else if on {
                "Auto"
            } else {
                "OFF"
            },
        )),
        Core::Cygne | Core::Lynx | Core::Gearsystem | Core::Ngp | Core::Pce => None,
        Core::Gpsp => Some((
            "gpsp_color_correction",
            if on { "enabled" } else { "disabled" },
        )),
    }
}

pub fn game_boy_player(platform: Platform, rom: &Path) -> bool {
    platform == Platform::Gba
        && !slot_store::header_code(rom).is_some_and(|code| code.starts_with('M'))
}

pub fn apply_game_boy_player(
    core: &mut dyn RetroCore,
    which: Core,
    platform: Platform,
    rom: &Path,
) {
    if which == Core::Mgba {
        let on = game_boy_player(platform, rom);
        core.set_option("mgba_force_gbp", if on { "ON" } else { "OFF" });
    }
}

/// The core option that sets frame blending, for the cores that have one.
/// slotPlus's other cores have none.
pub fn blend_option(which: Core, blending: Blending) -> Option<(&'static str, &'static str)> {
    Some(match (which, blending.on(which)) {
        (Core::Mgba, Blending::Off) => ("mgba_interframe_blending", "OFF"),
        (Core::Mgba, Blending::Mix) => ("mgba_interframe_blending", "mix"),
        (Core::Mgba, Blending::Ghosting) => ("mgba_interframe_blending", "lcd_ghosting"),
        (Core::Gpsp, Blending::Off) => ("gpsp_frame_mixing", "disabled"),
        (Core::Gpsp, Blending::Mix | Blending::Ghosting) => ("gpsp_frame_mixing", "enabled"),
        _ => return None,
    })
}

pub fn apply_blending(core: &mut LibretroCore, which: Core, blending: Blending) {
    if let Some((key, value)) = blend_option(which, blending) {
        core.set_option(key, value);
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
/// name: Cygne is built from beetle-wswan, whose keys all start `wswan_`, and
/// Beetle PCE Fast's all start `pce_fast_`.
pub fn option_prefix(which: Core) -> &'static str {
    match which {
        Core::Cygne => "wswan",
        Core::Pce => "pce_fast",
        other => other.as_str(),
    }
}

/// Whether the core has a `<prefix>_frameskip` option to set. Beetle Lynx
/// Gearsystem and Beetle NeoPop do not: they are cheap to run and have no
/// frameskip setting.
fn has_frameskip(which: Core) -> bool {
    !matches!(which, Core::Lynx | Core::Gearsystem | Core::Ngp)
}

/// Whether the core writes its own save file rather than handing Slot its
/// save memory. Beetle NeoPop writes a game's flash save to `<name>.flash`
/// in the save directory when the game is unloaded, so the core has to be
/// stopped before the handheld powers off.
pub fn writes_own_saves(which: Core) -> bool {
    which == Core::Ngp
}

/// The directory a core is told to save in. A core that writes its own save
/// files is given its platform's folder under `Saves/`, beside the saves Slot
/// writes for every other platform; the rest are given `Saves/` itself.
pub fn save_dir_for(root: &Path, which: Core) -> PathBuf {
    match which {
        Core::Ngp => root::saves_dir(root).join(Platform::Ngp.dir_name()),
        _ => root::saves_dir(root),
    }
}

pub const LYNX_ROTATION_FILE: &str = "Config/lynx_rotation.txt";

/// Lynx games made to be held on their side, by title, with the turn Beetle
/// Lynx asks for (270 is a cart flagged right, 90 one flagged left). The
/// core finds these from its own list of checksums, which knows only some
/// dumps of each: a "(World)" Klax is not on it, so the core would not turn
/// it. Matching on the title catches every dump of the same game. A game
/// that is turned is drawn for a handheld held turned a quarter anticlockwise.
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
/// core decide from the cart, `0`, `90`, `180` and `270` say how far the game
/// is made to be turned (anticlockwise), and `manual` turns it a quarter each
/// time SELECT is pressed. `Config/lynx_rotation.txt` can set any of these for
/// a game by name; without an entry, a game known to be played on its side is
/// turned and everything else is left to the core.
///
/// The core turns the picture upright for a handheld held the usual way. Slot
/// tells the host the handheld is held turned a quarter anticlockwise, which
/// draws a turned game one quarter less (so it fills the screen) and turns
/// the pad to match.
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

/// Whether SELECT flips a Lynx game between the turned and the upright layout.
/// With `manual` the core owns SELECT: each press steps its picture a quarter.
pub fn lynx_select_toggles(turn: &str) -> bool {
    turn != "manual"
}

pub fn apply_core_options(
    core: &mut LibretroCore,
    which: Core,
    platform: Platform,
    serial: &str,
    bios: bool,
    colour: bool,
    palette: Option<GbColors>,
) {
    if has_frameskip(which) {
        core.set_option(&format!("{}_frameskip", option_prefix(which)), "auto");
    }
    if which == Core::Mgba {
        core.set_option("mgba_sgb_borders", "OFF");
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
        if let Some((key, value)) = colour_option(which, platform, colour) {
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
        // match. The session sets each game's own turn when it starts, from
        // the rotation chosen for it. SELECT is left unused so nothing else
        // can fight it.
        core.set_option("lynx_rot_screen", "auto");
    }
    if which == Core::Gearsystem {
        // No BIOS: Game Gear games boot straight in, and Gearsystem never
        // looks for bios.gg.
        core.set_option("gearsystem_bios_gg", "Disabled");
        core.set_option("gearsystem_bios_sms", "Disabled");
        // The bare 160x144 Game Gear screen at square pixels, the same picture
        // a Game Boy draws, so it lands at 3x like one.
        core.set_option("gearsystem_overscan", "Disabled");
        core.set_option("gearsystem_aspect_ratio", "1:1 PAR");
        // One game on one screen: Slot does not drive the core's two-console
        // link mode.
        core.set_option("gearsystem_link_enable", "Disabled");
    }
    if which == Core::Ngp {
        // Games that speak both languages run in English.
        core.set_option("ngp_language", "english");
    }
    if which == Core::Pce {
        // The TurboExpress pad: two buttons, I and II, with Select and Run.
        // A six-button pad confuses games made for two.
        core.set_option("pce_fast_default_joypad_type_p1", "2 Buttons");
    }
    if which == Core::Gpsp {
        core.set_option("gpsp_serial", serial);
        core.set_option("gpsp_rtc_time_source", "system");
        if bios {
            core.set_option("gpsp_boot_mode", "bios");
        }
        if let Some((key, value)) = colour_option(which, platform, colour) {
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
    fn select_flips_the_layout_unless_the_core_was_given_it_to_turn_the_picture() {
        for turn in ["auto", "0", "90", "180", "270"] {
            assert!(lynx_select_toggles(turn), "{turn}");
        }
        assert!(!lynx_select_toggles("manual"));
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
    fn only_beetle_neopop_writes_its_own_saves_and_it_writes_them_in_saves_ngp() {
        for which in Core::ALL {
            assert_eq!(writes_own_saves(which), which == Core::Ngp, "{which:?}");
        }
        let root = Path::new("/sd");
        assert_eq!(save_dir_for(root, Core::Ngp), Path::new("/sd/Saves/NGP"));
        assert_eq!(save_dir_for(root, Core::Mgba), Path::new("/sd/Saves"));
        assert_eq!(save_dir_for(root, Core::Gearsystem), Path::new("/sd/Saves"));
    }

    #[test]
    fn beetle_lynx_and_gearsystem_have_no_frameskip_option() {
        assert!(has_frameskip(Core::Mgba));
        assert!(has_frameskip(Core::Gpsp));
        assert!(has_frameskip(Core::Cygne));
        assert!(!has_frameskip(Core::Lynx));
        assert!(!has_frameskip(Core::Gearsystem));
        assert!(!has_frameskip(Core::Ngp));
    }

    #[test]
    fn slotplus_s_other_cores_have_no_frame_blending() {
        for which in [Core::Cygne, Core::Lynx] {
            assert_eq!(blend_option(which, Blending::Mix), None, "{which:?}");
        }
    }

    #[test]
    fn each_core_prefixes_its_option_keys_the_way_its_source_does() {
        assert_eq!(option_prefix(Core::Mgba), "mgba");
        assert_eq!(option_prefix(Core::Gpsp), "gpsp");
        // beetle-wswan names every key wswan_*, not cygne_*, and a key it does
        // not know is dropped without a word.
        assert_eq!(option_prefix(Core::Cygne), "wswan");
        assert_eq!(option_prefix(Core::Lynx), "lynx");
        assert_eq!(option_prefix(Core::Gearsystem), "gearsystem");
        // beetle-ngp names its one option ngp_language.
        assert_eq!(option_prefix(Core::Ngp), "ngp");
        // beetle-pce-fast's keys start pce_fast_, not the core's own name.
        assert_eq!(option_prefix(Core::Pce), "pce_fast");
        assert!(has_frameskip(Core::Pce));
    }

    #[test]
    fn each_core_spells_frame_blending_its_own_way() {
        for (which, blending, want) in [
            (
                Core::Mgba,
                Blending::Off,
                ("mgba_interframe_blending", "OFF"),
            ),
            (
                Core::Mgba,
                Blending::Mix,
                ("mgba_interframe_blending", "mix"),
            ),
            (
                Core::Mgba,
                Blending::Ghosting,
                ("mgba_interframe_blending", "lcd_ghosting"),
            ),
            (Core::Gpsp, Blending::Off, ("gpsp_frame_mixing", "disabled")),
            (Core::Gpsp, Blending::Mix, ("gpsp_frame_mixing", "enabled")),
            (
                Core::Gpsp,
                Blending::Ghosting,
                ("gpsp_frame_mixing", "enabled"),
            ),
        ] {
            assert_eq!(
                blend_option(which, blending),
                Some(want),
                "{which:?} {blending:?}"
            );
        }
    }

    #[test]
    fn each_core_spells_colour_correction_its_own_way() {
        assert_eq!(
            colour_option(Core::Mgba, Platform::Gba, true),
            Some(("mgba_color_correction", "Auto"))
        );
        assert_eq!(
            colour_option(Core::Mgba, Platform::Gba, false),
            Some(("mgba_color_correction", "OFF"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, Platform::Gba, true),
            Some(("gpsp_color_correction", "enabled"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, Platform::Gba, false),
            Some(("gpsp_color_correction", "disabled"))
        );
    }

    #[test]
    fn mgba_uses_gbc_correction_for_game_boy_games() {
        assert_eq!(
            colour_option(Core::Mgba, Platform::Gb, true),
            Some(("mgba_color_correction", "GBC"))
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
