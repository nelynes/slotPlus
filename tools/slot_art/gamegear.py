#!/usr/bin/env python3
"""Draw the Game Gear's cart slot for Cart Slot set to Bespoke.

    tools/slot_art/gamegear.py [colour] [--out DIR]

The three strips are the size of Slot's (720x58) with its mouth cut out in the
same place, so a cart goes in the same way. They draw the top of a Game
Gear's back: the cartridge notch in the middle between its raised shoulders,
the moulded CARTRIDGE arrow under it, a T-shaped vent either side, the screw
holes, and the two battery doors on the ends. No maker's names or marks are
drawn.

colour is one of the Game Gear's shells (black, blue), or a hex RGB.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from strip import H, ROOT, W, colour, embossed, grain, mouth, near, open_mouth, rgb, to_image

SHELLS = {
    'black': (54, 55, 60),
    'blue': (44, 84, 176),
}
# Where the raised shoulders either side of the notch end.
SHOULDER = 40


def back(base):
    """Seen through the mouth: the floor of the cartridge notch."""
    light = np.full((H, W), 0.62) * grain(4, 0.02)
    ys = np.arange(H)[:, None]
    # In shadow under the shoulders at the top, lit further down.
    light *= 0.8 + 0.2 * np.clip(ys / 26, 0, 1)
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    for i in range(10):
        f = 0.55 + 0.07 * i / 10
        light[:, left + i] = f
        light[:, right - i] = f
    img = to_image(light, base)
    d = ImageDraw.Draw(img)
    # The screw boss in the middle of the floor.
    cx = (left + right) // 2
    d.ellipse([cx - 9, 21, cx + 9, 39], fill=rgb(base, 0.48))
    d.ellipse([cx - 7, 23, cx + 7, 37], fill=rgb(base, 0.72))
    d.ellipse([cx - 3, 27, cx + 3, 33], fill=(150, 128, 70))
    d.ellipse([cx - 2, 28, cx + 1, 31], fill=(196, 172, 104))
    # The contacts' slit across the top, where the cart goes in.
    d.rectangle([left + 14, 2, right - 14, 5], fill=rgb(base, 0.25))
    return img


def front(base):
    """The back of the case around the notch, open where the cart goes in."""
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    light = np.full((H, W), 1.0) * grain(5, 0.03)
    ys = np.arange(H)[:, None]
    light *= 1.04 - 0.08 * ys / H
    # The shoulders stand proud of the case: a lit top and a shaded fall-off
    # where they slope down to it.
    for x0, x1, outward in ((left - SHOULDER, left, -1), (right, right + SHOULDER, 1)):
        light[:, x0:x1] *= 1.08
        edge = x0 if outward < 0 else x1
        for i in range(8):
            x = edge - i * outward if outward < 0 else edge + i
            light[:, x] *= 0.78 + 0.03 * i
    light[0, :] *= 1.25
    light[1, :] *= 1.1
    light[near(m, 2)] *= 0.6
    light[near(m, 3) & ~near(m, 2)] *= 1.15
    img = to_image(light, base, np.where(m, 0.0, 255.0))
    d = ImageDraw.Draw(img)
    seam, lit = rgb(base, 0.55), rgb(base, 1.3)
    # The battery doors on the ends, running off the bottom.
    for x0, x1 in ((10, 140), (580, 710)):
        d.rounded_rectangle([x0, 8, x1, H + 8], radius=6, outline=seam, width=2)
        d.line([x0 + 6, 9, x1 - 6, 9], fill=lit)
        # The OPEN arrow points up the door.
        cx = (x0 + x1) // 2
        d.line([cx - 6, 16, cx, 12, cx + 6, 16], fill=seam, width=1)
        d.line([cx - 6, 17, cx, 13, cx + 6, 17], fill=lit, width=1)
    # A T-shaped vent either side of the notch.
    for cx in (left - SHOULDER - 18, right + SHOULDER + 18):
        hole = rgb(base, 0.3)
        d.rectangle([cx - 9, 28, cx + 9, 32], fill=hole)
        d.rectangle([cx - 2, 28, cx + 2, 46], fill=hole)
        d.line([cx - 9, 33, cx + 9, 33], fill=lit)
    # Screw holes, round and deep.
    for cx, cy in ((left - SHOULDER - 18, 12), (right + SHOULDER + 18, 12)):
        d.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], fill=rgb(base, 0.62))
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=rgb(base, 0.18))
    # CARTRIDGE and its arrow, moulded under the notch.
    cx = (left + right) // 2
    embossed(d, (cx, 49), 'CARTRIDGE', 8, rgb(base, 1.25))
    d.line([cx - 7, 54, cx, 57, cx + 7, 54], fill=rgb(base, 0.6))
    return open_mouth(img)


def plugged(base):
    """A cart in the notch: the front over the back, the notch holding the top
    of a black Game Gear cart."""
    b = np.array(back(base)).astype(float)
    m = mouth()
    cart = np.array(to_image(np.full((H, W), 1.0) * grain(6, 0.03), (30, 30, 34))).astype(float)
    ys = np.arange(H)[:, None]
    # The grip lines across the cart's top.
    ridged = np.broadcast_to(((ys % 5 == 0) & (ys > 3) & (ys < 22))[:, :, None], cart.shape)
    cart[ridged] *= 0.6
    inner = m.copy()
    xs = np.where(m[2])[0]
    inner[:, :xs.min() + 9] = False
    inner[:, xs.max() - 8:] = False
    b[inner] = cart[inner]
    b[:, :, 3] = 255
    out = Image.fromarray(b.astype(np.uint8), 'RGBA')
    out.alpha_composite(front(base))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('colour', nargs='?', default='black')
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    a = ap.parse_args()
    base = colour(a.colour, SHELLS)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    back(base).save(out / 'slot_gg_back.png', optimize=True)
    front(base).save(out / 'slot_gg_front.png', optimize=True)
    plugged(base).save(out / 'slot_gg_plugged.png', optimize=True)
    print(f'slot_gg_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
