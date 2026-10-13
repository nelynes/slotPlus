<div align="center">

<img src=".github/resources/banner.png" width="100%" alt="slot. A bespoke, Game Boy-centric frontend for the Anbernic RG SP.">

</div>

---

# What is slotPlus?

A slightly modified Slot. Has support for GBA, GBC, and GB titles as well as WonderSwan, WonderSwan Color and Atari Lynx.

## Atari Lynx

> [!IMPORTANT]
> Atari Lynx games need `lynxboot.img`, the Lynx boot ROM. It is **not included**. Put your own copy in `BIOS/`.

- **Games:** put `.lnx` and `.lyx` files in `Games/Lynx/`. They get their own shelf. The `Lynx` folders (`Games`, `Labels`, `Saves`, `States`) are created for you on first boot.
- **Label art:** optional PNGs go in `Labels/Lynx/<game name>.png`, roughly 268x221 px (about 1.21:1).
- **Core:** Lynx games run on Beetle Lynx.
- **Carts:** Lynx carts have their own shape with a stepped plug and a dark shell. Set `Config/cart_shell.txt` to give a single game a different shell.
- **Saves:** Beetle Lynx has no battery saves. Use save states and the resume state.

### Controls

| Press        | Does                                  |
|--------------|---------------------------------------|
| D-pad        | D-pad                                 |
| A / B        | A / B                                 |
| L1 / R1      | Option 1 / Option 2                   |
| Start        | Pause (the Lynx's Option button)      |
| Select       | Nothing                               |
| Hold X / Y   | Turbo A / Turbo B                     |

### Games that turn

Some Lynx games were made to be held on their side. Beetle Lynx knows which, and slot turns the picture for them automatically. The core also turns the D-pad to match.

> [!NOTE]
> Rotation direction and D-pad feel are new. If something feels wrong, please open an issue.

## WonderSwan / WonderSwan Color

- **Games:** put `.ws` and `.wsc` files in `Games/WS/`. They get their own shelf next to GBA, GB and GBC. The `WS` folders (`Games`, `Labels`, `Saves`, `States`) are created for you on first boot.
- **Label art:** optional PNGs go in `Labels/WS/<game name>.png`, roughly 328x124 px (about 2.65:1).
- **Core:** WonderSwan games run on Beetle WonderSwan (Cygne).
- **Carts:** WonderSwan carts have their own wide shape with a stepped plug, and default to a smoke-translucent shell. Set `Config/cart_shell.txt` to give a single game a different shell.

### Controls (horizontal games)

The WonderSwan has two sets of four direction buttons: the X pad (left side) and the Y pad (right side).

| Press                  | Does                                              |
|------------------------|---------------------------------------------------|
| D-pad                  | X pad                                             |
| A / B                  | A / B                                             |
| Start                  | Start                                             |
| Hold **L1** + D-pad    | Y pad (up / right / down / left) |
| Hold **R1** + X / A / B / Y | Y pad (X = up, A = right, B = down, Y = left) |
| Select                 | Nothing in horizontal games (they can't be rotated) |

Picture-mode shortcuts are turned off for WonderSwan games, so L1 and R1 can act as the Y-pad modifiers.

### Vertical games

Games made for vertical play are recognized from the ROM. Play them one of two ways:

- **Device upright:** press SELECT and the picture turns upright (it shows at 2x with black bars at the sides).
- **Device turned 90 degrees counter-clockwise:** leave the picture sideways at full size.

Either way the D-pad is the Y pad, the four face buttons are the X pad, L1 is A and R1 is B. Upright, D-pad up is up in the game. Turned sideways, the buttons keep the WonderSwan's own directions. The L1 / R1 Y-pad modifiers above are for horizontal games only.

> [!NOTE]
> Vertical controls are new but I have tested with multiple games. If something feels wrong, please open an issue.

---

# What is slot?

A bespoke, Game Boy-centric frontend for the Anbernic RG SP.

Has support for GBA, GBC, and GB titles only.

> [!IMPORTANT]
> ROMs must be unzipped. slot does not read `.zip` or `.7z` files.

A full user guide can be found at [slot-cfw.fyi](https://slot-cfw.fyi).

Release notes can be found in the [changelog](CHANGELOG.md).

---

# What does it look like?

<p align="center">
<img src=".github/resources/screenshots/carousel.gif" width="48%" alt="Scrolling the cart carousel across the Game Boy Advance, Game Boy and Game Boy Color shelves">
<img src=".github/resources/screenshots/core-picker.gif" width="48%" alt="Flipping a cart to its circuit board to choose mGBA or gpSP">
</p>

---

# Supported Devices

| Device             | Supported                  | Since Version |
|--------------------|----------------------------|---------------|
| Anbernic RG SP     | Yes                        | Always        |
| Anbernic RG34XX    | Yes                        | 1.4.0         |
| Anbernic RG34XXSP  | Yes (No Stick Support Yet) | 1.4.0         |
| Anbernic RG35XXSP  | Yes                        | 1.5.0         |
| Anbernic 35XX Plus | Yes                        | 1.5.0         |
| Anbernic RG28XX    | Yes                        | 1.6.0         |

RG28XX support was contributed by [GamerAnim](https://github.com/GamerAnim). Its panel is mounted
portrait, so slot turns the picture onto it. If it comes out upside down, add `export SLOT_ROTATE=ccw`
to `System/launch_frontend.sh`.

If you have one of the untested devices, or have tried slot on a device not listed,
please [open an issue on GitHub](https://github.com/BrandonKowalski/slot/issues) and let me know how it went.

---

# AI Disclosure

The Rust frontend was put together by Claude Opus. I reviewed everything that was
produced. All documentation is 100% free-range, meatbag prose.

The project is extremely low stakes. I wanted a bespoke frontend for my RG SP and thought
that something that evokes the feeling of using my GBA SP as a kid would be pretty neat.

Use it, don't use it, I don't care.

Figured I should share the end result of all the wasted water. ✌🏻
