#!/usr/bin/env python3
"""Draw the TurboExpress's cart slot, for the TurboGrafx-16 shelf with Cart
Slot set to Bespoke.

    tools/slot_art/turboexpress.py [colour] [--out DIR]

The three strips are the size of Slot's (720x58) with its mouth cut out in the
same place, so a HuCard goes in the same way. They draw the top of a
TurboExpress's back: the HuCard's sunken panel between the raised shoulders,
the grooves down its sides and the arrow moulded across it, the screw holes
in the top corners, and the DC IN recess on the right edge. No maker's names
or marks are drawn.

colour is the TurboExpress's charcoal, or a hex RGB.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from strip import H, ROOT, W, colour, grain, mouth, near, open_mouth, rgb, to_image

SHELLS = {
    'charcoal': (62, 64, 69),
}
# The sunken panel the HuCard goes into spans this.
PANEL = (190, 530)


def back(base):
    """Seen through the mouth: the HuCard slit, deep and dark, with its guides."""
    light = np.full((H, W), 0.55) * grain(10, 0.02)
    ys = np.arange(H)[:, None]
    light *= 0.75 + 0.25 * np.clip(ys / 30, 0, 1)
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    for i in range(10):
        f = 0.7 + 0.3 * i / 10
        light[:, left + i] *= f
        light[:, right - i] *= f
    img = to_image(light, base)
    d = ImageDraw.Draw(img)
    # The slit the card slides into, and its two guides.
    d.rectangle([left + 16, 3, right - 16, 9], fill=(14, 14, 16))
    d.line([left + 16, 10, right - 16, 10], fill=rgb(base, 0.9))
    for x in (left + 20, right - 26):
        d.rectangle([x, 10, x + 6, 30], fill=rgb(base, 0.4))
    return img


def front(base):
    """The case around the HuCard panel, open where the card goes in."""
    m = mouth()
    light = np.full((H, W), 1.0) * grain(11, 0.03)
    ys = np.arange(H)[:, None]
    light *= 1.04 - 0.08 * ys / H
    x0, x1 = PANEL
    # The panel is sunk below the shoulders: darker, shaded under their edges.
    light[:, x0:x1] *= 0.86
    for i in range(7):
        light[:, x0 + i] *= 0.72 + 0.04 * i
        light[:, x1 - 1 - i] *= 0.72 + 0.04 * i
    light[:, x0 - 2:x0] *= 1.25
    light[:, x1:x1 + 2] *= 1.25
    light[0, :x0] *= 1.3
    light[0, x1:] *= 1.3
    light[near(m, 2)] *= 0.6
    img = to_image(light, base, np.where(m, 0.0, 255.0))
    d = ImageDraw.Draw(img)
    groove, lit = rgb(base, 0.45), rgb(base, 1.3)
    # The grooves down the panel's sides.
    for x in (x0 + 18, x1 - 20):
        d.line([x, 0, x, 46], fill=groove, width=2)
        d.line([x + 2, 0, x + 2, 46], fill=lit)
    # The line moulded across the panel, broken by its arrow pointing in.
    cx = (x0 + x1) // 2
    for a, b in ((x0 + 18, cx - 14), (cx + 14, x1 - 18)):
        d.line([a, 47, b, 47], fill=groove, width=2)
        d.line([a, 49, b, 49], fill=lit)
    d.line([cx, 44, cx, 54], fill=groove, width=2)
    d.line([cx - 9, 47, cx, 56, cx + 9, 47], fill=groove, width=2)
    # Screw holes in the top corners.
    for hx in (26, 694):
        d.ellipse([hx - 6, 6, hx + 6, 18], fill=rgb(base, 0.6))
        d.ellipse([hx - 4, 8, hx + 4, 16], fill=(16, 16, 18))
    # The DC IN recess on the right edge.
    d.rounded_rectangle([706, 22, 716, H + 6], radius=3, outline=groove, width=1)
    return open_mouth(img)


def plugged(base):
    """A HuCard in the slot: the front over the back, the slot holding the
    top of a card and its grip edge."""
    b = np.array(back(base)).astype(float)
    m = mouth()
    card = np.array(to_image(np.full((H, W), 1.0) * grain(12, 0.02), (196, 198, 202))).astype(float)
    ys = np.arange(H)[:, None]
    # The grip ridges along the card's top edge.
    ridged = np.broadcast_to(((ys % 3 == 0) & (ys < 12))[:, :, None], card.shape)
    card[ridged] *= 0.72
    # The card's label below its edge.
    card[16:, 286:434, :3] = (84, 88, 104)
    inner = m.copy()
    xs = np.where(m[2])[0]
    inner[:, :xs.min() + 12] = False
    inner[:, xs.max() - 11:] = False
    b[inner] = card[inner]
    b[:, :, 3] = 255
    out = Image.fromarray(b.astype(np.uint8), 'RGBA')
    out.alpha_composite(front(base))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('colour', nargs='?', default='charcoal')
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    a = ap.parse_args()
    base = colour(a.colour, SHELLS)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    back(base).save(out / 'slot_tg16_back.png', optimize=True)
    front(base).save(out / 'slot_tg16_front.png', optimize=True)
    plugged(base).save(out / 'slot_tg16_plugged.png', optimize=True)
    print(f'slot_tg16_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
