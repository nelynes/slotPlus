"""What every system's cart slot strip is drawn with: the strip's size, Slot's
mouth, the moulded plastic's grain, and embossing."""

from pathlib import Path

import numpy as np
from PIL import Image, ImageFont

ROOT = Path(__file__).resolve().parent.parent.parent
W, H = 720, 58
FONT = ROOT / 'crates/slot-ui/assets/label.ttf'


def colour(name, shells):
    if name in shells:
        return shells[name]
    name = name.lstrip('#')
    return tuple(int(name[i:i + 2], 16) for i in (0, 2, 4))


def grain(seed, amount):
    """The moulded plastic's fine texture, as a multiplier around 1."""
    rng = np.random.default_rng(seed)
    n = rng.normal(0.0, 1.0, (H, W))
    # A little blur so it reads as texture, not noise.
    n = (n + np.roll(n, 1, 0) + np.roll(n, 1, 1) + np.roll(n, -1, 0) + np.roll(n, -1, 1)) / 5
    return 1.0 + n * amount


def mouth():
    """Slot's mouth: True where the front strip is open."""
    a = np.array(Image.open(ROOT / 'assets/slot_front_dark.png').convert('RGBA'))[:, :, 3]
    return a < 128


def near(mask, d):
    """Pixels within d of the mask, outside it."""
    out = np.zeros_like(mask)
    for dy in range(-d, d + 1):
        for dx in range(-d, d + 1):
            out |= np.roll(np.roll(mask, dy, 0), dx, 1)
    return out & ~mask


def shade(rgb, factor):
    return np.clip(np.array(rgb, float) * factor, 0, 255)


def to_image(light, base, alpha=None):
    """light is an H x W multiplier on the shell colour."""
    img = np.clip(light[:, :, None] * np.array(base, float)[None, None, :], 0, 255)
    a = np.full((H, W), 255.0) if alpha is None else alpha
    return Image.fromarray(np.dstack([img, a]).astype(np.uint8), 'RGBA')


def embossed(draw, xy, text, size, base, anchor='mm'):
    font = ImageFont.truetype(str(FONT), size)
    x, y = xy
    draw.text((x + 1, y + 1), text, font=font, fill=tuple(int(c) for c in shade(base, 1.12)), anchor=anchor)
    draw.text((x, y), text, font=font, fill=tuple(int(c) for c in shade(base, 0.72)), anchor=anchor)


def rgb(base, factor):
    return tuple(int(c) for c in shade(base, factor))


def open_mouth(img):
    """Keep the opening open whatever was drawn across it."""
    m = mouth()
    out = np.array(img)
    out[:, :, 3] = np.where(m, 0, out[:, :, 3])
    return Image.fromarray(out, 'RGBA')
