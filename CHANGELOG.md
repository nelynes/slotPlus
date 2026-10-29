# Changelog

## [1.11.0-plus2.4](https://github.com/nelynes/slotPlus/releases/tag/v1.11.0-plus2.4)

slotPlus brought up to Slot 1.11.0: a 1.5x fast forward speed, a Fast Forward Toggle shortcut, no more buzz in headphones after the sound stops, and the CPU left at its full fixed speed again. See [Slot's 1.11.0 notes](https://github.com/BrandonKowalski/slot/releases/tag/v1.11.0).

To upgrade, replace the `System` folder on your card with the one from the zip.

- Fast Forward Toggle is handy in slotPlusUltra's DS games, where R2, the Fast Forward shortcut's own button, is the stylus: bind the toggle to a button combo on the Shortcuts page.

## [1.10.0-plus2.4](https://github.com/nelynes/slotPlus/releases/tag/v1.10.0-plus2.4)

To upgrade, replace the `System` folder on your card with the one from the zip.

- **Nintendo DS, in slotPlusUltra.** Put `.nds` games in `Games/NDS/`. They play on DSperate, at full speed on the RG SP, and no BIOS is needed.
- **Screen layouts:** L2 swaps the main screen, and SELECT + L2 steps through Both Screens, One Screen, Full Screen, Side by Side and Picture in Picture. Each game remembers its own.
- **A stylus:** hold R2 and the D-pad moves a cursor over the bottom screen, and A touches.
- **The DS game card** has its own cart on the shelf, and with Cart Slot set to Bespoke the slot is the back of a silver DS Lite.
- **DS Lite bezels** for every layout, bundled and on by default. Your own can be one PNG, or a folder with a PNG for each layout.
- In heavy 3D scenes, DS games skip drawing the odd frame rather than slow down, and the CPU runs at full clock while one plays. DS games have no rewind, since their states are about 6 MB each.

See the [systems guide](https://github.com/nelynes/slotPlus/blob/main/SYSTEMS.md#nintendo-ds) for the controls, layouts and label art size.

## [1.10.0-plus2.3](https://github.com/nelynes/slotPlus/releases/tag/v1.10.0-plus2.3)

slotPlus brought up to Slot 1.10.0: screenshots, Scale and Frame Blending on each Screen page, a Game Boy Stretch setting, a clock set by city with daylight saving, and a Game Boy Advance Video shelf (`Games/GBAV`). See [Slot's 1.10.0 notes](https://github.com/BrandonKowalski/slot/releases/tag/v1.10.0).

To upgrade, replace the `System` folder on your card with the one from the zip.

On slotPlus's own systems:

- The WonderSwan and Lynx (and in slotPlusUltra, the Game Gear, NeoGeo Pocket and TurboGrafx-16) keep their picture as before: Scale, Stretch and Frame Blending are the Game Boy family's, and are left out of those systems' in-game menus.
- A new default wallpaper, with new stickers, in slotPlus and in slotPlusUltra.

## [1.9.0-plus2.3](https://github.com/nelynes/slotPlus/releases/tag/v1.9.0-plus2.3)

To upgrade, replace the `System` folder on your card with the one from the zip.

- Cart Slot, a new row on the General page of Settings: set it to Bespoke and the slot at the bottom of each shelf becomes the back of that system's handheld, where its carts go in.

## [1.9.0-plus2.2](https://github.com/nelynes/slotPlus/releases/tag/v1.9.0-plus2.2)

slotPlus brought up to Slot 1.9.0, with Slot 1.8.1 on the way. Both versions get everything below.

To upgrade, replace the `System` folder on your card with the one from the zip.

- Saves, save states, settings and the log reach the card as soon as they are written, and a game's own save is written every minute while it plays, so a reset or a dead battery loses far less.
- Undo Save and Undo Load: for 30 seconds after a save or a load, the in-game menu offers to undo it, under Resume. X on the state switcher does the same.
- Load State on Insert, a new row on the Gameplay page, decides whether putting a cart in loads its state. Auto Save on Eject is now Save State on Eject.
- A core that stops answering no longer freezes the handheld, and a game that crashes while loading starts you on the shelf next time instead of loading it again.
- GBA bezels on 640x480 handhelds: put .png files in `Bezels/GBA` and pick one under Settings > Screen > Game Boy Advance > Bezel.
- Game Boy games on an MBC2 cartridge, such as Kirby's Pinball Land, no longer freeze after the boot screen.
- Fast Forward can be turned off, save states are smaller, and holding Up or Down on the shelf to jump a letter at a time no longer empties it.
- The shelves follow Slot's new order: Game Boy Advance, Game Boy Color, Game Boy, then WonderSwan and Atari Lynx (and the Game Gear, NeoGeo Pocket and TurboGrafx-16 in slotPlusUltra).
- Card repair no longer hangs on a badly damaged card, and the BaseOS startup sound plays again.

## [1.11.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.11.0)

- Headphones no longer buzz a few seconds after the sound stops. slot keeps the audio device open while headphones are in. ([#163](https://github.com/BrandonKowalski/slot/issues/163))
- slot leaves the CPU at its full fixed speed again, as it did in 1.9.0. `schedutil` will have to be revisited. ([#162](https://github.com/BrandonKowalski/slot/issues/162))
- Fast Forward Toggle is a new shortcut. One press turns fast-forward on and the next turns it off. It's unbound by default. ([#164](https://github.com/BrandonKowalski/slot/issues/164))
- Fast Forward has a 1.5x speed, for games where 2x is too fast. ([#161](https://github.com/BrandonKowalski/slot/issues/161))

## [1.10.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.10.0)

- Screenshots: Press SELECT + X while you play. Once you take one, you can pick Screenshots in the in-game menu to view them. They're saved as `.png` in `Screenshots/`. ([#94](https://github.com/BrandonKowalski/slot/issues/94))
- Scale, a new row on each system's Screen page, sets how big the picture is. ([#121](https://github.com/BrandonKowalski/slot/issues/121))
- Stretch, on the Game Boy and Game Boy Color Screen pages, sets how far L1 stretches: Classic, matching GBA SP behavior of only widening, or Full Screen.
- The Default Game Boy and Game Boy Color bezels have additional versions that fit the Integer and Fit modes.
- Frame Blending can be toggled on each system's Screen page. This smooths games that flicker sprites every other frame. Mix blends each frame with the last; LCD Ghosting, on mGBA only, leaves a fading trail like the original screens. ([#66](https://github.com/BrandonKowalski/slot/issues/66))
- Date & Time takes a city instead of an offset from UTC, and moves the clock for daylight saving on its own. A new Clock row allows you to pick 24-hour or 12-hour time.
- Game Boy Advance Video carts get their own shelf when placed in `Games/GBAV`. Game Boy Player support is disabled for this shelf. ([#140](https://github.com/BrandonKowalski/slot/issues/140))
- Removed the last brightness step that turned the backlight completely off.
- Powering off or restarting closes the audio device first. ([#142](https://github.com/BrandonKowalski/slot/issues/142))
- The jumped-to letter on the shelf is drawn at the title size. ([#151](https://github.com/BrandonKowalski/slot/issues/151))

## [1.9.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.9.0)

- mGBA no longer crashes on an empty resume state, which a card repair can leave behind. Booting with that cart in the slot crashed slot every time.
- Saves, states, settings and slot's log reach the card as soon as they are written, not up to 30 seconds later. That shrinks the window in which pressing reset or the battery dying can leave the card read-only. ([#72](https://github.com/BrandonKowalski/slot/issues/72), [#144](https://github.com/BrandonKowalski/slot/issues/144))
- A game's own save is written to the card every minute while it plays. The resume state is written every 5 minutes instead of every minute.
- If a game takes slot down while it loads, slot starts on the shelf next time instead of loading it again.
- A core that stops answering no longer freezes slot. Saving waits at most 2 seconds for it, and ejecting at most 10 before slot restarts.
- Caching New Labels no longer stays up forever on a label it cannot read. After 20 seconds without progress the shelf comes up, and a label that crashes its decoder is skipped.
- GBA can use your own bezels on 640x480 devices. Add .png files to `Bezels/GBA` and pick one in Settings > Screen > Game Boy Advance > Bezel. On 720x480 devices a GBA game fills the screen, so there is no bezel.
- A card repair no longer hangs on a card too damaged to check. Each step is given up on after a time limit, the repair reports that it failed, and the device restarts.
- For 30 seconds after a save or a load, the in-game menu now offers Undo Save or Undo Load under Resume. This option is also accessible with X on the state switcher screen.

## [1.8.1](https://github.com/BrandonKowalski/slot/releases/tag/v1.8.1)

- Game Boy games on an MBC2 cartridge, such as Kirby's Pinball Land, no longer freeze after the boot screen. mGBA crashed on the first read of their save RAM; slot now builds it with upstream's fix. ([#137](https://github.com/BrandonKowalski/slot/issues/137))
- Load State on Insert, a new row on the Gameplay page, decides whether putting a cart in, or booting with one in the slot, loads its state. Auto Save on Eject is renamed Save State on Eject and only decides whether ejecting saves one.
- Holding Up or Down to jump through the shelf a letter at a time no longer empties the shelf.
- The BaseOS startup sound plays again. slot no longer takes the speaker while it boots, only once it has something to play. ([#135](https://github.com/BrandonKowalski/slot/issues/135))
- Save states and rewind are smaller: slot no longer keeps the extra space mGBA sets aside before a game finds its save chip. Metroid Fusion's states drop from 528 KB to 430 KB. ([#126](https://github.com/BrandonKowalski/slot/issues/126))
- Fast Forward can be set to Off, so holding it does nothing. ([#123](https://github.com/BrandonKowalski/slot/issues/123))

## [1.8.0-plus2.2](https://github.com/nelynes/slotPlus/releases/tag/v1.8.0-plus2.2)

slotPlus 2.2 adds a rotation picker for Atari Lynx games. Both versions get it.

To upgrade, replace the `System` folder on your card with the one from the zip.

- Atari Lynx: START on a Lynx cart turns it over to choose how the handheld is held for that game: Upright, 90CCW, 90CW or 180. The game starts upright for that hold, with the D-pad turned to match, and the choice is kept in `Config/lynx_hold.txt`.
- Atari Lynx: a game made to be held turned clockwise, such as Klax, now plays that way at full size, instead of upside down with the handheld turned the other way. SELECT switches between the chosen rotation and the picture upright.
- The card's `Labels/README.txt` and `Bezels/README.txt` cover slotPlus's systems and point to Cart Studio+.

## [1.8.0-plus2.1](https://github.com/nelynes/slotPlus/releases/tag/v1.8.0-plus2.1)

slotPlus 2.1 brings slotPlus up to Slot 1.8.0. It comes in two versions, both built from this release:

- **`slot-plus.zip`, slotPlus:** Game Boy Advance, Game Boy and Game Boy Color, plus WonderSwan and Atari Lynx.
- **`slot-plusultra.zip`, slotPlusUltra:** everything in slotPlus, plus Game Gear, NeoGeo Pocket and TurboGrafx-16.

To upgrade, replace the `System` folder on your card with the one from the zip.

### Both versions

- Everything from Slot 1.8.0, below: remappable controls, Brightness and Blue Light rows, one Palette row for the Game Boy, SD card repair and safer saving. Run-Ahead is gone from slotPlus too.
- HDMI: pulling the cable on an RG SP no longer brings the picture back squashed on the handheld's screen. Slot now takes the screen's size from the display driver, and a handheld already stuck squashed is put right the next time Slot starts.
- The In-Game controls page covers the WonderSwan and Atari Lynx. Both keep the L and R rows, since L1 and R1 are the Lynx's Option 1 and Option 2 and the WonderSwan's Y pad. In a WonderSwan game the Turbo A and Turbo B rows are called X Button and Y Button, since those buttons also work the Y pad with R1 held and the X pad in vertical games.

### slotPlusUltra

- The Game Gear, NeoGeo Pocket and TurboGrafx-16 get the In-Game controls page without L and R rows, since L1 and R1 stretch the picture there.
- The SD card repair stops the NeoGeo Pocket core first, as powering off does, so its save is written if the card still takes it.
- The Game Gear, NeoGeo Pocket and TurboGrafx-16 bezels are fitted to the screen's own size, as Slot 1.8.0 does for the Game Boy's.
- The Screen settings page scrolls, since General and the eight consoles are more rows than fit.

## [1.8.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.8.0)

- Controls are remappable. Split between three categories: In-Game buttons, Shortcuts, Shelf keys. All three are global. ([#23](https://github.com/BrandonKowalski/slot/issues/23), [#92](https://github.com/BrandonKowalski/slot/issues/92))
- The in-game menu and the shelf menu share the carousel footer, with button art legends that follow the light and dark themes. ([#102](https://github.com/BrandonKowalski/slot/issues/102), @MarkusMaribu)
- Game audio plays at 48 kHz through a filtered resampler, so high frequencies no longer fold back as grit. mGBA benefits most.
- When the SD card turns read-only, SELECT + START repairs it: slot steps aside, checks FAT32 or exFAT, and restarts. ([#72](https://github.com/BrandonKowalski/slot/issues/72), [#111](https://github.com/BrandonKowalski/slot/issues/111))
- Power off, including the shutdown after the lid has been closed for 3 minutes, no longer hangs on a black screen: the Wi-Fi driver is unloaded first, as Restart already does.
- If slot stops responding with the lid closed, a separate watcher powers off a minute after the Lid Off time.
- Run-Ahead is removed. It scrambled saves in games with a 128 KB flash chip, such as Super Mario Advance 4. May revisit in the future. ([#104](https://github.com/BrandonKowalski/slot/issues/104))
- Turbo A and Turbo B have their own rows on the In-Game controls page and can be moved off X and Y. ([#127](https://github.com/BrandonKowalski/slot/issues/127))
- Shortcuts and shelf keys take any two buttons held together, such as L1+R1. In a game the first button still reaches the game at once; the second fires the shortcut. SELECT, START and MENU only work as the first button.
- Setting a control that something else already uses shows a warning naming it, such as "Same as Fast Forward", and marks both rows with a warning triangle.
- Loading a state while a game is saving to a 128 KB flash chip no longer writes the save into the wrong half of the chip.
- The cart insertion animation no longer stutters, and a large ROM still loading after the cart seats powers the screen on to a blank panel that brightens until the game appears. ([#119](https://github.com/BrandonKowalski/slot/issues/119))
- On Anbernics with a left stick, the stick works as the D-pad. Left Stick as D-Pad on the Gameplay page turns it off. ([#106](https://github.com/BrandonKowalski/slot/issues/106), [#107](https://github.com/BrandonKowalski/slot/issues/107))
- `Labels/`, `Wallpapers/` and `Bezels/` each hold a `README.txt` with the sizes and formats they take, and the site's Shelf questions cover them too. ([#122](https://github.com/BrandonKowalski/slot/issues/122))
- On 640x480 devices a 640x480 wallpaper and a 640x480 bezel now show pixel for pixel. Both used to be scaled to 720x480 first. Bezels, including the default Game Boy and Game Boy Color ones, now reach the top and bottom of the screen. ([#122](https://github.com/BrandonKowalski/slot/issues/122))
- Brightness and Blue Light have their own rows: Settings → Screen → General on the shelf, and at the end of the in-game Screen page.
- One Palette row on the Game Boy screen page replaces the Palettes switch: Auto, then mGBA's 48 palettes. Every choice changes a running game at once, including Grayscale for plain monochrome, and SELECT + L2 / R2 step through the same list. ([#124](https://github.com/BrandonKowalski/slot/issues/124))
- slot no longer writes a full memory dump to the SD card when it crashes. The last 5 crash logs are kept.
- Power off and Restart write everything to the card before unloading the Wi-Fi driver, and give the driver 5 seconds, so a driver that hangs can no longer leave unsaved data behind.
- The resume state is not rewritten when nothing changed, and files a power cut left half-written are cleaned up at boot.
- The boot partition is only opened for writing when the boot logo changes.
- The card repair waits for a charge below 20% battery, and runs fsck until the card comes back clean instead of reporting a card it fixed as failed.
- The red card alarm no longer clears when an unrelated write succeeds, and a failed save state raises it too.
- Deleting a save state asks first: Y opens the question, X deletes and B keeps it.

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

- The slot, carts and shelf now have a skeuomorphic effect, with light and dark themes and bundled wallpapers. Huge thanks to Markus for this amazing work! ([#74](https://github.com/BrandonKowalski/slot/issues/74), @MarkusMaribu)
- An in-game menu: tap MENU to pause the game behind it. A double tap still goes straight to States.
- Favorites: Y stars a cart onto its own shelf. ([#89](https://github.com/BrandonKowalski/slot/issues/89), @diluno)
- A Recents shelf of the last 10 games, switched on from Settings -> General.
- slot remembers the shelf and cart at power off and boots back onto them. ([#53](https://github.com/BrandonKowalski/slot/issues/53))
- Per-platform bezels. Markus again! ([#77](https://github.com/BrandonKowalski/slot/issues/77), @MarkusMaribu)
- On 4:3 panels the shelf fills the screen. ([#81](https://github.com/BrandonKowalski/slot/issues/81), @diluno)
- Game Titles on the General page fades the shelf name into the selected game's name. ([#84](https://github.com/BrandonKowalski/slot/issues/84))
- A 1 frame run-ahead setting for GBA games on mGBA. ([#56](https://github.com/BrandonKowalski/slot/issues/56), @454135431515)
- Wallpaper Tint darkens your own wallpaper on the shelf.
- GBA, GB and GBC each get their own Shader and Color Correction rows. ([#83](https://github.com/BrandonKowalski/slot/issues/83), @paulocoe-creator)
- LCD3x is offered for GB and GBC games. ([#88](https://github.com/BrandonKowalski/slot/issues/88), @paulocoe-creator)
- Game Boy stretch now properly only widens the picture.
- The backlight steps follow a curve, so the low-end goes dimmer. ([#54](https://github.com/BrandonKowalski/slot/issues/54), @flo333)
- Holding a direction repeats it in menus, the clock and the shelf letter jump, but never in play.
- RG34XX uses its own chords for brightness and the menu display. ([#85](https://github.com/BrandonKowalski/slot/issues/85), @paulocoe-creator)
- The RG28XX picture is turned the way BaseOS says.
- Restart no longer hangs on a black screen: the WiFi driver is unloaded first.
- Muted play keeps its frame rate. ([#51](https://github.com/BrandonKowalski/slot/issues/51), @flo333)
- Fast-forward no longer stutters on frames that take longer than average. ([#70](https://github.com/BrandonKowalski/slot/issues/70))
- gpSP no longer crashes when a crowded boot left its JIT cache unmapped.
- Core options follow the platform that is playing. ([#82](https://github.com/BrandonKowalski/slot/issues/82))
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

- Runs on the RG28XX: its panel is mounted portrait, so slot turns the picture onto it. ([#35](https://github.com/BrandonKowalski/slot/issues/35), @GamerAnim)
- Pokémon Pinball: Ruby & Sapphire rumble. mGBA now sees the game's Game Boy Player check. You still need to turn rumble on in game settings. ([#25](https://github.com/BrandonKowalski/slot/issues/25))
- A red alarm shows when the SD card turns read-only or a save fails to write.
- A new shortcut for mute: SELECT + VOL+ / VOL-. The 34XX has a fake volume wheel so VOL+ + VOL- isn't possible. ([#39](https://github.com/BrandonKowalski/slot/issues/39))
- Game Boy cartridge clocks are kept in the .sav. ([#22](https://github.com/BrandonKowalski/slot/issues/22))
- The GBA cartridge clock keeps running on gpSP while the game is off.
- Large libraries now should boot faster, and the first frame shows sooner.
- Games start faster: resting on a cart reads its ROM ahead, so big ROMs no longer wait on the SD card when you insert them.
- Wallpapers are dealt from a shuffled deck, so none repeats until all have shown.
- PNG art is turned the way its EXIF orientation tag says.
- When slot crashes it logs where, and keeps that run's log in `crash/` to attach to a bug report.
- The BaseOS startup sound no longer leaves slot silent. ([#20](https://github.com/BrandonKowalski/slot/issues/20))
- A user defined `Config/bootlogo.bmp` will take precedence over the shipped one. ([#30](https://github.com/BrandonKowalski/slot/issues/30))
- Cart shell, core and video mode files are now `.txt` instead of `.ini`, so Chrome on Windows opens them. slot renames them on first boot. ([#15](https://github.com/BrandonKowalski/slot/issues/15))
- The grid shader's lines stay even on the RG34XX and at fractional scales, and the dot shader no longer darkens a white screen as much. ([#26](https://github.com/BrandonKowalski/slot/issues/26), [#36](https://github.com/BrandonKowalski/slot/issues/36))

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
