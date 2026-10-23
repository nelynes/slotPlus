# slotPlus systems guide

Everything slotPlus adds on top of Slot's Game Boy Advance, Game Boy and Game Boy Color, one system at a time: where the games go, how big label art should be, what the Screen menu offers, how saves work, and the controls.

slotPlus comes in two versions. **slotPlus** (`slot-plus.zip`) adds the WonderSwan and Atari Lynx. **slotPlusUltra** (`slot-plusultra.zip`) adds those two plus the TurboGrafx-16, NeoGeo Pocket and Game Gear. Each system below says which version has it.

For the Game Boy systems, and everything else Slot does, see the [Slot user guide](https://slot-cfw.fyi). For playing on a TV over HDMI, see [Playing on a TV](README.md#playing-on-a-tv) in the README.

- [TurboGrafx-16 / TurboExpress](#turbografx-16--turboexpress)
- [NeoGeo Pocket / NeoGeo Pocket Color](#neogeo-pocket--neogeo-pocket-color)
- [Sega Game Gear](#sega-game-gear)
- [Atari Lynx](#atari-lynx)
- [WonderSwan / WonderSwan Color](#wonderswan--wonderswan-color)

## Things every added system shares

- **Shelf:** each system gets its own shelf. Its folders under `Games`, `Labels`, `Saves`, `States` and `Bezels` are created for you on first boot.
- **Label art:** optional PNGs go in `Labels/<folder>/<game name>.png`, named after the game's file without its extension. Each system's cart has its own label size, given below. Art of a different size is scaled to cover the label and cropped, except on HuCards, where it keeps its own shape (see the TurboGrafx-16).
- **Controls:** the In-Game controls page in Settings covers every system, so any button can be moved. The tables below show where each one starts.
- **Screen menu:** each system has its own page under Screen, with its own Shader and Bezel, beside General for Brightness and Blue Light. SELECT + B / A in a game steps through that system's shaders.
- **Bezels:** your own PNGs go in `Bezels/<folder>/`, the size of the screen with the screen area transparent: 720x480, or 640x480 on a 4:3 handheld such as the RG 35XXSP, where it shows pixel for pixel and the SP frame is offered too. A bezel shows at actual size, not when the picture is stretched.
- **Stretch:** on the TurboGrafx-16, Game Gear and NeoGeo Pocket, L1 stretches the picture to fill the screen and R1 gives back actual size. On the Lynx and WonderSwan, L1 and R1 are the game's own buttons, so a game is set to stretch with a line in `Config/video_mode.txt` (`Game Name (USA) = stretch`). Each game remembers its choice. In slotPlusUltra, on a 4:3 handheld the stretched picture fills the whole panel.
- **Cart shells:** each system's carts have a default shell colour. Set `Config/cart_shell.txt` to give a single game a different one.
- **Save states and resume** work as they do for every other system.

---

## TurboGrafx-16 / TurboExpress

The TurboExpress was NEC's handheld TurboGrafx-16: it played the same HuCards as the console.

| | |
|---|---|
| **Version** | slotPlusUltra only |
| **Games** | HuCard games as `.pce` in `Games/TG16/`. CD games and SuperGrafx games are not supported. |
| **BIOS** | Not needed. HuCards boot straight in. |
| **Core** | Beetle PCE Fast, with the two-button pad. |
| **Label art** | `Labels/TG16/`, 191 px wide and up to 163 px tall. As on real HuCards, labels come in different heights: the art is scaled to the full width at its own shape and sits at the bottom of the card, with plain card above a shorter label. Art taller than the space is shrunk to fit whole. |
| **Picture** | Most games draw 256x240, shown at 2x. Games that switch to a wider mode follow it: up to 352 wide stays at 2x, and the 512-wide mode, whose pixels are half as wide, is drawn 1x across and 2x down, centred. Stretched, the picture fills the screen whatever its width. |
| **Shaders** | Off, LCD3x, Grid, Dot. |
| **Bezel** | Four bundled: TG16, a dark face with the TurboGrafx-16 logo down the left side, and PCE, the PC Engine's white case either side of the screen, each with a Wide version for games that draw 320 wide. Or your own in `Bezels/TG16/`, 720x480 with the 512x480 window from x 104 clear (640x480 from x 40 for a wide one). |
| **Cart** | The HuCard: a thin white card with gold contacts along the top over a dark textured panel, the label printed square at the bottom of the space below the panel, and a ridged grip tab at the foot. It turns end over end before it goes into the slot, contacts first, and turns back when it comes out. |
| **Saves** | Games that save to the console's backup memory keep it in `Saves/TG16/`. |

### Controls

The buttons follow the TurboExpress's layout: II on the left, I on the right.

| Press      | Does                                              |
|------------|---------------------------------------------------|
| D-pad      | D-pad                                             |
| B / A      | II / I                                            |
| Start      | Run                                               |
| Select     | Select (Select + Run resets the game, as on the real console) |
| L1 / R1    | Stretch the picture to fill the screen / actual size |
| Hold Y / X | Turbo II / turbo I                                |

---

## NeoGeo Pocket / NeoGeo Pocket Color

| | |
|---|---|
| **Version** | slotPlusUltra only |
| **Games** | `.ngp`, `.ngc` and `.ngpc` in `Games/NGP/`. Black-and-white and colour games share one shelf. |
| **BIOS** | Not needed. The core stands in for it. |
| **Core** | Beetle NeoPop, set to English for games that have both languages. |
| **Label art** | `Labels/NGP/`, about 246x133 px (1.85:1). |
| **Picture** | The 160x152 screen at 3x, or stretched to fill the screen. |
| **Shaders** | Off, LCD3x, Grid, Simpletex. |
| **Bezel** | A bundled Default with the NeoGeo Pocket name and logo down the left side, or your own in `Bezels/NGP/`. |
| **Cart** | The NeoGeo Pocket pak: a ridged cap, the label in a recessed frame, and a lower band with SNK and NGP plaques and a down arrow. Dark grey by default. |
| **Saves** | `Saves/NGP/<game name>.flash`. |

The core writes its `.flash` save itself when the game stops. Eject the cart, power off or restart from the menu as usual, and slotPlus stops the core first so the save is written. Switching off with the power switch mid-game can lose it.

### Controls

The buttons follow the NeoGeo Pocket's own layout, so its A is the left button and its B the right one.

| Press      | Does                                              |
|------------|---------------------------------------------------|
| D-pad      | Joystick                                          |
| B / A      | A / B                                             |
| Start      | Option                                            |
| L1 / R1    | Stretch the picture to fill the screen / actual size |
| Hold Y / X | Turbo A / turbo B                                 |
| Select     | Nothing                                           |

---

## Sega Game Gear

| | |
|---|---|
| **Version** | slotPlusUltra only |
| **Games** | `.gg` in `Games/GG/`. |
| **BIOS** | Not needed. Games boot straight in. |
| **Core** | Gearsystem. |
| **Label art** | `Labels/GG/`, about 290x200 px (1.45:1). |
| **Picture** | The 160x144 screen at 3x, or stretched to fill the screen. |
| **Shaders** | Off, LCD3x, Grid, Simpletex. |
| **Bezel** | A bundled Default, a black face with the Game Gear's green power light glowing above the screen, or your own in `Bezels/GG/`. |
| **Cart** | The Game Gear pak: a raised hood with grip ridges arching over the label, and the SEGA plaque at the foot. Black by default. |
| **Saves** | Games with battery saves keep them in `Saves/GG/`. |

### Controls

| Press      | Does                                              |
|------------|---------------------------------------------------|
| D-pad      | D-pad                                             |
| B / A      | Button 1 / Button 2                               |
| Start      | Start                                             |
| L1 / R1    | Stretch the picture to fill the screen / actual size |
| Hold Y / X | Turbo button 1 / turbo button 2                   |
| Select     | Nothing (the Game Gear has no Select)             |

---

## Atari Lynx

> [!IMPORTANT]
> Lynx games need `lynxboot.img`, the Lynx boot ROM. It is **not included**. Put your own copy in `BIOS/`.

| | |
|---|---|
| **Version** | slotPlus and slotPlusUltra |
| **Games** | `.lnx` and `.lyx` in `Games/Lynx/`. |
| **BIOS** | `lynxboot.img` in `BIOS/` (not included). |
| **Core** | Beetle Lynx. |
| **Label art** | `Labels/Lynx/`, about 284x190 px (1.5:1). |
| **Picture** | Shown at 4x, or stretched to fill the screen through `Config/video_mode.txt`. Games made to be held on their side are drawn turned (see below). |
| **Shaders** | Off, LCD3x, Grid, Dot. |
| **Bezel** | Your own in `Bezels/Lynx/`. |
| **Cart** | The curved-lip card, with its gold contacts behind a clear strip at the foot. |
| **Saves** | The core has no battery saves. Use save states and the resume state. |

### Controls

| Press      | Does                                         |
|------------|----------------------------------------------|
| D-pad      | D-pad                                        |
| A / B      | A / B                                        |
| L1 / R1    | Option 1 / Option 2                          |
| Start      | Pause (the Lynx's Option button)             |
| Select     | Picture upright / the way the game is made   |
| Hold X / Y | Turbo A / turbo B                            |

### Games that turn

Some Lynx games were made to be held on their side, some turned a quarter anticlockwise and some a quarter clockwise. Each starts with its picture drawn the way the game is made, at full size: turn the handheld the way the game was meant to be held and it is upright, with the D-pad following it, so press toward the top of the picture for up. A, B, Option 1, Option 2 and Start are unchanged.

Beetle Lynx knows many of these games, and slotPlus also recognises Klax, Gauntlet III, Lexis and NFL Football by name, whichever dump you have. Every other game is drawn as normal.

### Choosing a rotation

On the shelf, press START on a Lynx cart to turn it over. Its back has a socket for each way the handheld can be held: Upright (north), 90CCW (west), 90CW (east) and 180 (south). Each D-pad direction moves the chip to that socket. Press A to keep it or B to leave the game as it was. The choice is saved in `Config/lynx_hold.txt`.

The game then starts upright for the way you chose, with the D-pad turned to match, whichever way the game is made to be held: Upright on a sideways game draws it standing up with bars at the sides, and 180 plays any game with the handheld upside down.

In a game, SELECT switches between the way you are holding it and the picture upright, or, held upright, the way the game is made to be held. Press it again to go back.

### Games slotPlus doesn't recognise

For a game that is made to be held sideways and is not recognised, add a line to `Config/lynx_rotation.txt`, using the game's file name without the extension, so slotPlus knows which way it is made:

```
Game Name (USA) = 90
```

The values are `auto` (let the core decide), `0` (not turned), `90` (made to be held a quarter anticlockwise), `270` (a quarter clockwise), `180`, and `manual`. If a game comes up upside down, swap 90 for 270. With `manual`, SELECT goes to the core instead and each press turns the picture another quarter, so you can find the right setting and then write it into the file. An entry in the file wins over everything else.

> [!NOTE]
> The turned layout and its D-pad feel are new. If something feels wrong, please open an issue.

---

## WonderSwan / WonderSwan Color

| | |
|---|---|
| **Version** | slotPlus and slotPlusUltra |
| **Games** | `.ws` and `.wsc` in `Games/WS/`. Both share one shelf. |
| **BIOS** | Not needed. |
| **Core** | Beetle WonderSwan (Cygne). |
| **Label art** | `Labels/WS/`, about 324x132 px (2.45:1). |
| **Picture** | Horizontal games at 3x, or stretched to fill the screen through `Config/video_mode.txt`. Vertical games are turned (see below). |
| **Shaders** | Off, LCD3x, Grid, Simpletex. |
| **Bezel** | Your own in `Bezels/WS/`. |
| **Cart** | The wide WonderSwan cart, with the oval nameplate on top and the bare board below the shoulder. Black for a WonderSwan game (`.ws`), frosted smoke-clear with the board showing through for a WonderSwan Color game (`.wsc`). |
| **Saves** | Games with battery saves keep them in `Saves/WS/`. |

### Controls (horizontal games)

The WonderSwan has two sets of four direction buttons: the X pad (left side) and the Y pad (right side).

| Press                       | Does                                            |
|-----------------------------|-------------------------------------------------|
| D-pad                       | X pad                                           |
| A / B                       | A / B                                           |
| Start                       | Start                                           |
| Hold **L1** + D-pad         | Y pad (up / right / down / left)                |
| Hold **R1** + X / A / B / Y | Y pad (X = up, A = right, B = down, Y = left)   |
| Select                      | Nothing in horizontal games                     |

The stretch shortcuts are off for WonderSwan games, so L1 and R1 can act as the Y-pad modifiers.

### Vertical games

Games made for vertical play are recognised from the ROM. Play them one of two ways:

- **Handheld upright:** press SELECT and the picture turns upright, at 2x with black bars at the sides.
- **Handheld turned 90 degrees counter-clockwise:** leave the picture sideways at full size.

Either way the D-pad is the Y pad, the four face buttons are the X pad, L1 is A and R1 is B. Upright, D-pad up is up in the game; turned, the buttons keep the WonderSwan's own directions. The L1 / R1 Y-pad modifiers above are for horizontal games only.

> [!NOTE]
> Vertical controls are new but have been tested with several games. If something feels wrong, please open an issue.
