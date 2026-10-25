#!/usr/bin/env python3
"""Draw the Lynx's cart slot for Cart Slot set to Bespoke.

    tools/slot_art/lynx.py [colour] [--out DIR]

The three strips are the size of Slot's (720x58) with its mouth cut out in the
same place, so a cart goes in the same way. They draw the top of a Lynx II's
back: the cartridge bay sunk into the raised middle block, the step of the
band below it, the two recesses either side for the rating label and serial
number, and the rubber grips with their screws on the ends. No maker's names
or marks are drawn: the band is left plain.

colour is the Lynx's dark grey, or a hex RGB.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from strip import H, ROOT, W, colour, grain, mouth, near, open_mouth, rgb, to_image

SHELLS = {
    'grey': (68, 70, 75),
}
# The raised middle block, and the rubber grips on the ends.
BLOCK = (176, 544)
GRIP = 54


def back(base):
    """Seen through the mouth: the floor of the bay."""
    light = np.full((H, W), 0.72) * grain(13, 0.03)
    ys = np.arange(H)[:, None]
    light *= 0.8 + 0.2 * np.clip(ys / 28, 0, 1)
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    for i in range(10):
        f = 0.72 + 0.28 * i / 10
        light[:, left + i] *= f
        light[:, right - i] *= f
    img = to_image(light, base)
    d = ImageDraw.Draw(img)
    # The cart's edge connector slot at the top of the bay.
    d.rectangle([left + 22, 2, right - 22, 7], fill=(16, 16, 18))
    d.line([left + 22, 8, right - 22, 8], fill=rgb(base, 1.0))
    return img


def front(base):
    """The case around the bay, open where the cart goes in."""
    m = mouth()
    light = np.full((H, W), 1.0) * grain(14, 0.04)
    ys = np.arange(H)[:, None]
    light *= 1.03 - 0.07 * ys / H
    x0, x1 = BLOCK
    # The middle block stands proud: lighter, with lit and shaded edges.
    light[:, x0:x1] *= 1.08
    light[:, x0:x0 + 2] *= 1.25
    light[:, x1 - 2:x1] *= 0.7
    light[0, x0:x1] *= 1.25
    # The band under the bay steps out of the block.
    light[46:, x0 + 6:x1 - 6] *= 1.07
    light[45, x0 + 6:x1 - 6] *= 1.4
    light[46, x0 + 6:x1 - 6] *= 1.15
    # The rubber grips on the ends: darker, smooth, curving in.
    for side in (0, 1):
        for y in range(H):
            reach = GRIP - int(10 * (1 - ((y - H / 2) / (H / 2)) ** 2))
            if side == 0:
                light[y, :reach] = 0.62 + 0.04 * np.sin(y / 9)
                light[y, reach] *= 0.6
            else:
                light[y, W - reach:] = 0.62 + 0.04 * np.sin(y / 9)
                light[y, W - reach - 1] *= 0.6
    light[near(m, 2)] *= 0.6
    img = to_image(light, base, np.where(m, 0.0, 255.0))
    d = ImageDraw.Draw(img)
    seam, lit = rgb(base, 0.55), rgb(base, 1.3)
    # The recesses for the rating label and the serial number.
    for a, b in ((84, 152), (568, 636)):
        d.rectangle([a, 8, b, H + 4], outline=seam)
        d.line([a + 1, 9, b - 1, 9], fill=lit)
    # Screws in the grips.
    for cx in (22, 698):
        d.ellipse([cx - 5, 8, cx + 5, 18], fill=rgb(base, 0.4))
        d.ellipse([cx - 3, 10, cx + 3, 16], fill=(150, 140, 110))
        d.point([(cx, 13)], fill=(60, 56, 44))
    return open_mouth(img)


def plugged(base):
    """A cart in the bay: the front over the back, the bay holding the top of
    a grey Lynx cart with its rounded lip."""
    b = np.array(back(base)).astype(float)
    m = mouth()
    cart = np.array(to_image(np.full((H, W), 1.0) * grain(15, 0.03), (104, 106, 112))).astype(float)
    ys = np.arange(H)[:, None]
    lip = np.broadcast_to(((ys >= 2) & (ys < 14))[:, :, None], cart.shape)
    cart[lip] *= 1.15
    cart[(ys == 14).repeat(W, 1)] *= 0.6
    cart[20:, 288:432, :3] = (226, 224, 218)
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
    ap.add_argument('colour', nargs='?', default='grey')
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    a = ap.parse_args()
    base = colour(a.colour, SHELLS)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    back(base).save(out / 'slot_lynx_back.png', optimize=True)
    front(base).save(out / 'slot_lynx_front.png', optimize=True)
    plugged(base).save(out / 'slot_lynx_plugged.png', optimize=True)
    print(f'slot_lynx_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
