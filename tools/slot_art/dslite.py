#!/usr/bin/env python3
"""Draw the DS Lite's game card slot, for the Nintendo DS shelf with Cart Slot
set to Bespoke.

    tools/slot_art/dslite.py [colour] [--out DIR]

The three strips are the size of Slot's (720x58) with its mouth cut out in the
same place, so a game card goes in the same way. They draw the top of a DS
Lite's back: the glossy band under the hinge, the hinge's knuckles at either
end, the curved scoop the card slot sits in, the two square screw recesses on
the band, and the little card-and-arrow mark beside the slot. No maker's
names or marks are drawn.

colour is one of the DS Lite's black, silver or rose, or a hex RGB.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

import strip
from strip import H, ROOT, W, colour, grain, near, rgb, to_image

SHELLS = {
    'black': (38, 38, 42),
    'silver': (172, 174, 178),
    'rose': (190, 166, 164),
}
# A DS card is far smaller than a GBA cart, so it seats narrower (slot-ui's
# NDS seated width, 168) and the slot is Slot's mouth narrowed to match, a
# little wider than the card.
CARD_W = 168
NARROW = (CARD_W + 14) / (240 + 14)
# The scoop round the slot: a shallow dish a little wider than the mouth.
SCOOP = (238, 482)
# The hinge's knuckles at either end of the band.
KNUCKLES = ((0, 70), (650, 720))
CARD = (58, 59, 63)  # the game card's dark grey, as slot-ui's NDS_SHELL


def mouth():
    """Slot's mouth, narrowed about the middle to the DS card's width."""
    m = strip.mouth()
    xs = np.arange(W)
    centre = (W - 1) / 2
    src = np.clip(np.round(centre + (xs - centre) / NARROW).astype(int), 0, W - 1)
    return m[:, src]


def open_mouth(img):
    """Keep the narrowed opening open whatever was drawn across it."""
    out = np.array(img)
    out[:, :, 3] = np.where(mouth(), 0, out[:, :, 3])
    return Image.fromarray(out, 'RGBA')


def gloss(light):
    """The DS Lite's piano-gloss: a soft bright band along the top of the
    case and a fainter one lower down."""
    ys = np.arange(H)[:, None] / H
    return light * (1.0 + 0.10 * np.exp(-((ys - 0.12) / 0.10) ** 2)
                    + 0.04 * np.exp(-((ys - 0.7) / 0.12) ** 2))


def back(base):
    """Seen through the mouth: the card slot, deep and dark, with its guides."""
    light = np.full((H, W), 0.5) * grain(20, 0.02)
    ys = np.arange(H)[:, None]
    light *= 0.7 + 0.3 * np.clip(ys / 30, 0, 1)
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    for i in range(10):
        f = 0.7 + 0.3 * i / 10
        light[:, left + i] *= f
        light[:, right - i] *= f
    img = to_image(light, base)
    d = ImageDraw.Draw(img)
    # The slot the card slides into, narrower than the mouth as the card is,
    # with a guide on each side.
    cx = (left + right) // 2
    d.rectangle([cx - 86, 3, cx + 86, 9], fill=(12, 12, 14))
    d.line([cx - 86, 10, cx + 86, 10], fill=rgb(base, 0.9))
    for x in (cx - 92, cx + 86):
        d.rectangle([x, 4, x + 6, 28], fill=rgb(base, 0.38))
    return img


def front(base):
    """The case round the slot, open where the card goes in."""
    m = mouth()
    light = gloss(np.full((H, W), 1.0) * grain(21, 0.015))
    ys = np.arange(H)[:, None]
    xs = np.arange(W)[None, :]

    # The scoop: a shallow dish round the mouth, deepest by the slot, with a
    # bright lip along its rim where the case curves back up.
    x0, x1 = SCOOP
    cx, half = (x0 + x1) / 2, (x1 - x0) / 2
    depth_y = 50.0
    u = ((xs - cx) / half) ** 2 + (ys / depth_y) ** 2
    dish = np.clip(1.0 - u, 0, 1)
    light *= 1.0 - 0.22 * dish ** 0.6
    rim = np.exp(-((u - 1.0) / 0.05) ** 2)
    light *= 1.0 + 0.16 * rim * (ys > 2)

    # The hinge's knuckles: cylinders across the top of the ends, lit along
    # their crown, with a dark gap where they meet the band.
    for k0, k1 in KNUCKLES:
        part = (xs >= k0) & (xs < k1) & (ys < 26)
        crown = 0.82 + 0.38 * np.exp(-((ys - 8) / 6.0) ** 2)
        light = np.where(part, light * crown / 1.06, light)
        gap = (xs >= k0) & (xs < k1) & (ys >= 26) & (ys < 29)
        light = np.where(gap, light * 0.45, light)
        inner = k1 if k0 == 0 else k0
        side = (np.abs(xs - inner) < 2) & (ys < 29)
        light = np.where(side, light * 0.5, light)

    light[0, :] *= 1.2
    light[near(m, 2)] *= 0.55
    img = to_image(light, base, np.where(m, 0.0, 255.0))
    d = ImageDraw.Draw(img)
    groove, lit = rgb(base, 0.5), rgb(base, 1.25)

    # The two square screw recesses on the band, each with its screw.
    for sx in (142, 578):
        d.rounded_rectangle([sx - 11, 22, sx + 11, 44], radius=3, fill=rgb(base, 0.72))
        d.line([sx - 10, 44, sx + 10, 44], fill=lit)
        d.line([sx + 11, 23, sx + 11, 43], fill=lit)
        d.ellipse([sx - 6, 27, sx + 6, 39], fill=rgb(base, 0.5))
        d.ellipse([sx - 5, 28, sx + 5, 38], fill=rgb(base, 0.85))
        d.line([sx - 3, 33, sx + 3, 33], fill=rgb(base, 0.4), width=2)
        d.line([sx, 30, sx, 36], fill=rgb(base, 0.4), width=2)

    # The mark beside the slot: a little card with an arrow pressing it in.
    mx, my = 500, 30
    d.rounded_rectangle([mx, my, mx + 12, my + 13], radius=1, outline=groove, width=2)
    d.line([mx + 1, my + 14, mx + 13, my + 14], fill=lit)
    d.line([mx + 6, my - 9, mx + 6, my - 3], fill=groove, width=2)
    d.line([mx + 2, my - 6, mx + 6, my - 2, mx + 10, my - 6], fill=groove, width=2)
    return open_mouth(img)


def plugged(base):
    """A game card in the slot: the front over the back, the slot holding the
    top edge of a card."""
    b = np.array(back(base)).astype(float)
    m = mouth()
    card = np.array(to_image(np.full((H, W), 1.0) * grain(22, 0.02), CARD)).astype(float)
    ys = np.arange(H)[:, None]
    xs = np.arange(W)[None, :]
    cx = W // 2
    # The card is a little narrower than the slot: dark either side.
    within = np.broadcast_to((np.abs(xs - cx) <= CARD_W // 2), (H, W))
    # A bevel along the card's top edge, then its label below.
    card[:3] *= 1.35
    card[3:5] *= 0.8
    card[14:, cx - 67:cx + 68, :3] = (210, 214, 220)
    card[14:15, cx - 67:cx + 68, :3] *= 0.7
    inner = m & within
    b[inner] = card[inner]
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
    back(base).save(out / 'slot_nds_back.png', optimize=True)
    front(base).save(out / 'slot_nds_front.png', optimize=True)
    plugged(base).save(out / 'slot_nds_plugged.png', optimize=True)
    print(f'slot_nds_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
