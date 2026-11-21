//! slotPlus: one sentence of help for each menu row, shown in a dim line just above the slot
//! strip while the row is chosen. Kept to one line on the 720px screen (see the test in
//! `crates/slot-ui/tests/help.rs`).

use crate::QuickRow;

impl QuickRow {
    /// What the row does, in a sentence.
    pub fn help(self) -> &'static str {
        match self {
            // Settings, the top level.
            QuickRow::General => "Theme, wallpaper, cart slot and shelf life.",
            QuickRow::Network => "How games are arranged, and how games and art arrive.",
            QuickRow::Game => "Fast forward, rewind, rumble and resuming games.",
            QuickRow::Screen => "Brightness, colour, and how each system's games look.",
            QuickRow::Controls => "Shelf buttons, in-game shortcuts and game buttons.",
            QuickRow::Online => "Wi-Fi, RetroAchievements and updates.",
            QuickRow::System => "Kid Mode, clock, nickname, power and battery.",

            // Look.
            QuickRow::Theme => "Dark or light menus and shelf.",
            QuickRow::Wallpaper => "The picture behind the shelf: Slot's, per shelf, or yours.",
            QuickRow::WallpaperTint => "Darkens your wallpaper so the carts stand out.",
            QuickRow::CartSlot => "Bespoke shows each system's own handheld at the slot.",
            QuickRow::ShowTitles => "Writes each game's name under its cart.",
            QuickRow::ShelfLife => "Shrink wrap, dust and wear that carts pick up over time.",
            QuickRow::ShrinkWrap => "Unplayed games stay shrink-wrapped, price sticker on.",
            QuickRow::Dust => "Carts you haven't played in a while slowly gather dust.",
            QuickRow::Wear => "Carts you play a lot slowly show it on their labels.",
            QuickRow::CleanResets => "What holding B on a cart cleans off: dust, wear or both.",
            QuickRow::CleanAllCarts => "Cleans and unwraps every cart at once.",

            // Library.
            QuickRow::ShelfMode => "A shelf for each system, or your collections instead.",
            QuickRow::Recents => "Adds a shelf of the games you played last.",
            QuickRow::FetchArt => "Downloads label art for games that don't have any yet.",
            QuickRow::Server => "Manage the card from a browser on the same Wi-Fi.",
            QuickRow::Romm => "Pair with your RomM server to get games from it.",
            #[cfg(feature = "nds")]
            QuickRow::FetchBoxArt => "Downloads DS box art from your RomM server.",
            QuickRow::RommProtocol => "HTTPS for a secure server, HTTP for a plain one.",
            QuickRow::RommServer => "Your RomM server's address.",
            QuickRow::RommPort => "Your RomM server's port.",
            QuickRow::RommSelfSigned => "Trusts a server whose certificate you made yourself.",
            QuickRow::RommPair => "Pairs the handheld with your RomM server.",
            QuickRow::RommStatus => "Whether the handheld is paired with RomM.",
            QuickRow::RommPairedServer => "The RomM server the handheld is paired with.",
            QuickRow::RommDisconnect => "Unpairs the handheld from your RomM server.",

            // Play.
            QuickRow::FastForward => "How fast fast forward runs, and whether you hear it.",
            QuickRow::FastForwardSound => "Plays sound while fast forwarding.",
            QuickRow::Rewind => "Lets you rewind the last few seconds of play.",
            QuickRow::Rumble => "Rumble for games that support it.",
            QuickRow::LeftStick => "Lets the left stick work as the D-pad.",
            QuickRow::ResumeInsert => "Picks up where you left off: on insert, power on, or both.",
            QuickRow::ResumePowerOn => "Puts the last game back in when the handheld starts.",

            // Screen.
            QuickRow::ScreenGeneral => "Brightness and colour for the whole screen.",
            QuickRow::PlayScreen => "This game's picture: filter, frame, colours and size.",
            QuickRow::Brightness | QuickRow::PlayBrightness => "How bright the screen is.",
            QuickRow::BlueLight | QuickRow::PlayBlueLight => {
                "Warms the picture, easier on the eyes at night."
            }
            QuickRow::Saturation | QuickRow::PlaySaturation => "Makes colours more vivid.",
            QuickRow::Contrast | QuickRow::PlayContrast => "Deepens darks and brightens lights.",
            QuickRow::ScreenGba => "How Game Boy Advance games look.",
            QuickRow::ScreenGbc => "How Game Boy Color games look.",
            QuickRow::ScreenGb => "How Game Boy games look.",
            QuickRow::ScreenWs => "How WonderSwan games look.",
            QuickRow::ScreenLynx => "How Atari Lynx games look.",
            QuickRow::ScreenGg => "How Game Gear games look.",
            QuickRow::ScreenNgp => "How NeoGeo Pocket games look.",
            QuickRow::ScreenTg16 => "How TurboGrafx-16 games look.",
            QuickRow::ScreenNds => "How Nintendo DS games look.",
            QuickRow::ScreenPm => "How Pokémon mini games look.",
            QuickRow::Shader
            | QuickRow::GbaShader
            | QuickRow::GbcShader
            | QuickRow::GbShader
            | QuickRow::WsShader
            | QuickRow::LynxShader
            | QuickRow::GgShader
            | QuickRow::NgpShader
            | QuickRow::Tg16Shader
            | QuickRow::NdsShader
            | QuickRow::PmShader => "A filter over the picture, like an LCD grid or dots.",
            QuickRow::ColourCorrection
            | QuickRow::GbaColourCorrection
            | QuickRow::GbcColourCorrection
            | QuickRow::GbColourCorrection => "Shifts colours to look like the original screen.",
            QuickRow::Palette | QuickRow::GbPalette => {
                "The colours original Game Boy games are shown in."
            }
            QuickRow::Bezel
            | QuickRow::GbaBezel
            | QuickRow::GbcBezel
            | QuickRow::GbBezel
            | QuickRow::WsBezel
            | QuickRow::LynxBezel
            | QuickRow::GgBezel
            | QuickRow::NgpBezel
            | QuickRow::Tg16Bezel
            | QuickRow::NdsBezel
            | QuickRow::PmBezel => "A frame drawn around the game.",
            QuickRow::Scale | QuickRow::GbaScale | QuickRow::GbcScale | QuickRow::GbScale => {
                "Integer keeps pixels sharp; Fit fills more of the screen."
            }
            QuickRow::Stretch | QuickRow::GbcStretch | QuickRow::GbStretch => {
                "Full Screen fills the screen, not the original shape."
            }
            QuickRow::Blending
            | QuickRow::GbaBlending
            | QuickRow::GbcBlending
            | QuickRow::GbBlending => "Blends frames to stop flicker; Ghosting adds LCD trails.",
            QuickRow::Overlay
            | QuickRow::GbaOverlay
            | QuickRow::GbcOverlay
            | QuickRow::GbOverlay => "A picture laid over the game, from the Overlays folder.",
            QuickRow::PmFullScreen => "Shows the picture larger, filling more of the screen.",

            // Controls.
            QuickRow::ControlsShelf => "Buttons for changing shelves, favorites and recents.",
            QuickRow::ControlsCollections => "Buttons for changing collections.",
            QuickRow::Shortcuts => "Combos for saves, fast forward, screenshots and more.",
            QuickRow::ControlsInGame => "Which handheld button presses each game button.",
            QuickRow::ShelfPrev => "The button that moves to the previous shelf.",
            QuickRow::ShelfNext => "The button that moves to the next shelf.",
            QuickRow::ShelfFavorite => "The button that marks a game as a favorite.",
            QuickRow::ShelfForget => "The button that takes a game off the Recents shelf.",
            QuickRow::CollPrev => "The button that moves to the previous collection.",
            QuickRow::CollNext => "The button that moves to the next collection.",
            QuickRow::GameUp => "The handheld button that presses Up in games.",
            QuickRow::GameDown => "The handheld button that presses Down in games.",
            QuickRow::GameLeft => "The handheld button that presses Left in games.",
            QuickRow::GameRight => "The handheld button that presses Right in games.",
            QuickRow::GameA => "The handheld button that presses A in games.",
            QuickRow::GameB => "The handheld button that presses B in games.",
            QuickRow::GameSelect => "The handheld button that presses Select in games.",
            QuickRow::GameStart => "The handheld button that presses Start in games.",
            QuickRow::GameL => "The handheld button that presses L in games.",
            QuickRow::GameR => "The handheld button that presses R in games.",
            QuickRow::GameTurboA => "Presses A over and over while you hold it.",
            QuickRow::GameTurboB => "Presses B over and over while you hold it.",
            QuickRow::ScSave => "Saves the game as it is right now, to a save state.",
            QuickRow::ScLoad => "Loads your last save state.",
            QuickRow::ScReset => "Restarts the game from the beginning.",
            QuickRow::ScRewind => "Rewinds the game while you hold it.",
            QuickRow::ScFastForward => "Runs the game fast while you hold it.",
            QuickRow::ScFastForwardToggle => "Turns fast forward on or off with one press.",
            QuickRow::ScScreenshot => "Takes a screenshot of the game.",
            QuickRow::ScGuide => "Opens this game's guide.",
            QuickRow::ScBrightDown => "Turns the brightness down while you play.",
            QuickRow::ScBrightUp => "Turns the brightness up while you play.",
            QuickRow::ScBlueDown => "Turns the blue light filter down while you play.",
            QuickRow::ScBlueUp => "Turns the blue light filter up while you play.",
            QuickRow::ScColour => "Turns color correction on or off while you play.",
            QuickRow::ScPaletteBack => "Steps back to the previous Game Boy palette.",
            QuickRow::ScPaletteNext => "Steps on to the next Game Boy palette.",
            QuickRow::ScShaderBack => "Steps back to the previous shader.",
            QuickRow::ScShaderNext => "Steps on to the next shader.",
            QuickRow::ScSunlightDown => "Less sun on the solar sensor in games like Boktai.",
            QuickRow::ScSunlightUp => "More sun on the solar sensor in games like Boktai.",

            // Online.
            QuickRow::Wifi => "Turn Wi-Fi on or off, and choose a network.",
            QuickRow::WifiPower => "Turns Wi-Fi on or off.",
            QuickRow::Networks => "Choose and join a Wi-Fi network.",
            QuickRow::Achievements => "Sign in to earn RetroAchievements in your games.",
            QuickRow::CheckUpdates => "Looks for a newer slotPlus and installs it.",
            QuickRow::RaUsername => "Your RetroAchievements username.",
            QuickRow::RaPassword => "Your RetroAchievements password.",
            QuickRow::RaMode => "Casual keeps save states, rewind and fast forward on.",
            QuickRow::RaEncore => "Earn achievements again that your account already has.",
            QuickRow::RaDownload => "Downloads achievement sets for the games on your card.",
            QuickRow::RaSetsStatus => "How the download is going.",
            QuickRow::RaSetsSystems => "How many systems have been checked so far.",
            QuickRow::RaSetsFound => "Sets found for your games, new and already on the card.",
            QuickRow::RaSetsNone => "Games that RetroAchievements has no set for.",
            QuickRow::RaSetsBadges => "New achievement badges downloaded.",

            // System.
            QuickRow::Kid => "Locks the handheld to chosen shelves, with a play limit.",
            QuickRow::KidMode => "Locks the handheld to one shelf, or to a few shelves.",
            QuickRow::KidShelf => "Which shelf or collection Kid Mode keeps.",
            QuickRow::KidAllowance => "How long kids can play each day before the cart ejects.",
            QuickRow::DateTime => "Sets the clock and time zone.",
            QuickRow::Nickname => "Your name, used for link play and the DS.",
            QuickRow::LidOff => "How long the lid can stay shut before power off.",
            QuickRow::BatteryDisplay => "Shows the battery as an icon, a percentage or both.",
            QuickRow::About => "Version, credits and licences.",

            // The in-game menu.
            QuickRow::Resume => "Back to the game.",
            QuickRow::States => "Save states for this game: save, load and see your slots.",
            QuickRow::Screenshots => "Screenshots you've taken of this game.",
            QuickRow::Guide => "This game's guide, from the Guides folder.",
            QuickRow::Settings => "Screen, play and control settings, while you play.",
            QuickRow::ExitGame => "Leaves the game and goes back to the shelf.",
            QuickRow::UndoSave => "Puts back the save state you just saved over.",
            QuickRow::UndoLoad => "Goes back to where you were before loading a state.",
            QuickRow::UndoReset => "Goes back to where you were before the reset.",
            QuickRow::PlayGameplay => "Fast forward, rewind, rumble and resume.",
            QuickRow::PlayControls => "Buttons and shortcuts.",
        }
    }
}
