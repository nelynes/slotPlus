# Changelog

## Unreleased

slotPlus on Slot 1.8.0.

- Brings in everything from Slot 1.8.0, below. Run-Ahead is gone from slotPlus too.
- The In-Game controls page covers the WonderSwan and Atari Lynx. Both keep the L and R rows, since L1 and R1 are the Lynx's Option 1 and Option 2 and the WonderSwan's Y pad. In a WonderSwan game the Turbo A and Turbo B rows are called X Button and Y Button, since those buttons also work the Y pad with R1 held and the X pad in vertical games.
- slotPlusUltra: the Game Gear, NeoGeo Pocket and TurboGrafx-16 get the In-Game controls page without L and R rows, since L1 and R1 stretch the picture there.
- slotPlusUltra: the SD card repair stops the NeoGeo Pocket core first, as powering off does, so its save is written if the card still takes it.

## [1.8.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.8.0)

- When the SD card turns read-only, SELECT + START repairs it: slot steps aside, checks FAT32 or exFAT, and restarts. (#72, #111)
- Power off, including the shutdown after the lid has been closed for 3 minutes, no longer hangs on a black screen: the Wi-Fi driver is unloaded first, as Restart already does.
- If slot stops responding with the lid closed, a separate watcher powers off a minute after the Lid Off time.
- Run-Ahead is removed. It scrambled saves in games with a 128 KB flash chip, such as Super Mario Advance 4. (#104)
- Turbo A and Turbo B have their own rows on the In-Game controls page and can be moved off X and Y. (#127)
- Loading a state while a game is saving to a 128 KB flash chip no longer writes the save into the wrong half of the chip.
- The cart insertion animation no longer stutters, and a large ROM still loading after the cart seats powers the screen on to a blank panel that brightens until the game appears. (#119)
- On Anbernics with a left stick, the stick works as the D-pad. Left Stick as D-Pad on the Gameplay page turns it off. (#106, #107)

## [1.7.0-plus2.0](https://github.com/nelynes/slotPlus/releases/tag/v1.7.0-plus2.0)

slotPlus 2.0 comes in two versions, both built from this release. Pick one from the release's downloads:

- **`slot-plus.zip`, slotPlus:** Game Boy Advance, Game Boy and Game Boy Color, plus WonderSwan and Atari Lynx.
- **`slot-plusultra.zip`, slotPlusUltra:** everything in slotPlus, plus Game Gear, NeoGeo Pocket and TurboGrafx-16.

To upgrade, replace the `System` folder on your card with the one from the zip. Switching between the two versions works the same way.

### Both versions

- HDMI output on handhelds with a mini-HDMI port, such as the RG SP. Plug in a TV and the shelf and games show on the TV at 1080p, with the sound there too and the handheld's screen off. Plug in or pull the cable at any time: Slot saves the game, starts again on the other screen within a few seconds and picks up where you were. Closing the lid doesn't sleep the handheld while it's on the TV.
- The README now covers both versions, with a table of the added systems and which version has each, and shows the Plus and Ultra shelves in motion. Controls, label art sizes, bezels, saves and the games that turn sideways are in a new systems guide, [SYSTEMS.md](SYSTEMS.md).

### slotPlusUltra: TurboGrafx-16

- TurboGrafx-16 HuCard games (`.pce`) get their own shelf, for the TurboExpress. Put them in `Games/TG16/`. They run on the Beetle PCE Fast core with the two-button pad. No BIOS is needed. CD and SuperGrafx games are not supported.
- HuCards are drawn as the real card: white, with gold contacts across the top over a dark textured panel, a square label in the lower half (art goes in `Labels/TG16/`, 191 px wide and up to 163 tall; it keeps its own shape, spans the card's width and sits at the bottom, so a shorter label leaves plain card above it, as on real HuCards) and a ridged grip tab at the foot. A HuCard turns end over end before it goes into the slot, so it goes in contacts first, and turns back upright when it comes out.
- The TurboGrafx-16 picture is shown at 2x and follows the width each game draws, from 256 up to the 512-wide mode. L1 stretches it to fill the screen and R1 gives it back. The Screen menu has a TurboGrafx-16 page with its own Shader (Off, LCD3x, Grid or Dot, Grid by default) and Bezel: four bundled, TG16 (a dark face with the TurboGrafx-16 logo down the left side) and PCE (the PC Engine's white case either side of the screen), each with a Wide version for games that draw 320 wide, or your own PNGs in `Bezels/TG16/`. A bundled bezel can now have a name of its own in the Bezel row, rather than Default.
- The buttons follow the TurboExpress: the left face button is II, the right one I, Start is Run and Select is Select.
- The `TG16` folders under `Games`, `Labels`, `Saves`, `States` and `Bezels` are created for you on first boot.

### slotPlusUltra: NeoGeo Pocket

- NeoGeo Pocket and NeoGeo Pocket Color games (`.ngp`, `.ngc`, `.ngpc`) get one shared shelf. Put them in `Games/NGP/`. They run on the Beetle NeoPop core, in English where a game has both languages. No BIOS is needed.
- NeoGeo Pocket carts are drawn in 1.7.0's look: a ridged cap, a wide label in a recessed frame (art goes in `Labels/NGP/`, about 246x133) and a lower band with a down arrow and SNK and NGP plaques pressed into it, their letters raised, with a dark grey default shell.
- The NeoGeo Pocket picture is shown actual size at 3x. L1 stretches it to fill the screen and R1 gives it back. The Screen menu has a NeoGeo Pocket page with its own Shader (Off, LCD3x, Grid or Simpletex) and Bezel: a bundled Default with the NeoGeo Pocket name and logo down the left side, or your own PNGs in `Bezels/NGP/`.
- NeoGeo Pocket saves go to `Saves/NGP/`. The core writes them when the game stops, so slot now stops it before powering off or restarting.
- The buttons follow the NeoGeo Pocket's layout: the left face button is its A and the right one its B. Start is Option.
- The `NGP` folders under `Games`, `Labels`, `Saves`, `States` and `Bezels` are created for you on first boot.

### slotPlusUltra: Game Gear

- Sega Game Gear games (`.gg`) get their own shelf. Put them in `Games/GG/`. They run on the Gearsystem core. No BIOS is needed.
- Game Gear carts are drawn in 1.7.0's look: grey moulded plastic tinted by the shell colour, with the raised hood and grip ridges over a squarish label (art goes in `Labels/GG/`, about 290x200), and a black default shell.
- The Game Gear picture is shown actual size at 3x. L1 stretches it to fill the screen and R1 gives it back.
- The Screen menu has a Game Gear page with its own Shader (Off, LCD3x, Grid or Simpletex) and Bezel. For the bezel there is a bundled Default, a black face with the Game Gear's green power light glowing above the screen, as well as your own PNGs in `Bezels/GG/` and the SP frame on 4:3 screens. SELECT + B / A in a game changes the Game Gear shader too.
- SELECT does nothing in a Game Gear game. Gearsystem would take it as a console reset, which the Game Gear does not have.
- The `GG` folders under `Games`, `Labels`, `Saves`, `States` and `Bezels` are created for you on first boot.

### slotPlusUltra: everything else

- The 1.7.0-plus1.1 menu freeze fix covers the Game Gear and NeoGeo Pocket too.
- On 4:3 handhelds such as the RG 35XXSP, a stretched game now fills the whole screen instead of its 3:2 window above the SP frame. This is every system but the Game Boy and Game Boy Color, which keep 1.7.0's stretch.
- The Bezel rows on every added system's Screen page now show which bezel is picked.

## [1.7.0-plus1.2](https://github.com/nelynes/slotPlus/releases/tag/v1.7.0-plus1.2)

- WonderSwan carts default to the shell the game came in: black for a WonderSwan game (`.ws`), frosted clear with the board showing for a WonderSwan Color game (`.wsc`). Before, every WonderSwan cart was clear. A shell set in `Config/cart_shell.txt` still wins.

## [1.7.0-plus1.1](https://github.com/nelynes/slotPlus/releases/tag/v1.7.0-plus1.1)

- Fixed: opening the menu froze the handheld after playing a WonderSwan or Atari Lynx game, until it was switched off.

## [1.7.0-plus1](https://github.com/nelynes/slotPlus/releases/tag/v1.7.0-plus1)

slotPlus: Slot 1.7.0 plus WonderSwan and Atari Lynx.

- Brings in everything from Slot 1.7.0, below.
- WonderSwan and Lynx carts are redrawn in 1.7.0's look: grey moulded plastic tinted by the shell colour, with light and dark themes. The WonderSwan has its oval nameplate and the bare board below its shoulder; the Lynx is the curved-lip card with its contacts behind a clear strip. Their label art sizes change a little: about 324x132 for WonderSwan and 284x190 for Lynx.
- The Screen menu has a WonderSwan page and an Atari Lynx page, each with its own Shader and Bezel. A bezel for either is your own PNG in `Bezels/WS/` or `Bezels/Lynx/`, or the SP frame on 4:3 screens.
- The Transparent shader is on both the Game Boy and Game Boy Color pages.
- WonderSwan gets LCD3x, which 1.7.0 offers for GB and GBC.
- A new Transparent screen shader for Game Boy and Game Boy Color games. It is LCD3x with the screen left unlit: light pixels let a warm, grainy backing show through instead of glaring white, and dark pixels cast soft drop shadows onto it. Pick it from the Shader row on the Game Boy or Game Boy Color page of the Screen menu, or with SELECT + B / A in a game. The idea comes from Matt Akins' Pixel Transparency shader for RetroArch; slot's version is written from scratch.

## [1.7.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.7.0)

- The slot, carts and shelf now have a skeuomorphic effect, with light and dark themes and bundled wallpapers. Huge thanks to Markus for this amazing work! (#74, @MarkusMaribu)
- An in-game menu: tap MENU to pause the game behind it. A double tap still goes straight to States.
- Favorites: Y stars a cart onto its own shelf. (#89, @diluno)
- A Recents shelf of the last 10 games, switched on from Settings -> General.
- slot remembers the shelf and cart at power off and boots back onto them. (#53)
- Per-platform bezels. Markus again! (#77, @MarkusMaribu)
- On 4:3 panels the shelf fills the screen. (#81, @diluno)
- Game Titles on the General page fades the shelf name into the selected game's name. (#84)
- A 1 frame run-ahead setting for GBA games on mGBA. (#56, @454135431515)
- Wallpaper Tint darkens your own wallpaper on the shelf.
- GBA, GB and GBC each get their own Shader and Color Correction rows. (#83, @paulocoe-creator)
- LCD3x is offered for GB and GBC games. (#88, @paulocoe-creator)
- Game Boy stretch now properly only widens the picture.
- The backlight steps follow a curve, so the low-end goes dimmer. (#54, @flo333)
- Holding a direction repeats it in menus, the clock and the shelf letter jump, but never in play.
- RG34XX uses its own chords for brightness and the menu display. (#85, @paulocoe-creator)
- The RG28XX picture is turned the way BaseOS says.
- Restart no longer hangs on a black screen: the WiFi driver is unloaded first.
- Muted play keeps its frame rate. (#51, @flo333)
- Fast-forward no longer stutters on frames that take longer than average. (#70)
- gpSP no longer crashes when a crowded boot left its JIT cache unmapped.
- Core options follow the platform that is playing. (#82)
- Card logs moved into a `Logs` folder, and the session log rotates. Card alarms are logged too.

## [1.6.0-lyx2](https://github.com/nelynes/slotPlus/releases/tag/v1.6.0-lyx2)

- WonderSwan and Atari Lynx games get their own screen shaders. The Screen menu has a WS / WSC Shader row (Off, Grid or Simpletex) and a Lynx Shader row (Off, LCD3x, Grid or Dot), and SELECT + B / A in a game changes that system's shader alone. Before, the WonderSwan shared the Game Boy's shader and the Lynx shared the GBA's.
- Each starts on the shader it was sharing, so nothing changes until you pick a different one.

## [1.6.0-lyx](https://github.com/nelynes/slotPlus/releases/tag/v1.6.0-lyx)

slotPlus: Slot 1.6.0 plus WonderSwan and Atari Lynx.

- Atari Lynx games (`.lnx`, `.lyx`) get their own shelf. Put them in `Games/Lynx/`. They run on the Beetle Lynx core.
- Atari Lynx games need `lynxboot.img`, which is not included. Put your own copy in `BIOS/`.
- Lynx carts have their own shape, a label (art goes in `Labels/Lynx/`) and a dark default shell.
- Games made to be held on their side are played with the handheld turned 90 degrees counter-clockwise, like the vertical WonderSwan games: the picture fills the screen and the D-pad follows it. Klax, Gauntlet III, Lexis and NFL Football are recognised by name, so any dump of them works. `Config/lynx_rotation.txt` sets it for any other game.
- SELECT in a Lynx game that turns flips it to the usual way of holding the handheld: the picture upright and tall with bars at the sides. Press it again to turn the handheld back. Each launch starts turned.
- The `Lynx` folders under `Games`, `Labels`, `Saves` and `States` are created for you on first boot.
- Lynx games run at their own refresh rate. The core calls it 75 frames a second, but each game sets its own, so slot reads the real rate from the game's audio and runs it at that.
- Lynx has no battery saves. Use save states.

## [1.6.0-ws3](https://github.com/nelynes/slotPlus/releases/tag/v1.6.0-ws3)

- WonderSwan games now run on automatic frameskip, like GBA and GB. The setting was being sent to the Cygne core under the wrong name and ignored, so it ran with frameskip off. Heavy scenes hold their speed better.

## [1.6.0-ws2](https://github.com/nelynes/slotPlus/releases/tag/v1.6.0-ws2)

slotPlus: Slot 1.6.0 plus WonderSwan.

- WonderSwan and WonderSwan Color games (`.ws`, `.wsc`) get their own shelf. Put them in `Games/WS/`. They run on the Beetle WonderSwan (Cygne) core.
- WonderSwan carts have their own wide shape with a stepped plug, a wide label (art goes in `Labels/WS/`), and a smoke-translucent default shell.
- The WonderSwan's Y pad is reachable: hold L1 and the D-pad becomes the Y pad, hold R1 and X / A / B / Y become the Y pad.
- Turbo and the picture-mode shortcuts are off for WonderSwan games so the shoulder buttons can act as modifiers. SELECT turns a vertical game upright and back.
- SELECT only rotates vertical games. Horizontal WonderSwan games ignore it, so they can't be turned onto a layout their buttons don't follow.
- Vertical games get their own buttons: the D-pad is the Y pad, the face buttons are the X pad, L1 is A and R1 is B. The mapping follows whether the picture is upright (SELECT) or the device is turned counter-clockwise.
- Vertical games fit the screen: the whole picture shows at 2x with black bars at the sides. GBA, GB and GBC are unchanged.
- The `WS` folders under `Games`, `Labels`, `Saves` and `States` are created for you on first boot.
- The core picker works again on GBA games.

## [1.6.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.6.0)

- Runs on the RG28XX: its panel is mounted portrait, so slot turns the picture onto it. (#35, @GamerAnim)
- Pokémon Pinball: Ruby & Sapphire rumble. mGBA now sees the game's Game Boy Player check. You still need to turn rumble on in game settings. (#25)
- A red alarm shows when the SD card turns read-only or a save fails to write.
- A new shortcut for mute: SELECT + VOL+ / VOL-. The 34XX has a fake volume wheel so VOL+ + VOL- isn't possible. (#39)
- Game Boy cartridge clocks are kept in the .sav. (#22)
- The GBA cartridge clock keeps running on gpSP while the game is off.
- Large libraries now should boot faster, and the first frame shows sooner.
- Games start faster: resting on a cart reads its ROM ahead, so big ROMs no longer wait on the SD card when you insert them.
- Wallpapers are dealt from a shuffled deck, so none repeats until all have shown.
- PNG art is turned the way its EXIF orientation tag says.
- When slot crashes it logs where, and keeps that run's log in `crash/` to attach to a bug report.
- The BaseOS startup sound no longer leaves slot silent. (#20)
- A user defined `Config/bootlogo.bmp` will take precedence over the shipped one. (#30)
- Cart shell, core and video mode files are now `.txt` instead of `.ini`, so Chrome on Windows opens them. slot renames them on first boot. (#15)
- The grid shader's lines stay even on the RG34XX and at fractional scales, and the dot shader no longer darkens a white screen as much. (#26, #36)

## [1.5.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.5.0)

- Rumble regression fixed.
- The boot logo is built to the size of the one BaseOS installed, so it fits every panel instead of only 720x480.
- Runs on the RG35XXSP: the picture fills the 4:3 screen's width and a GBA SP bezel sits underneath while a game plays.
- Screen shaders chosen per platform: Off, LCD3x, Grid or Dot for GBA, and Off, Grid or Simpletex for Game Boy and Game Boy Color.
- SELECT + B / A in game steps back and forth through the screen shaders. A game's own soft reset that holds SELECT with A or B no longer reaches it; hold A on the carousel to start a game fresh instead.
- New settings: auto save on eject, turbo buttons and rewind can each be switched off.
- Settings are grouped into Screen and Gameplay pages, and every value wraps from the last choice to the first.
- Label art is cached on the card, so the shelf no longer stutters building cart faces. New labels are cached once at boot behind a Caching New Labels screen.
- Named palettes for Game Boy games: turn on GB Palettes in the Screen settings, then SELECT + L2 / R2 in game steps back and forth through mGBA's 48 palettes.
- The speaker no longer buzzes while nothing is playing. slot lets go of the audio device after 3 seconds of silence and takes it back when sound starts. Thanks to flo333 for finding it.

## [1.4.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.4.0)

- The volume bar shows a headphones icon while they are plugged in.
- Speaker and headphones keep their own volume and mute, and switch the moment you plug / unplug.
- Settings moved from `System/` to a new `Config/` folder, so replacing `System/` on update no longer resets them. slot moves them on first boot.
- The quick menu and power menu wrap from the last item to the first, and back.

## [1.3.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.3.0)

- The SP boots to a slot logo instead of BaseOS's. slot installs `System/bootlogo.bmp` on first run and keeps the original as `bootlogo.baseos.bmp`.

## [1.2.2](https://github.com/BrandonKowalski/slot/releases/tag/v1.2.2)

- Fix for hitch in games caused by autosave. The autosave now writes to the card in the background.

## [1.2.1](https://github.com/BrandonKowalski/slot/releases/tag/v1.2.1)

- Shell colors from Cart Studio now apply to games with accented names.
- Clear GBA carts no longer show their circuit board above the label.

## [1.2.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.2.0)

- Hold X for turbo A and Y for turbo B.
- Carts on the shelf are larger, and shrink into the slot as they go in.
- Carts are drawn at their real proportions, so full label scans fit without cropping.
- Jumping a letter with Up or Down shows the letter in the slot.
- Saves now stick for games whose save is smaller than the core first reports, such as Golden Sun on mGBA.
- The seated cart no longer shows through the slot as a game starts.
- SELECT reaches the game as soon as it can no longer be the start of a shortcut.

## [1.1.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.1.0)

- Fast-forward runs several game frames per refresh, so it is properly fast, and rewind no longer records while fast forwarding.
- The About label credits BaseOS.

## [1.0.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.0.0)

- Game Boy and Game Boy Color, each on its own shelf. L1 and R1 switch shelves, and carts are drawn at their own size in their own plastics.
- Two cores, mGBA and gpSP. Press START on a cart to open it and swap the chip.
- Link play between two RG SPs over WiFi: the link cable and the Wireless Adapter for Pokémon trades and Advance Wars, mGBA's link mode for Mario Kart, and Game Boy link for Tetris.
- A quick menu on the shelf for fast forward speed and sound, rumble, color correction, the date and time, and About.
- Game Boy games stretch to fill the screen with L1 and go back with R1.
- The real BIOS boot animation plays when the card has one.
- Per-cart shell colors, set in `cart_shell.ini`.
- Up and Down jump through the shelf a letter at a time.
- Your own folder layout on the card is kept, and slot runs on upstream BaseOS.

## [0.1.1](https://github.com/BrandonKowalski/slot/releases/tag/v0.1.1)

- The game holds still behind the power menu.
- Rewind records at every speed and replays exactly what happened.

## [0.1.0](https://github.com/BrandonKowalski/slot/releases/tag/v0.1.0)

- First release: a GBA-only shelf of cartridges with an insert scrape and clunk.
- The game in the slot resumes on the next boot. Closing the lid sleeps and saves.
- Save states with a switcher, rewind, fast-forward, brightness, blue light and volume.
- A power menu on a held POWER, and an About label.
