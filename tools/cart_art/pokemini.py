#!/usr/bin/env python3
"""Draw the Pokémon mini cart's layers, in the format slotPlus's other moulds use.

    tools/cart_art/pokemini.py [--out DIR]

Writes, all 300x262:
  pm_cart.png      the outline, white
  pm_neutral.png   the grey shell the game's colour is blended onto
  pm_clear.png     the same for a clear shell (the Pokémon mini's usual frosted plastic)
  pm_interior.png  the back half's inside
  pm_board.png     the board, seen through a clear shell
  pm_decal.png     the hood's lip and its shadow, drawn over the label

The cart is wider than it is tall. A raised hood runs across the top, with a screw post
at each corner and a lip that dips in the middle where it overhangs the label. The
label covers the lower half, and a small arrow below it points the way the cart goes
in. No maker's names or marks.
"""

import argparse
import math
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[2]
W, H = 300, 262
SS = 4
GREY = 120
# The label, as the code's PM_LABEL_* constants: x, y, w, h. About 1.82:1, as the labels are.
LABEL = (26, 104, 248, 136)
LABEL_TOP_R, LABEL_BOTTOM_R = 6, 26
TOP_R, BOTTOM_R = 10, 42
# The step on the right edge, below the hood.
NOTCH = (98, 126, 5)


def lip_y(x):
    """The hood's front edge: level at the sides, dipping in a shallow curve in the middle."""
    centre = W / 2
    d = abs(x - centre) / 100.0
    dip = 9.0 * math.cos(min(d, 1.0) * math.pi / 2) ** 2 if d < 1 else 0.0
    return 90.0 + dip


