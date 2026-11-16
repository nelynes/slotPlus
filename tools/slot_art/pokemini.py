#!/usr/bin/env python3
"""Draw the Pokémon mini's cart slot for Cart Slot set to Bespoke.

    tools/slot_art/pokemini.py [colour] [--out DIR]

The three strips are the size of Slot's (720x58) with its mouth cut out in the
same place, so a cart goes in the same way. They draw the top of the little
console: glossy rounded plastic, the slot sunk into a raised, rounded block in
the middle, a shallow groove either side where the shell halves meet, and the
small round speaker holes on the right end. No maker's names or marks.

colour is one of the shells (blue, green, purple), or a hex RGB.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from strip import H, ROOT, W, colour, grain, mouth, near, open_mouth, rgb, to_image

SHELLS = {
    'blue': (92, 150, 210),
    'green': (122, 186, 92),
    'purple': (152, 116, 196),
}
# The raised block the slot sits in.
BLOCK = (196, 524)


def back(base):
    """Seen through the mouth: the floor of the slot, and its contacts."""
    light = np.full((H, W), 0.66) * grain(21, 0.02)
    ys = np.arange(H)[:, None]
    light *= 0.78 + 0.22 * np.clip(ys / 26, 0, 1)
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    for i in range(10):
        f = 0.7 + 0.3 * i / 10
        light[:, left + i] *= f
        light[:, right - i] *= f
    img = to_image(light, base)
    d = ImageDraw.Draw(img)
    # The connector at the bottom of the slot: a dark block with gold springs.
    cx = (left + right) // 2
    d.rectangle([cx - 70, 3, cx + 70, 11], fill=(22, 22, 26))
    for x in range(cx - 64, cx + 65, 6):
        d.line([x, 5, x, 9], fill=(196, 160, 70))
    d.line([cx - 70, 12, cx + 70, 12], fill=rgb(base, 0.9))
    return img


def front(base):
    """The shell around the slot, open where the cart goes in."""
    m = mouth()
    light = np.full((H, W), 1.0) * grain(22, 0.015)
    ys = np.arange(H)[:, None].astype(float)
    xs = np.arange(W)[None, :].astype(float)
    # Rounded plastic: lit along the top, curving away below.
    light *= 1.06 - 0.16 * (ys / H) ** 1.5
    # The glossy sheen of the shell, a soft bright band.
    light *= 1.0 + 0.16 * np.exp(-((ys - 9) / 4.0) ** 2)
    x0, x1 = BLOCK
    # The raised block: a touch lighter, rounded ends, a lit top edge.
    inside = (xs >= x0) & (xs <= x1)
    light = np.where(inside, light * 1.05, light)
    for k in range(8):
        f = 1.0 - 0.05 * (8 - k)
        light[:, x0 + k] *= f
        light[:, x1 - k] *= 1.0 - 0.03 * (8 - k)
    light[0:2, x0:x1] *= 1.15
    # The seams where the halves meet, either side of the block.
    for x in (150, 570):
        light[:, x] *= 0.75
        light[:, x + 1] *= 1.12
    # The ends round off into the sides.
    for side in (0, 1):
        for i in range(18):
            f = 0.82 + 0.18 * (i / 18) ** 0.5
            col = i if side == 0 else W - 1 - i
            light[:, col] *= f
    light[near(m, 2)] *= 0.62
    light[near(m, 3) & ~near(m, 2)] *= 1.1
    img = to_image(light, base, np.where(m, 0.0, 255.0))
    d = ImageDraw.Draw(img)
    # The speaker: a small grid of round holes on the right end, kept to the top half
    # where the battery and clock printed over the bottom ends leave it clear.
    for row in range(3):
        for col in range(5):
            x = 630 + col * 9 + (4 if row % 2 else 0)
            y = 11 + row * 7
            d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=rgb(base, 0.35))
            d.point([(x - 1, y - 1)], fill=rgb(base, 0.6))
    # A small arrow on the block pointing into the slot.
    cx = (x0 + x1) // 2
    d.polygon([(cx - 6, 48), (cx + 6, 48), (cx, 54)], fill=rgb(base, 0.72))
    d.line([(cx - 6, 47), (cx + 6, 47)], fill=rgb(base, 1.15))
    return open_mouth(img)


def plugged(base):
    """A cart in the slot: the front over the back, the slot holding the top of a
    grey Pokémon mini cart with its ridged hood."""
    b = np.array(back(base)).astype(float)
    m = mouth()
    cart = np.array(to_image(np.full((H, W), 1.0) * grain(23, 0.03), (96, 98, 104))).astype(float)
    ys = np.arange(H)[:, None]
    hood = np.broadcast_to(((ys >= 2) & (ys < 20))[:, :, None], cart.shape)
    cart[hood] *= 1.12
    ridges = np.broadcast_to(((ys % 4 == 1) & (ys >= 4) & (ys < 18))[:, :, None], cart.shape)
    cart[ridges] *= 0.8
    cart[(ys == 20).repeat(W, 1)] *= 0.6
    inner = m.copy()
    xs = np.where(m[2])[0]
    inner[:, :xs.min() + 10] = False
    inner[:, xs.max() - 9:] = False
    b[inner] = cart[inner]
    b[:, :, 3] = 255
    out = Image.fromarray(b.astype(np.uint8), 'RGBA')
    out.alpha_composite(front(base))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('colour', nargs='?', default='blue')
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    a = ap.parse_args()
    base = colour(a.colour, SHELLS)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    back(base).save(out / 'slot_pm_back.png', optimize=True)
    front(base).save(out / 'slot_pm_front.png', optimize=True)
    plugged(base).save(out / 'slot_pm_plugged.png', optimize=True)
    print(f'slot_pm_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
