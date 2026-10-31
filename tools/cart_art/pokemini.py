#!/usr/bin/env python3
"""Draw the Pokémon mini cart's layers, in the format slotPlus's other moulds use.

    tools/cart_art/pokemini.py [--out DIR]

Writes pm_cart.png (the outline, white), pm_neutral.png (the grey shell the game's
colour is blended onto), pm_clear.png (the same for a clear shell, with a gloss),
pm_interior.png (the back half's inside) and pm_board.png (the board), all 264x280.

The cart is a small, nearly square block: a grip band of ridges across the top, a
label that covers most of the face, and rounded corners. No maker's names or marks.
"""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[2]
W, H = 264, 280
SS = 4  # supersampling
RADIUS = 26
GREY = 120
# The label well, as the code's PM_LABEL_* constants: x, y, w, h.
LABEL = (24, 66, 216, 196)
GRIP = (34, 12, W - 34, 52)


def mask(radius=RADIUS, inset=0):
    m = Image.new('L', (W * SS, H * SS), 0)
    d = ImageDraw.Draw(m)
    i = inset * SS
    d.rounded_rectangle([i, i, W * SS - 1 - i, H * SS - 1 - i], radius=radius * SS, fill=255)
    return m.resize((W, H), Image.Resampling.LANCZOS)