def outline(scale=SS):
    """The cart's silhouette, at `scale` times the size."""
    w, h = W * scale, H * scale
    m = Image.new('L', (w, h), 0)
    d = ImageDraw.Draw(m)
    tr, br = TOP_R * scale, BOTTOM_R * scale
    d.rounded_rectangle([0, 0, w - 1, h // 2 + tr], radius=tr, fill=255)
    d.rounded_rectangle([0, h // 2 - br, w - 1, h - 1], radius=br, fill=255)
    d.rectangle([0, tr, w - 1, h - br], fill=255)
    y0, y1, inset = (v * scale for v in NOTCH)
    d.rectangle([w - inset, y0, w, y1], fill=0)
    return m


def mask():
    return outline().resize((W, H), Image.Resampling.LANCZOS)


def label_shape(scale=SS):
    m = Image.new('L', (W * scale, H * scale), 0)
    d = ImageDraw.Draw(m)
    x, y, w, h = (v * scale for v in LABEL)
    d.rounded_rectangle([x, y, x + w - 1, y + h - 1], radius=LABEL_BOTTOM_R * scale, fill=255)
    d.rectangle([x, y, x + w - 1, y + LABEL_BOTTOM_R * scale], fill=0)
    d.rounded_rectangle([x, y, x + w - 1, y + 2 * LABEL_BOTTOM_R * scale], radius=LABEL_TOP_R * scale, fill=255)
    return m


def hood_shape(scale=SS):
    m = Image.new('L', (W * scale, H * scale), 0)
    d = ImageDraw.Draw(m)
    pts = [(0, 0), (W * scale, 0)]
    pts += [(x * scale, lip_y(x) * scale) for x in np.linspace(W, 0, 120)]
    d.polygon(pts, fill=255)
    return m


def grain(seed, amount):
    rng = np.random.default_rng(seed)
    g = rng.normal(0.0, amount, (H, W))
    return np.asarray(
        Image.fromarray(np.clip(g * 255 + 128, 0, 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(0.6)),
        np.float32,
    ) / 255 - 0.5


def small(m):
    return np.asarray(m.resize((W, H), Image.Resampling.LANCZOS), np.float32) / 255


def body(base, seed):
    ys = np.linspace(0, 1, H)[:, None]
    xs = np.linspace(-1, 1, W)[None, :]
    v = base * (1.0 - 0.06 * ys - 0.03 * xs ** 4) + grain(seed, 0.012) * 18
    hood = small(hood_shape())
    # The hood stands proud: a little lighter, lit along its top.
    v = v * (1 + 0.07 * hood) + hood * 14 * np.exp(-((ys * H - 6) / 8) ** 2)
    # The lip: a bright edge along the hood's front, a shadow just below it.
    for x in range(W):
        y = lip_y(x)
        for dy, k in [(-2, 0.10), (-1, 0.16), (1, -0.20), (2, -0.14), (3, -0.08), (4, -0.04)]:
            yy = int(round(y + dy))
            if 0 <= yy < H:
                v[yy, x] *= 1 + k
    # The edges of the shell catch the light at the top and fall away at the bottom.
    edge = small(outline()) - small(outline().filter(ImageFilter.MinFilter(13)))
    v = v * (1 + 0.14 * edge * (1 - ys) - 0.18 * edge * ys)
    img = Image.fromarray(np.clip(v, 0, 255).astype(np.uint8)).convert('RGB')
    d = ImageDraw.Draw(img)
    # The screw posts at the hood's corners.
    for x in (36, W - 36):
        y = 34
        d.ellipse([x - 11, y - 11, x + 11, y + 11], fill=(base + 10,) * 3, outline=(base - 26,) * 3, width=2)
        d.ellipse([x - 4, y - 4, x + 4, y + 4], fill=(base - 34,) * 3)
    # The label well: flat, the label's own shape, a hair darker.
    well = small(label_shape())
    arr = np.asarray(img, np.float32)
    arr = arr * (1 - 0.05 * well[..., None])
    img = Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8))
    d = ImageDraw.Draw(img)
    # The arrow below the label, pointing the way the cart goes in.
    cx, cy = W // 2, 250
    d.polygon([(cx - 8, cy - 4), (cx + 8, cy - 4), (cx, cy + 4)], outline=(base - 30,) * 3, fill=(base - 8,) * 3)
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
    # A soft gloss across the hood, as the frosted plastic catches the light.
    d.polygon([(10, 60), (70, 0), (120, 0), (40, 86)], fill=(255, 255, 255, 34))
    return with_alpha(img, mask())


def interior():
    ys = np.linspace(0, 1, H)[:, None]
    v = (GREY + 2) * (1 - 0.05 * ys) + grain(3, 0.012) * 16
    img = Image.fromarray(np.clip(v, 0, 255).astype(np.uint8)).convert('RGB')
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([9, 9, W - 10, H - 10], radius=BOTTOM_R - 8, outline=(GREY + 18,) * 3, width=3)
    for x, y in [(36, 34), (W - 36, 34)]:
        d.ellipse([x - 10, y - 10, x + 10, y + 10], fill=(GREY + 12,) * 3, outline=(GREY - 22,) * 3)
        d.ellipse([x - 4, y - 4, x + 4, y + 4], fill=(GREY - 30,) * 3)
    return with_alpha(img, mask())


def board():
    """The board as the real ones are: bright green, a half-round cut at each top corner
    round the screw posts, one wide flat chip across the top, a few small parts, and two
    staggered rows of contacts along the bottom edge."""
    big = Image.new('RGBA', (W * SS, H * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(big)
    S = SS
    bx0, by0, bx1, by1 = 30, 14, W - 30, 224
    green, trace, silk = (0x2e, 0x9c, 0x46, 255), (0x52, 0xb8, 0x5e, 255), (0xee, 0xf2, 0xe8, 255)
    d.rectangle([bx0 * S, by0 * S, bx1 * S, by1 * S], fill=green)
    # The cuts round the screw posts, and the small notch in the top edge.
    for cx in (bx0, bx1):
        d.ellipse([(cx - 17) * S, (36 - 17) * S, (cx + 17) * S, (36 + 17) * S], fill=(0, 0, 0, 0))
    d.rectangle([(W // 2 - 8) * S, by0 * S, (W // 2 + 8) * S, (by0 + 5) * S], fill=(0, 0, 0, 0))
    # Traces fanning down from the chip to the contacts.
    rng = np.random.default_rng(7)
    for i in range(16):
        x0 = W // 2 - 60 + i * 8
        x1 = bx0 + 22 + i * ((bx1 - bx0 - 44) / 15)
        d.line([x0 * S, 70 * S, x0 * S, 110 * S, x1 * S, 150 * S, x1 * S, 168 * S], fill=trace, width=S)
        d.ellipse([(x1 - 3) * S, (166 - 3) * S, (x1 + 3) * S, (166 + 3) * S], fill=(0xd0, 0xc8, 0x90, 255))
    # The chip: a wide flat package with legs along its long sides.
    cx, cy, cw, ch = W // 2, 46, 64, 20
    for i in range(26):
        x = cx - cw + 4 + i * (2 * cw - 8) / 25
        for side in (-1, 1):
            y0 = cy + side * ch
            d.line([x * S, y0 * S, x * S, (y0 + side * 6) * S], fill=(0xc8, 0xc8, 0xc0, 255), width=S)
    d.rectangle([(cx - cw) * S, (cy - ch) * S, (cx + cw) * S, (cy + ch) * S], fill=(0x1c, 0x1d, 0x22, 255))
    d.ellipse([(cx - cw + 6) * S, (cy + ch - 12) * S, (cx - cw + 12) * S, (cy + ch - 6) * S], fill=(0x30, 0x31, 0x38, 255))
    # Small parts in white outlines: two beside the chip, a pair lower left, two on the right.
    for x, y, w, h in [(70, 66, 12, 8), (W - 82, 60, 12, 8), (64, 100, 8, 14), (78, 100, 8, 14),
                       (130, 98, 10, 14), (W - 70, 98, 10, 14), (172, 98, 30, 14)]:
        d.rectangle([(x - 2) * S, (y - 2) * S, (x + w + 2) * S, (y + h + 2) * S], outline=silk, width=S)
        d.rectangle([x * S, y * S, (x + w) * S, (y + h) * S], fill=(0x3a, 0x34, 0x2c, 255))
    # Two staggered rows of gold contacts, the upper longer, along the bottom edge.
    n = 13
    pitch = (bx1 - bx0 - 20) / n
    for i in range(n):
        x = bx0 + 10 + pitch * (i + 0.5)
        d.rounded_rectangle([(x - 4) * S, 172 * S, (x + 4) * S, 196 * S], radius=S, fill=(0xd8, 0xa8, 0x3c, 255))
        if i < n - 1:
            x2 = x + pitch / 2
            d.rounded_rectangle([(x2 - 4) * S, 202 * S, (x2 + 4) * S, 220 * S], radius=S, fill=(0xd8, 0xa8, 0x3c, 255))
    return big.resize((W, H), Image.Resampling.LANCZOS)


def decal():
    """Over the label: the hood's lip and the soft shadow it casts on the label's top edge."""
    img = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    a = np.zeros((H, W), np.float32)
    lx, ly, lw, lh = LABEL
    for x in range(lx, lx + lw):
        y = lip_y(x)
        for yy in range(int(y), int(y) + 10):
            if yy >= ly:
                a[yy, x] = max(a[yy, x], 0.38 * math.exp(-(yy - y) / 3.0))
    rgba = np.zeros((H, W, 4), np.uint8)
    rgba[..., 3] = np.clip(a * 255, 0, 255).astype(np.uint8)
    return Image.fromarray(rgba, 'RGBA')


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
    decal().save(out / 'pm_decal.png', optimize=True)
    print(f'pm_cart/neutral/clear/interior/board/decal.png in {out}')


if __name__ == '__main__':
    main()
