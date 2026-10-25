#!/usr/bin/env python3
"""Draw the Neo Geo Pocket Color's cart slot for Cart Slot set to Bespoke.

    tools/slot_art/neogeopocket.py [colour] [--out DIR]

The three strips are the size of Slot's (720x58) with its mouth cut out in the
same place, so a cart goes in the same way. They draw the top of a Neo Geo
Pocket Color's back: the cartridge bay sunk between two raised blocks, its
two guide ribs, the battery door on the left block, the screw holes, number
plate and the top of the coin cell's round door on the right one, and the top
of the silver label under the bay. No maker's names or marks are drawn.

colour is one of the shells (platinum, carbon), or a hex RGB. Carbon is the
dark carbon-fibre print, drawn as its fine grid of dots.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from strip import H, ROOT, W, colour, grain, mouth, near, open_mouth, rgb, to_image

SHELLS = {
    'platinum': (150, 164, 190),
    'carbon': (52, 53, 58),
}
# The blocks either side of the bay end here.
BAY = (196, 524)


def surface(base, seed, carbon):
    """The case's plastic as a light multiplier: grain, and carbon's dots."""
    light = np.full((H, W), 1.0) * grain(seed, 0.025)
    if carbon:
        ys, xs = np.mgrid[0:H, 0:W]
        # A staggered grid of dots, every 5 pixels.
        row = ys // 5
        dx = (xs + (row % 2) * 2.5) % 5 - 2.5
        dy = ys % 5 - 2.5
        dot = (dx * dx + dy * dy) < 2.6
        light[dot] *= 0.62
    return light


def back(base, carbon):
    """Seen through the mouth: the floor of the bay, with its guide ribs."""
    light = surface(base, 7, carbon) * 0.7
    ys = np.arange(H)[:, None]
    light *= 0.82 + 0.18 * np.clip(ys / 30, 0, 1)
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    for i in range(10):
        f = 0.75 + 0.25 * i / 10
        light[:, left + i] *= f
        light[:, right - i] *= f
    img = to_image(light, base)
    d = ImageDraw.Draw(img)
    for x in (left + 52, right - 58):
        d.rectangle([x, 0, x + 5, 34], fill=rgb(base, 0.85))
        d.line([x, 0, x, 34], fill=rgb(base, 1.1))
        d.line([x + 6, 0, x + 6, 34], fill=rgb(base, 0.4))
    return img


def front(base, carbon):
    """The case around the bay, open where the cart goes in."""
    m = mouth()
    light = surface(base, 8, carbon)
    ys = np.arange(H)[:, None]
    light *= 1.03 - 0.07 * ys / H
    x0, x1 = BAY
    # The bay floor between the blocks sits lower, in their shadow.
    light[:, x0:x1] *= 0.84
    for i in range(6):
        light[:, x0 + i] *= 0.7 + 0.05 * i
        light[:, x1 - 1 - i] *= 0.7 + 0.05 * i
    light[:, x0 - 1] *= 1.2
    light[:, x1] *= 1.2
    light[0, :x0] *= 1.25
    light[0, x1:] *= 1.25
    light[near(m, 2)] *= 0.62
    img = to_image(light, base, np.where(m, 0.0, 255.0))
    d = ImageDraw.Draw(img)
    seam, lit = rgb(base, 0.32 if carbon else 0.5), rgb(base, 1.6 if carbon else 1.25)
    # The battery door on the left block, running off the bottom, with the
    # panel moulded in its middle.
    d.rounded_rectangle([12, 7, 176, H + 8], radius=5, outline=seam, width=2)
    d.line([16, 8, 172, 8], fill=lit)
    d.rounded_rectangle([60, 18, 128, H + 8], radius=3, outline=rgb(base, 0.75), width=1)
    # Screw holes on the right block, and one by the battery door.
    for cx, cy in ((550, 14), (186, 46)):
        d.ellipse([cx - 7, cy - 7, cx + 7, cy + 7], fill=rgb(base, 0.6))
        d.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=(18, 18, 20))
    # The number plate, left blank.
    d.rectangle([600, 6, 704, 17], fill=(206, 206, 200), outline=rgb(base, 0.45))
    d.line([601, 7, 703, 7], fill=(236, 236, 232))
    # The top of the coin cell's round door, in the right block's corner.
    d.ellipse([566, 30, 690, 154], outline=seam, width=2)
    d.arc([568, 32, 688, 152], 200, 340, fill=lit)
    # The top of the silver label under the bay.
    label = [258, 47, 462, 60]
    d.rectangle(label, fill=(196, 198, 202))
    d.line([258, 47, 462, 47], fill=(232, 233, 236))
    for a, b in ((272, 330), (342, 400), (412, 448)):
        d.line([a, 52, b, 52], fill=(120, 122, 126))
        d.line([a, 55, b - 14, 55], fill=(140, 142, 146))
    return open_mouth(img)


def plugged(base, carbon):
    """A cart in the bay: the front over the back, the bay holding the top of
    a dark grey Neo Geo Pocket cart and its white label."""
    b = np.array(back(base, carbon)).astype(float)
    m = mouth()
    cart = np.array(to_image(np.full((H, W), 1.0) * grain(9, 0.03), (70, 72, 78))).astype(float)
    ys = np.arange(H)[:, None]
    # The cart's top edge is a step, lit along its lip.
    cart[(ys == 12).repeat(W, 1)] *= 1.4
    label = np.zeros((H, W), bool)
    label[18:, 276:444] = True
    cart[label] = (232, 230, 224, 255)
    inner = m.copy()
    xs = np.where(m[2])[0]
    inner[:, :xs.min() + 8] = False
    inner[:, xs.max() - 7:] = False
    b[inner] = cart[inner]
    b[:, :, 3] = 255
    out = Image.fromarray(b.astype(np.uint8), 'RGBA')
    out.alpha_composite(front(base, carbon))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('colour', nargs='?', default='carbon')
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    a = ap.parse_args()
    base = colour(a.colour, SHELLS)
    carbon = a.colour == 'carbon'
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    back(base, carbon).save(out / 'slot_ngp_back.png', optimize=True)
    front(base, carbon).save(out / 'slot_ngp_front.png', optimize=True)
    plugged(base, carbon).save(out / 'slot_ngp_plugged.png', optimize=True)
    print(f'slot_ngp_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