def grain(seed, amount):
    rng = np.random.default_rng(seed)
    g = rng.normal(0.0, amount, (H, W))
    return Image.fromarray(np.clip(g * 255 + 128, 0, 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(0.6))


def shade(base, seed=1):
    """A flat grey with a soft top light, darker toward the bottom and the sides."""
    ys = np.linspace(0, 1, H)[:, None]
    xs = np.linspace(-1, 1, W)[None, :]
    light = 1.0 + 0.06 * np.exp(-((ys - 0.05) / 0.12) ** 2) - 0.05 * ys - 0.03 * xs ** 4
    g = np.asarray(grain(seed, 0.012), np.float32) / 255 - 0.5
    v = np.clip(base * light + g * 18, 0, 255)
    return v


def body(base, seed):
    v = shade(base, seed)
    img = Image.fromarray(v.astype(np.uint8)).convert('RGB')
    d = ImageDraw.Draw(img)
    # Bevel: a lighter rim inside the edge, a darker one at the bottom.
    edge = np.asarray(mask()).astype(np.float32) - np.asarray(mask(RADIUS - 3, 3)).astype(np.float32)
    rim = Image.fromarray(np.clip(edge, 0, 255).astype(np.uint8))
    top = Image.new('RGB', (W, H), (base + 22,) * 3)
    bottom = Image.new('RGB', (W, H), (base - 26,) * 3)
    upper = Image.fromarray((np.linspace(1, 0, H)[:, None] * np.ones((1, W)) * 255).astype(np.uint8))
    img = Image.composite(top, img, Image.fromarray((np.asarray(rim) * np.asarray(upper) / 255).astype(np.uint8)))
    lower = Image.fromarray((np.linspace(0, 1, H)[:, None] * np.ones((1, W)) * 255).astype(np.uint8))
    img = Image.composite(bottom, img, Image.fromarray((np.asarray(rim) * np.asarray(lower) / 255).astype(np.uint8)))
    d = ImageDraw.Draw(img)
    # The grip: ridges across the top, each lit above and shadowed below.
    x0, y0, x1, y1 = GRIP
    for i in range(6):
        y = y0 + 4 + i * 6.5
        d.line([x0, y, x1, y], fill=(base - 30,) * 3, width=2)
        d.line([x0, y + 2, x1, y + 2], fill=(base + 20,) * 3, width=1)
    # The label well: a shallow recess, its upper edge in shadow and its lower lit.
    lx, ly, lw, lh = LABEL
    d.rounded_rectangle([lx - 2, ly - 2, lx + lw + 1, ly + lh + 1], radius=12, outline=(base - 34,) * 3, width=2)
    d.line([lx + 10, ly + lh + 2, lx + lw - 10, ly + lh + 2], fill=(base + 18,) * 3, width=1)
    d.rounded_rectangle([lx, ly, lx + lw - 1, ly + lh - 1], radius=10, fill=(base - 6,) * 3)
    # A small arrow on the grip band's right, pointing the way the cart goes in.
    ax, ay = W - 26, 30
    d.polygon([(ax - 5, ay - 4), (ax + 5, ay - 4), (ax, ay + 4)], fill=(base - 28,) * 3)
    return img


def with_alpha(img, alpha):
    out = img.convert('RGBA')
    out.putalpha(alpha)
    return out


def neutral():
    return with_alpha(body(GREY, 1), mask())


def clear():
    img = body(GREY + 4, 2)
    d = ImageDraw.Draw(img, 'RGBA')
    # A diagonal gloss across the upper left, as the other clear moulds have.
    d.polygon([(0, 70), (70, 0), (110, 0), (0, 110)], fill=(255, 255, 255, 40))
    return with_alpha(img, mask())


def interior():
    v = shade(GREY + 2, 3)
    img = Image.fromarray(v.astype(np.uint8)).convert('RGB')
    d = ImageDraw.Draw(img)
    # The tub: a raised wall round the inside, and two screw posts.
    d.rounded_rectangle([10, 10, W - 11, H - 11], radius=RADIUS - 8, outline=(GREY + 18,) * 3, width=3)
    for x, y in [(W // 2, 34), (W // 2, H - 30)]:
        d.ellipse([x - 9, y - 9, x + 9, y + 9], fill=(GREY + 12,) * 3, outline=(GREY - 22,) * 3)
        d.ellipse([x - 3, y - 3, x + 3, y + 3], fill=(GREY - 30,) * 3)
    return with_alpha(img, mask())


def board():
    img = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    green = (0x1f, 0x5a, 0x3a)
    bx0, by0, bx1, by1 = 30, 22, W - 30, H - 26
    d.rounded_rectangle([bx0, by0, bx1, by1], radius=6, fill=green + (255,))
    # Traces.
    rng = np.random.default_rng(7)
    for _ in range(22):
        x = int(rng.integers(bx0 + 10, bx1 - 10))
        y = int(rng.integers(by0 + 40, by1 - 10))
        dx = int(rng.integers(-40, 40))
        d.line([x, y, x + dx, y, x + dx, y - int(rng.integers(10, 40))], fill=(0x2c, 0x7a, 0x4e, 255), width=2)
    # The one chip, and the gold contacts along the top edge, where the cart goes in.
    cx, cy = W // 2, H // 2 + 20
    d.rectangle([cx - 46, cy - 46, cx + 46, cy + 46], fill=(0x1a, 0x1c, 0x22, 255))
    for i in range(10):
        for side in (-1, 1):
            y = cy - 40 + i * 9
            d.line([cx + side * 46, y, cx + side * 52, y], fill=(0xb8, 0xb8, 0xb0, 255), width=2)
    for i in range(14):
        x = bx0 + 14 + i * ((bx1 - bx0 - 28) / 13)
        d.rectangle([x - 4, by0 + 4, x + 4, by0 + 26], fill=(0xd8, 0xa8, 0x3c, 255))
    return img


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--out', default=str(ROOT / 'assets'))
    out = Path(ap.parse_args().out)
    out.mkdir(parents=True, exist_ok=True)
    white = Image.new('RGBA', (W, H), (255, 255, 255, 255))
    white.putalpha(mask())
    white.save(out / 'pm_cart.png', optimize=True)
    neutral().save(out / 'pm_neutral.png', optimize=True)
    clear().save(out / 'pm_clear.png', optimize=True)
    interior().save(out / 'pm_interior.png', optimize=True)
    board().save(out / 'pm_board.png', optimize=True)
    print(f'pm_cart/neutral/clear/interior/board.png in {out}')


if __name__ == '__main__':
    main()
