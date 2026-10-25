#!/usr/bin/env python3
"""Draw the WonderSwan's cart slot for Cart Slot set to Bespoke.

    tools/slot_art/wonderswan.py [colour] [--out DIR]

The three strips are the size of Slot's (720x58) with its mouth cut out in the
same place, taken from assets/slot_front_dark.png, so a cart goes in the same
way. They draw the back of a WonderSwan: the cartridge bay in the middle with
a clip slot down each inner wall and the embossed plate on its floor,
the raised plastic either side with its screws, and the top of the silver
label below. No maker's names or marks are drawn.

colour is one of the WonderSwan's shells (pearl, wine, crystal, black), or a
hex RGB.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

from strip import FONT, H, ROOT, W, colour, embossed, grain, mouth, near, shade, to_image

SHELLS = {
    'pearl': (226, 223, 214),
    'wine': (128, 28, 44),
    'black': (46, 46, 50),
    'crystal': (196, 200, 204),
}


def screw(draw, cx, cy, r, base):
    rim = tuple(int(c) for c in shade(base, 0.55))
    head = (150, 150, 152)
    draw.ellipse([cx - r - 1, cy - r - 1, cx + r + 1, cy + r + 1], fill=rim)
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=head)
    draw.ellipse([cx - r + 1, cy - r + 1, cx + r - 2, cy + r - 2], fill=(178, 178, 180))
    slot = (70, 70, 74)
    draw.line([cx - r + 2, cy, cx + r - 2, cy], fill=slot, width=1)
    draw.line([cx, cy - r + 2, cx, cy + r - 2], fill=slot, width=1)


def back(base):
    """Seen through the mouth: the floor of the cartridge bay."""
    light = np.full((H, W), 0.84) * grain(1, 0.02)
    # Deeper towards the top, where the bay runs under the raised plastic.
    ys = np.arange(H)[:, None]
    light *= 0.88 + 0.12 * np.clip(ys / 20, 0, 1)
    m = mouth()
    xs = np.where(m[2])[0]
    left, right = xs.min(), xs.max()
    # The bay's side walls, in shadow.
    for i in range(12):
        f = 0.70 + 0.14 * i / 12
        light[:, left + i] *= f / 0.84
        light[:, right - i] *= f / 0.84
    img = to_image(light, base)
    d = ImageDraw.Draw(img)
    # A clip slot down each inner wall.
    for x in (left + 16, right - 22):
        d.rounded_rectangle([x, 4, x + 6, 26], radius=2, fill=tuple(int(c) for c in shade(base, 0.5)))
        d.line([x + 1, 5, x + 1, 25], fill=tuple(int(c) for c in shade(base, 0.38)))
    # The embossed plate on the bay floor.
    cx = (left + right) // 2
    plate = [cx - 92, 9, cx + 92, 23]
    d.rounded_rectangle(plate, radius=7, outline=tuple(int(c) for c in shade(base, 0.68)), width=1)
    d.rounded_rectangle([p + q for p, q in zip(plate, (1, 1, 1, 1))], radius=7,
                        outline=tuple(int(c) for c in shade(base, 1.0)), width=1)
    # Moulded ribs where the real plate carries its maker's text.
    for x in range(cx - 80, cx + 81, 8):
        d.line([x, 13, x, 19], fill=tuple(int(c) for c in shade(base, 0.7)))
        d.line([x + 1, 13, x + 1, 19], fill=tuple(int(c) for c in shade(base, 0.95)))
    return img


def front(base):
    """The raised plastic around the bay, open where the cart goes in."""
    m = mouth()
    light = np.full((H, W), 1.0) * grain(2, 0.025)
    # Lit from above: a bright top edge and a soft fall-off.
    light[0, :] *= 1.18
    light[1, :] *= 1.08
    ys = np.arange(H)[:, None]
    light *= 1.02 - 0.06 * ys / H
    # The bay's rim: shadowed just outside the opening, a highlight past it.
    light[near(m, 2)] *= 0.72
    light[near(m, 3) & ~near(m, 2)] *= 1.08
    # The seams where the raised sides meet the middle panel.
    for x in (198, 521):
        light[:, x] *= 0.78
        light[:, x + 1] *= 1.1
    img = to_image(light, base, np.where(m, 0.0, 255.0))
    d = ImageDraw.Draw(img)
    # The battery and clock sit on the ends of the bottom half, so the screws
    # and marks keep to the top and the seams.
    for cx, cy in ((180, 15), (540, 15), (180, 44), (540, 44)):
        screw(d, cx, cy, 6, base)
    # The top of the silver label, under the bay.
    label = [258, 47, 462, 60]
    d.rectangle(label, fill=(196, 198, 202))
    d.line([258, 47, 462, 47], fill=(232, 233, 236))
    d.line([258, 47, 258, 58], fill=(150, 152, 156))
    d.line([462, 47, 462, 58], fill=(150, 152, 156))
    # Fine print as grey rules, no maker's name or marks.
    for x0, x1 in ((272, 330), (342, 400), (412, 448)):
        d.line([x0, 52, x1, 52], fill=(120, 122, 126))
        d.line([x0, 55, x1 - 14, 55], fill=(140, 142, 146))
    # The EXT. and CONTRAST marks on the ends.
    embossed(d, (24, 9), 'EXT.', 8, base, 'lm')
    embossed(d, (696, 9), 'CONTRAST', 8, base, 'rm')
    # Keep the opening open whatever was drawn across it.
    out = np.array(img)
    out[:, :, 3] = np.where(m, 0, out[:, :, 3])
    return Image.fromarray(out, 'RGBA')


def plugged(base):
    """A cart in the bay: the front over the back, the bay holding the top of a
    black WonderSwan cart."""
    b = np.array(back(base)).astype(float)
    m = mouth()
    cart = np.array(to_image(np.full((H, W), 1.0) * grain(3, 0.03), (34, 34, 38))).astype(float)
    ys = np.arange(H)[:, None]
    ridged = (ys % 4 == 0) & (ys < 24)
    cart[ridged[:, :, None].repeat(4, 2) & np.ones((H, W, 4), bool)] *= 0.7
    inner = m.copy()
    xs = np.where(m[2])[0]
    inner[:, :xs.min() + 10] = False
    inner[:, xs.max() - 9:] = False
    b[inner] = cart[inner]
    b[:, :, 3] = 255
    f = front(base)
    out = Image.fromarray(b.astype(np.uint8), 'RGBA')
    out.alpha_composite(f)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('colour', nargs='?', default='pearl')
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    a = ap.parse_args()
    base = colour(a.colour, SHELLS)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    back(base).save(out / 'slot_ws_back.png', optimize=True)
    front(base).save(out / 'slot_ws_front.png', optimize=True)
    plugged(base).save(out / 'slot_ws_plugged.png', optimize=True)
    print(f'slot_ws_back/front/plugged.png in {out}, colour {base}')


if __name__ == '__main__':
    main()
