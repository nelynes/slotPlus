# slotPlus settings guide

Where everything is in Settings from slotPlus 3.1, what moved from 3.0, and how the two editions, **slotPlus Core** and **slotPlus Ultra**, differ.

MENU opens Settings on the shelf, and the in-game menu in a game, as in Slot. A line just above the slot says what the chosen row does.

- [Which edition is for you?](#which-edition-is-for-you)
- [Where things moved in 3.1](#where-things-moved-in-31)
- [Every page in Ultra](#every-page-in-ultra)
- [What Core keeps, simplifies and leaves out](#what-core-keeps-simplifies-and-leaves-out)

---

## Which edition is for you?

Both editions play every system slotPlus has: Game Boy Advance, Game Boy and Game Boy Color, WonderSwan and WonderSwan Color, Atari Lynx, Pokémon mini, Game Gear, NeoGeo Pocket and Pocket Color, TurboGrafx-16 and the Nintendo DS, plus the PokeDaisy shelf. A system only gets a shelf when it has games on the card. Both have the carts, Shelf Life, play time, HDMI, the cart slots and everything else you see on the shelf. They differ in how much there is to set up.

| | **slotPlus Core** | **slotPlus Ultra** |
|---|---|---|
| **For** | Putting games on the card and playing. A gift, a child's handheld, or anyone who'd rather not open a menu. | Tinkerers: everyone who wants every option. |
| **Setup** | Each system comes set up for you. Most rows are a plain On or Off. | Every option, set the way Slot sets it. |
| **RetroAchievements** | No | Yes |
| **RomM and Gamebit** | No | Yes |
| **Save states, screenshots, guides in a game** | The save, load and screenshot shortcuts only | The full in-game menu, with undo |
| **Picture** | One Shader row per system: Clean or LCD | Shaders, overlays, frame blending, bezels, scale, stretch, blue light, saturation and contrast |
| **Download** | `slot-core.zip` | `slot-plusultra.zip` (also published as `slot-plus.zip`) |

**Not sure?** Ultra is what slotPlus has always been, and what you get if you're updating from 3.0. Core is for handing someone a handheld that just plays.

**Switching** works over Wi-Fi: in Settings → Online → Check for Updates, when you're up to date, X offers the other edition and A installs it in place. Or replace the `System` folder on your card with the one from the other zip. Your games, saves, save states, labels and wallpapers stay either way. Going to Core starts you on Core's own settings, since most of Ultra's choices have no row there.

---

## Where things moved in 3.1

3.1 groups Settings by what you're doing rather than by which part of slotPlus owns a setting. The top level used to be Shelf, Gameplay, Controls, Screen, Tools, RetroAchievements and System. It's now:

**Look · Library · Play · Screen · Controls · Online · System**

Nothing is stored differently, so every choice you made on 3.0 carries over.

| You're looking for | On 3.0 it was in | On 3.1 it's in |
|---|---|---|
| Theme, Wallpaper, Wallpaper Tint | Shelf | **Look** |
| Cart Slot, Game Titles | Shelf | **Look** |
| Shelf Life (shrink wrap, dust, wear) | Shelf | **Look → Shelf Life** |
| Shelf Mode, Recents | Shelf | **Library** |
| Gamebit, RomM, Fetch Missing Art, Fetch Box Art | Tools | **Library** |
| Kid Mode, Kid Shelf, Kid Play Time | Shelf | **System → Kid Mode**, as Mode, Kid Shelf and Play Time |
| Fast Forward, Rewind, Rumble, Left Stick as D-Pad | Gameplay | **Play** |
| Fast Forward Sound | Gameplay, its own row | **Play → Fast Forward**: the sound is part of the speed (Off, 1.5×, 1.5× Sound, 2×, 2× Sound, up to 6× Sound) |
| Resume on Insert, Resume on Power On | Gameplay, two rows | **Play → Resume**, one row: Off, Insert, Power On or Both |
| Wi-Fi | System | **Online** |
| RetroAchievements | Its own top-level page | **Online → RetroAchievements** |
| Check for Updates | System | **Online** |
| Date & Time, Nickname, Lid Power Off, Battery Display, About | System | **System** |
| Shaders, bezels and the rest of the picture | Screen | **Screen**, which now only shows a page for systems with games on the card |
| Button mapping and shortcuts | Controls | **Controls**, unchanged |

The in-game menu (MENU in a game) hasn't changed, except that it has the same line of help.

---

## Every page in Ultra

### Look

How the shelf looks.

- **Theme**
- **Wallpaper**, and **Wallpaper Tint** once a picture is the wallpaper. With Wallpaper set to Shelf, a picture named after a shelf (`ws.png`, `lynx.png`, `pokemini.png`, `pokedaisy.png`, `gg.png`, `ngp.png`, `tg16.png`, `nds.png`) shows on that shelf.
- **Cart Slot:** Slot's own, or Bespoke, where each shelf's slot is the back of that system's handheld.
- **Game Titles**
- **Shelf Life:** Shrink Wrap, Dust and Wear (each On or Off), Clean Button Resets (what holding B on a cart takes off: Wear, Dust, Both or Disabled), and Clean All Carts.

### Library

How games are arranged on the shelf, and how they and their art reach the card.

- **Shelf Mode** and **Recents**
- **Fetch Missing Art** (labels), and **Fetch Box Art** for the DS
- **Gamebit:** manage the card from a browser
- **RomM:** pair with a RomM server

Gamebit, RomM and the fetch rows need Wi-Fi, so a handheld without it doesn't show them.

### Play

How games play.

- **Fast Forward:** Off, 1.5×, 2×, 3×, 4× or 6×, each with or without sound
- **Rewind**
- **Rumble**
- **Left Stick as D-Pad**
- **Resume:** when a game picks up where you left it: Off, when a cart goes in (Insert), when the handheld starts (Power On), or Both

### Screen

- **General:** Brightness, Blue Light, Saturation and Contrast
- A page for each system with games on the card:
  - **Game Boy Advance:** Shader, Overlay, Color Correction, Frame Blending, Bezel, Scale
  - **Game Boy Color:** the same, plus Stretch
  - **Game Boy:** the same as Game Boy Color, plus Palette
  - **WonderSwan, Atari Lynx, Game Gear, NeoGeo Pocket, TurboGrafx-16, Nintendo DS:** Shader and Bezel
  - **Pokémon mini:** Shader, Bezel and Full Screen

In a game, SELECT + B / A steps through that system's shaders. The [systems guide](SYSTEMS.md) has what each system's page offers.

### Controls

- **Shelf** and **Collections:** the buttons on the shelf
- **Shortcuts:** the in-game shortcuts (save and load a state, reset, rewind, fast forward, screenshot, guide, brightness, blue light, colour correction, palette, shader and the sunlight sensor)
- **In-Game:** where each of the game's buttons sits, for every system, including Turbo A and Turbo B

### Online

Everything that needs the network.

- **Wi-Fi:** on or off, and the network to join
- **RetroAchievements:** Username, Password, Cheevo Mode, Encore, and Download Achievement Sets (X there refreshes them all). See [the RetroAchievements notes](crates/slot-achievements/README.md).
- **Check for Updates:** updates to the newest slotPlus, and offers the other edition

### System

The handheld itself.

- **Kid Mode:** Mode, Kid Shelf (which shelf Kid Mode shows) and Play Time (a daily limit, from 15 minutes to 2 hours)
- **Date & Time**
- **Nickname:** your name for link play, the back of a HuCard and the DS
- **Lid Power Off**
- **Battery Display**
- **About**

### In a game (MENU)

Resume, Undo Save, Undo Load, Undo Reset, States, Screenshots, Guide, Settings (Screen, Gameplay and Controls for the game you're playing) and Exit Game.

### The blue dots

A small blue dot marks every row that's new in this version, and every row on the way to one. In 3.1 they mark the Online and Kid Mode pages and the new Fast Forward and Resume rows.

---

## What Core keeps, simplifies and leaves out

Core is Ultra with the setting-up done for you. Its menus follow the same Look, Library, Play, Screen, Controls, Online and System layout, with fewer rows.

**Set for you.** These have no row in Core, and are set the way most players end up with them:

- Dark theme, game titles shown, and the Bespoke cart slot on every shelf, with Core's own wallpaper on each shelf
- Fast forward at 4×, without sound
- Resume on both a cart going in and the handheld starting up
- Game Boy and Game Boy Color filling the screen; Pokémon mini at Full Screen
- Bezels off
- Battery shown as an icon, and the handheld switching off a minute after the lid is shut
- Rumble on, and the left stick working as the D-pad
- The solar sensor (Boktai) at full sun

**Made simpler:**

- **Shelf Life** is one switch for shrink wrap, dust and wear, starting On. Holding B on a cart still cleans it.
- **Kid Mode** is Off or One Shelf.
- **Screen** has one Shader row for each system with games on the card: **Clean** (sharp pixels) or **LCD** (the look of the handheld's own screen, with Game Boy and Game Boy Color pixels you can see through), starting on LCD. Color Correction stays an On/Off row for the Game Boy Advance and Game Boy Color, and Palette stays on the Game Boy page.

**Kept as in Ultra:** Fetch Missing Art, the In-Game button mapping, the save, load and screenshot shortcuts, Wi-Fi, Check for Updates, Date & Time, Nickname and About.

**Left out:** RetroAchievements, RomM, Gamebit, Recents and Collections, the in-game States, Screenshots, Guide and Undo rows, Wallpaper Tint, Clean All Carts, Blue Light, Saturation, Contrast, Overlays, Frame Blending, Bezels, the GBA's Scale, and the shortcuts for reset, fast-forward toggle, guide, blue light, colour correction, shader and sunlight.
