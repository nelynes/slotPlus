#!/usr/bin/env python3
"""Draw the bezels slotPlus bundles for its own systems, in the style of Slot's
Game Boy ones: a 720x480 face with a clear window over the 3x picture, a soft
shadow just inside the window's edge, and the handheld's power light.

For each system it writes, into the repository's `assets/`:

    bezel_<system>.png   the face, its window transparent

usage: python3 assets/src/bezels.py [gg ...]
"""

import os
import sys

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.dirname(HERE)

W, H = 720, 480
S = 4  # supersampling


def down(a):
    h, w = a.shape[0] // S, a.shape[1] // S
    return a.reshape(h, S, w, S, *a.shape[2:]).mean(axis=(1, 3))


def window_alpha(x0, y0, x1, y1, radius=3, shadow=6):
    """1 over the face, 0 in the window, with the Game Boy bezels' soft
    shadow falling `shadow` pixels in from the window's edge."""
    m = Image.new("L", (W * S, H * S), 0)
    ImageDraw.Draw(m).rounded_rectangle(
        [x0 * S, y0 * S, x1 * S - 1, y1 * S - 1], radius=radius * S, fill=255)
    inside = down(np.asarray(m, np.float32) / 255.0)
    dist = ndimage.distance_transform_edt(inside > 0.5)
    fall = np.clip(1.0 - dist / shadow, 0.0, 1.0) ** 1.6 * 0.55
    return np.where(inside > 0.5, fall, 1.0 - inside * 0.45)


def grain(seed, std):
    rng = np.random.default_rng(seed)
    n = ndimage.gaussian_filter(rng.normal(0.0, 1.0, (H, W)), 0.6)
    return n / n.std() * std


def glow(cx, cy, rx, ry):
    ys, xs = np.mgrid[0:H, 0:W]
    return np.exp(-(((xs - cx) / rx) ** 2 + ((ys - cy) / ry) ** 2))


def game_gear():
    # The picture sits at 3x in the middle of the screen: 480x432 from
    # (120, 24). The face is the Game Gear's black plastic.
    x0, y0, x1, y1 = 120, 24, 600, 456
    base = np.full((H, W), 9.0) + grain(51, 1.6)
    rgb = np.dstack([base, base, base + 1.0])

    # The power light, centred above the screen: a small bright green lamp
    # in a wide soft glow that spills along the top edge.
    cx, cy = W / 2, 11.0
    halo = glow(cx, cy, 110.0, 9.0) * 0.55 + glow(cx, cy, 46.0, 5.0) * 0.9
    lamp = np.clip(glow(cx, cy, 22.0, 2.6) * 1.6, 0, 1)
    green = np.array([40.0, 230.0, 70.0])
    core = np.array([200.0, 255.0, 200.0])
    rgb = rgb + halo[..., None] * green * 0.6
    rgb = rgb * (1 - lamp[..., None]) + core * lamp[..., None]

    alpha = window_alpha(x0, y0, x1, y1)
    out = np.dstack([np.clip(rgb, 0, 255), alpha * 255.0])
    Image.fromarray(out.round().astype(np.uint8), "RGBA").save(
        os.path.join(ASSETS, "bezel_gg.png"), optimize=True)


SYSTEMS = {"gg": game_gear}

if __name__ == "__main__":
    for name in sys.argv[1:] or SYSTEMS:
        SYSTEMS[name]()
        print("drew bezel", name)
