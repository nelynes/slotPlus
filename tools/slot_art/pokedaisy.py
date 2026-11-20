#!/usr/bin/env python3
"""Draw the PokeDaisy shelf's cart slot for Cart Slot set to Bespoke.

    tools/slot_art/pokedaisy.py [colour] [--out DIR]

PokeDaisy is a player, not a handheld, so its slot is slotPlus's own: a meadow of
moulded plastic, the slot sunk into a raised, rounded block in the middle, as a
Game Boy Advance's is, since all three of the shelf's systems' carts go in it. Small
daisies are printed across the shell either side of the block, with a few blades of
grass under them, kept clear of the ends' middle, where the battery, wifi and clock
are printed. No maker's names or marks: no game's, no creature's and no ball.

The three strips are the size of Slot's (720x58) with its mouth cut out in the same
place, so a cart goes in the same way.

colour is one of the shells (meadow, dusk, cream), or a hex RGB.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from strip import H, ROOT, W, colour, grain, mouth, near, open_mouth, rgb, to_image

SHELLS = {
    'meadow': (88, 148, 86),
    'dusk': (70, 92, 140),
    'cream': (232, 222, 196),
}
# The raised block the slot sits in.
BLOCK = (200, 520)
PETAL = (250, 248, 240)
HEART = (240, 196, 60)


def back(base):
    """Seen through the mouth: the floor of the slot, and its contacts."""
    light = np.full((H, W), 0.62) * grain(31, 0.02)
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
    d.rectangle([cx - 96, 3, cx + 96, 11], fill=(22, 24, 22))
    for x in range(cx - 90, cx + 91, 6):
        d.line([x, 5, x, 9], fill=(196, 160, 70))
    d.line([cx - 96, 12, cx + 96, 12], fill=rgb(base, 0.9))
    return img


def daisy(d, x, y, r, tilt):
    """A small daisy: eight petals round a yellow heart, lightly shaded."""
    for k in range(8):
        a = tilt + k * np.pi / 4
        px, py = x + np.cos(a) * r * 0.62, y + np.sin(a) * r * 0.62
        pr = r * 0.42
        d.ellipse([px - pr, py - pr, px + pr, py + pr], fill=PETAL)
    for k in range(8):
        a = tilt + k * np.pi / 4
        px, py = x + np.cos(a) * r * 0.7, y + np.sin(a) * r * 0.7
        d.point([(px, py)], fill=(222, 220, 210))
    h = r * 0.34
    d.ellipse([x - h, y - h, x + h, y + h], fill=HEART)
    d.point([(x - h * 0.4, y - h * 0.4)], fill=(252, 226, 120))


def blades(d, x, y, base, seed):
    """A tuft of grass under a daisy, a shade darker than the shell."""
    rng = np.random.default_rng(seed)
    for _ in range(3):
        dx = rng.uniform(-5, 5)
        lean = rng.uniform(-3, 3)
        tall = rng.uniform(5, 9)
        d.line([(x + dx, y), (x + dx + lean, y - tall)], fill=rgb(base, 0.78), width=1)


# Where the daisies sit, (x, y, radius, tilt). The battery, wifi and clock are printed
# across the middle of each end (rows 17 to 41), so the ends carry a border of small
# daisies along the top, and the bigger ones, with grass, stand in the gaps between the
# printing and the block. Spaced a little unevenly so the meadow doesn't look stamped.
BORDER = [(x, 7 + (i % 2), 3.4, 0.4 * i) for i, x in enumerate(
    [22, 41, 62, 80, 101, 121, 140, 582, 601, 622, 641, 660, 680, 699])]
FLOWERS = [
    (166, 18, 5.5, 0.2), (184, 37, 4.2, 0.6), (158, 44, 3.6, 0.1),
    (540, 20, 4.4, 0.5), (560, 39, 5.5, 0.0), (576, 17, 3.6, 0.3),
]


def front(base):
    """The shell around the slot, open where the cart goes in."""
    m = mouth()
    light = np.full((H, W), 1.0) * grain(32, 0.015)
    ys = np.arange(H)[:, None].astype(float)
    xs = np.arange(W)[None, :].astype(float)
    # Rounded plastic: lit along the top, curving away below.
    light *= 1.06 - 0.16 * (ys / H) ** 1.5
    light *= 1.0 + 0.12 * np.exp(-((ys - 9) / 4.0) ** 2)
    x0, x1 = BLOCK
    # The raised block: a touch lighter, rounded ends, a lit top edge.
    inside = (xs >= x0) & (xs <= x1)
    light = np.where(inside, light * 1.06, light)
    for k in range(8):
        light[:, x0 + k] *= 1.0 - 0.05 * (8 - k)
        light[:, x1 - k] *= 1.0 - 0.03 * (8 - k)
    light[0:2, x0:x1] *= 1.15
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
    for x, y, r, tilt in BORDER:
        daisy(d, x, y, r, tilt)
    for i, (x, y, r, tilt) in enumerate(FLOWERS):
        blades(d, x, min(y + r + 4, H - 4), base, i)
        daisy(d, x, y, r, tilt)
    # A small arrow on the block pointing into the slot.
    cx = (x0 + x1) // 2
    d.polygon([(cx - 6, 48), (cx + 6, 48), (cx, 54)], fill=rgb(base, 0.72))
    d.line([(cx - 6, 47), (cx + 6, 47)], fill=rgb(base, 1.15))
    return open_mouth(img)


def plugged(base):
    """A cart in the slot: the front over the back, the slot holding the top of a grey
    cart, as any of the shelf's three systems' would show."""
    b = np.array(back(base)).astype(float)
    m = mouth()
    cart = np.array(to_image(np.full((H, W), 1.0) * grain(33, 0.03), (118, 118, 126))).astype(float)
    ys = np.arange(H)[:, None]
    lip = np.broadcast_to(((ys >= 2) & (ys < 8))[:, :, None], cart.shape)
    cart[lip] *= 1.1
    cart[(ys == 8).repeat(W, 1)] *= 0.7
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
    ap.add_argument('colour', nargs='?', default='meadow')
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    a = ap.parse_args()
    base = colour(a.colour, SHELLS)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    back(base).save(out / 'slot_pd_back.png', optimize=True)
    front(base).save(out / 'slot_pd_front.png', optimize=True)
    plugged(base).save(out / 'slot_pd_plugged.png', optimize=True)
    print(f'slot_pd_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
