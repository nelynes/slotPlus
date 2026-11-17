#!/usr/bin/env python3
"""Draw the slotPlus cart moulds in the style of Slot's hand drawn ones.

Slot's GBA and Game Boy carts are hand drawn as 50% grey plastic that the game's
shell colour is blended onto. This draws the moulds slotPlus adds the same way:
a height map of the plastic is lit from the top left and given a fine stipple,
the label well is left flat grey, and anything that must keep its own colour
(the gold contacts) goes in a separate decal that is drawn on top untinted.

For each mould it writes, into the repository's `assets/`:

    <mould>_cart.png      the outline: white, its alpha the cart's silhouette
    <mould>_neutral.png   the opaque shell
    <mould>_clear.png     the see-through shell, with a rim and a sheen
    <mould>_interior.png  the back plastic a clear cart shows
    <mould>_board.png     the board a clear cart shows
    <mould>_decal.png     drawn over the shell untinted (contacts, PCB)

usage: python3 assets/src/carts.py [ws|lynx|gg|ngp|tg16|nds ...]
"""

import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont
from scipy import ndimage

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.dirname(HERE)
FONT = os.path.join(ASSETS, "..", "crates", "slot-ui", "assets", "label.ttf")

S = 4  # supersampling
NEUTRAL = 128.0
WELL = 119
LIGHT = np.array([-0.62, -0.78])  # from the top left


class Canvas:
    def __init__(self, w, h):
        self.w, self.h = w, h
        self.W, self.H = w * S, h * S

    def layer(self):
        return Image.new("L", (self.W, self.H), 0)

    def rrect(self, img, x0, y0, x1, y1, r, fill=255, corners=None):
        d = ImageDraw.Draw(img)
        box = [x0 * S, y0 * S, x1 * S - 1, y1 * S - 1]
        if corners is None:
            d.rounded_rectangle(box, radius=r * S, fill=fill)
        else:
            d.rounded_rectangle(box, radius=r * S, fill=fill, corners=corners)
        return img

    def poly(self, img, pts, fill=255):
        ImageDraw.Draw(img).polygon([(x * S, y * S) for x, y in pts], fill=fill)
        return img

    def ellipse(self, img, x0, y0, x1, y1, fill=255, width=0):
        d = ImageDraw.Draw(img)
        if width:
            d.ellipse([x0 * S, y0 * S, x1 * S, y1 * S], outline=fill, width=width * S)
        else:
            d.ellipse([x0 * S, y0 * S, x1 * S, y1 * S], fill=fill)
        return img

    def text(self, img, cx, cy, words, px, italic=0.0, weight=0):
        font = ImageFont.truetype(FONT, int(px * S))
        d = ImageDraw.Draw(img)
        l, t, r, b = d.textbbox((0, 0), words, font=font, stroke_width=int(weight * S))
        tw, th = r - l, b - t
        tile = Image.new("L", (tw + 8 * S, th + 8 * S), 0)
        ImageDraw.Draw(tile).text(
            (4 * S - l, 4 * S - t), words, font=font, fill=255,
            stroke_width=int(weight * S), stroke_fill=255,
        )
        if italic:
            tile = tile.transform(
                tile.size, Image.AFFINE, (1, italic, -italic * tile.size[1] / 2, 0, 1, 0),
                resample=Image.BICUBIC,
            )
        img.paste(tile, (int(cx * S - tile.size[0] / 2), int(cy * S - tile.size[1] / 2)), tile)
        return img


def f(img):
    return np.asarray(img, dtype=np.float32) / 255.0


def soft(mask, px):
    return ndimage.gaussian_filter(mask, px * S) if px else mask


def down(a):
    """Box-filter a supersampled array back to 1x."""
    h, w = a.shape[0] // S, a.shape[1] // S
    return a.reshape(h, S, w, S, *a.shape[2:]).mean(axis=(1, 3))


def bevel(mask, width):
    """A rounded shoulder rising `width` pixels in from the mask's edge."""
    dist = ndimage.distance_transform_edt(mask > 0.5) / S
    t = np.clip(dist / width, 0.0, 1.0)
    return np.sin(t * np.pi / 2)


def light(height, gain):
    gy, gx = np.gradient(height, 1.0 / S)
    return (gx * LIGHT[0] + gy * LIGHT[1]) * gain


def grain(shape, seed, std=5.0):
    rng = np.random.default_rng(seed)
    n = rng.normal(0.0, std, (shape[0] // S, shape[1] // S)).astype(np.float32)
    n = ndimage.gaussian_filter(n, 0.42)
    n *= std / max(n.std(), 1e-6)
    return np.kron(n, np.ones((S, S), dtype=np.float32))


def rim(body):
    """The moulded edge the hand drawn carts have: two pixels bright where the
    edge faces up, a little dark down the sides and dark along the bottom."""
    dist = ndimage.distance_transform_edt(body > 0.5) / S
    band = np.clip(2.2 - dist, 0, 1)
    smooth = ndimage.gaussian_filter(body, 2.0 * S)
    gy, gx = np.gradient(smooth)
    norm = np.hypot(gx, gy) + 1e-6
    nx, ny = -gx / norm, -gy / norm  # outward
    up = np.clip(-ny, 0, 1)
    down = np.clip(ny, 0, 1)
    return band * (60 * up - 25 * down - 20 * np.abs(nx))


def shade(height, body, well, seed, gain=26.0, occlusion=None, smooth=None):
    """Light the plastic, stipple it, and leave the label well flat. Where
    `smooth` is set the stipple fades out, as on a polished recess floor."""
    g = NEUTRAL + light(height, gain)
    if occlusion is not None:
        g -= occlusion
    g += rim(body)
    stipple = grain(height.shape, seed)
    if smooth is not None:
        stipple *= 1.0 - 0.85 * smooth
    g += stipple
    g = np.where(well > 0.5, WELL, g)
    return np.clip(g, 0, 255) * body


def rgba(grey, alpha):
    g = down(grey / np.maximum(alpha, 1e-3) * (alpha > 0))
    a = down(alpha)
    out = np.dstack([g, g, g, a * 255.0])
    return Image.fromarray(np.clip(out, 0, 255).round().astype(np.uint8), "RGBA")


def rgba_colour(rgb, alpha):
    a = down(alpha)
    c = down(rgb * alpha[..., None]) / np.maximum(a[..., None], 1e-3)
    out = np.dstack([c, a * 255.0])
    return Image.fromarray(np.clip(out, 0, 255).round().astype(np.uint8), "RGBA")


def clear_from(grey, body, seed):
    """The see-through shell: flatter plastic, a darker rim, a diagonal sheen."""
    h, w = body.shape
    dist = ndimage.distance_transform_edt(body > 0.5) / S
    rim = dist < 5
    ys, xs = np.mgrid[0:h, 0:w] / S
    sheen = np.abs(xs + ys - (w / S) / 3) < (w / S) / 14
    g = NEUTRAL + (grey - NEUTRAL) * 0.55
    g = np.where(sheen, g + 24, g)
    g = np.where(rim, g - 28, g)
    return np.clip(g, 0, 255) * body


def interior(c, body, seed, bosses):
    """The back plastic: flat grey with its moulded screw bosses and ribs."""
    height = bevel(body, 4.0) * 2.0
    for (cx, cy, r) in bosses:
        ring = c.ellipse(c.layer(), cx - r, cy - r, cx + r, cy + r)
        hole = c.ellipse(c.layer(), cx - r * 0.45, cy - r * 0.45, cx + r * 0.45, cy + r * 0.45)
        height += soft(f(ring), 0.6) * 2.0 - soft(f(hole), 0.4) * 3.0
    g = NEUTRAL - 6 + light(height, 24.0) + grain(height.shape, seed, 4.0)
    return rgba(np.clip(g, 0, 255) * body, body)


def pcb(c, region, seed, chips):
    """A green board with traces and chips, inside `region`."""
    h, w = c.H, c.W
    rng = np.random.default_rng(seed)
    base = np.array([31, 92, 58], dtype=np.float32)
    rgb = np.ones((h, w, 3), np.float32) * base
    rgb += grain((h, w), seed + 1, 5.0)[..., None]
    traces = c.layer()
    d = ImageDraw.Draw(traces)
    for _ in range(26):
        x = rng.uniform(0.1, 0.9) * c.w
        y = rng.uniform(0.15, 0.7) * c.h
        pts = [(x, y)]
        for _ in range(3):
            if rng.random() < 0.5:
                x += rng.uniform(-40, 40)
            else:
                y += rng.uniform(-30, 30)
            pts.append((x, y))
        d.line([(px * S, py * S) for px, py in pts], fill=255, width=int(1.2 * S))
    t = f(traces)[..., None]
    rgb = rgb * (1 - t) + np.array([52, 128, 80], np.float32) * t
    for (x0, y0, x1, y1) in chips:
        chip = f(c.rrect(c.layer(), x0, y0, x1, y1, 1.5))[..., None]
        rgb = rgb * (1 - chip) + np.array([22, 26, 34], np.float32) * chip
        leg = c.layer()
        for x in np.arange(x0 + 2, x1 - 1, 3.0):
            c.rrect(leg, x, y0 - 2, x + 1.4, y0, 0.3)
            c.rrect(leg, x, y1, x + 1.4, y1 + 2, 0.3)
        lg = f(leg)[..., None]
        rgb = rgb * (1 - lg) + np.array([196, 196, 190], np.float32) * lg
    return rgb * f(region)[..., None]


def fingers(c, x0, x1, y0, y1, count, rgb, alpha, width=0.55):
    """Gold edge contacts, `count` of them spread from x0 to x1."""
    pitch = (x1 - x0) / count
    gold = c.layer()
    for k in range(count):
        x = x0 + pitch * (k + (1 - width) / 2)
        c.rrect(gold, x, y0, x + pitch * width, y1, 0.4)
    g = f(gold)
    shine = np.linspace(1.08, 0.86, c.H)[:, None]
    colour = np.array([208, 164, 72], np.float32) * shine[..., None]
    rgb = rgb * (1 - g[..., None]) + colour * g[..., None]
    return rgb, np.maximum(alpha, g)


# --------------------------------------------------------------------------
# The WonderSwan board, after Bandai's own: a coin cell in its clip on the left,
# the big ROM in the middle, the mapper square on the right, the save RAM below,
# a little eight-leg chip, a light band of vias over the contacts, and notches in
# its sides where the shell's posts come through. No makers' names on anything.

GREEN = np.array([10, 88, 50], np.float32)
TRACE = np.array([38, 132, 80], np.float32)
SILK = np.array([226, 230, 222], np.float32)
CHIP = np.array([20, 24, 34], np.float32)
LEG = np.array([200, 202, 198], np.float32)
VIA = np.array([184, 148, 58], np.float32)

# Where the ROM's marking goes; `slot_ui::board` writes the game's name there.
WS_ROM = (124, 28, 236, 88)


def paint(rgb, mask, colour, k=1.0):
    m = mask[..., None] * k
    return rgb * (1 - m) + np.asarray(colour, np.float32) * m


def chip_with_legs(c, rgb, x0, y0, x1, y1, sides, pitch, leg=3.5, seed=0):
    """A black chip with silver legs on `sides` (any of "lrtb")."""
    legs = c.layer()
    def row(a0, a1, fixed0, fixed1, along_x):
        for a in np.arange(a0 + pitch * 0.7, a1 - pitch * 0.4, pitch):
            if along_x:
                c.rrect(legs, a, fixed0, a + pitch * 0.5, fixed1, 0.2)
            else:
                c.rrect(legs, fixed0, a, fixed1, a + pitch * 0.5, 0.2)
    if "l" in sides:
        row(y0, y1, x0 - leg, x0 + 0.5, False)
    if "r" in sides:
        row(y0, y1, x1 - 0.5, x1 + leg, False)
    if "t" in sides:
        row(x0, x1, y0 - leg, y0 + 0.5, True)
    if "b" in sides:
        row(x0, x1, y1 - 0.5, y1 + leg, True)
    lg = f(legs)
    shine = np.linspace(1.05, 0.85, c.H)[:, None, None]
    rgb = rgb * (1 - lg[..., None]) + (LEG * shine) * lg[..., None]
    shadow = soft(f(c.rrect(c.layer(), x0 + 1.2, y0 + 1.6, x1 + 1.2, y1 + 1.6, 1.2)), 1.2)
    rgb = rgb * (1 - 0.45 * shadow[..., None])
    body = f(c.rrect(c.layer(), x0, y0, x1, y1, 1.2))
    rgb = rgb * (1 - body[..., None]) + (CHIP + grain(body.shape, seed, 2.0)[..., None]) * body[..., None]
    # A soft sheen along the top edge, and the pin-one dimple.
    top = f(c.rrect(c.layer(), x0 + 1, y0 + 1, x1 - 1, y0 + 2.2, 0.6)) * body
    rgb = paint(rgb, top, [58, 64, 80], 0.6)
    r = min(x1 - x0, y1 - y0) * 0.07 + 1.2
    dimple = f(c.ellipse(c.layer(), x0 + 3, y1 - 3 - 2 * r, x0 + 3 + 2 * r, y1 - 3)) * body
    rgb = paint(rgb, dimple, [10, 12, 18], 0.9)
    return rgb


def silk_text(c, rgb, cx, cy, words, px, weight=0):
    t = f(c.text(c.layer(), cx, cy, words, px, weight=weight))
    shadow = np.roll(np.roll(soft(t, 0.4), S, axis=0), S // 2, axis=1)
    rgb = paint(rgb, shadow, [0, 30, 14], 0.5)
    return paint(rgb, t, SILK)


def silk_mark(c, rgb, right, cy, year_px=11, name_px=13):
    """The boards' maker's mark, "2026 SlotPlus+", set to end at `right`."""
    font = lambda px: ImageFont.truetype(FONT, int(px * S))
    probe = ImageDraw.Draw(Image.new("L", (1, 1)))
    width = lambda words, px, weight: (probe.textlength(words, font=font(px)) + 2 * weight * S) / S
    name_w = width("SlotPlus+", name_px, 0.5)
    year_w = width("2026", year_px, 0)
    gap = 4
    rgb = silk_text(c, rgb, right - name_w / 2, cy, "SlotPlus+", name_px, weight=0.5)
    return silk_text(c, rgb, right - name_w - gap - year_w / 2, cy + 0.5, "2026", year_px)


def silk_box(c, rgb, x0, y0, x1, y1, width=0.6):
    outer = f(c.rrect(c.layer(), x0, y0, x1, y1, 0.8))
    inner = f(c.rrect(c.layer(), x0 + width, y0 + width, x1 - width, y1 - width, 0.5))
    return paint(rgb, np.clip(outer - inner, 0, 1), SILK, 0.9)


def passive(c, rgb, x, y, vertical=False, label=None, label_at=None):
    """A little capacitor or resistor in its silkscreen box."""
    w, h = (4.0, 7.0) if vertical else (7.0, 4.0)
    rgb = silk_box(c, rgb, x - 1.6, y - 1.6, x + w + 1.6, y + h + 1.6)
    body = f(c.rrect(c.layer(), x, y, x + w, y + h, 0.5))
    rgb = paint(rgb, body, [150, 122, 92])
    cap = c.layer()
    if vertical:
        c.rrect(cap, x, y, x + w, y + 1.6, 0.4)
        c.rrect(cap, x, y + h - 1.6, x + w, y + h, 0.4)
    else:
        c.rrect(cap, x, y, x + 1.6, y + h, 0.4)
        c.rrect(cap, x + w - 1.6, y, x + w, y + h, 0.4)
    rgb = paint(rgb, f(cap), [214, 214, 206])
    if label:
        lx, ly = label_at or (x + w / 2, y - 6)
        rgb = silk_text(c, rgb, lx, ly, label, 7)
    return rgb


def vias(c, rgb, points, r=2.1):
    ring = c.layer()
    hole = c.layer()
    for (x, y) in points:
        c.ellipse(ring, x - r, y - r, x + r, y + r)
        c.ellipse(hole, x - r * 0.45, y - r * 0.45, x + r * 0.45, y + r * 0.45)
    rgb = paint(rgb, f(ring), VIA)
    return paint(rgb, f(hole), [8, 30, 18])


def route(d, pts, width):
    d.line([(x * S, y * S) for x, y in pts], fill=255, width=int(width * S), joint="curve")


def ws_pcb(c, shoulder, strip_x0, strip_x1):
    w, h = c.w, c.h
    rng = np.random.default_rng(41)
    x0, x1, y0, y1 = strip_x0 + 4, strip_x1 - 4, 14, h - 2
    outline = c.rrect(c.layer(), x0, y0, x1, y1, 4)
    notch = 118
    cut = c.layer()
    c.ellipse(cut, x0 - 9, notch - 9, x0 + 9, notch + 9)
    c.ellipse(cut, x1 - 9, notch - 9, x1 + 9, notch + 9)
    # A small keyed slot in the top edge.
    c.rrect(cut, w * 0.5 - 7, y0 - 2, w * 0.5 + 7, y0 + 5, 2)
    region = np.clip(f(outline) - f(cut), 0, 1)

    # Solder mask: green, mottled, darker where the copper plane shows through.
    rgb = np.ones((c.H, c.W, 3), np.float32) * GREEN
    blotch = ndimage.gaussian_filter(rng.normal(0, 1, (c.H // 8 + 1, c.W // 8 + 1)), 2.2)
    blotch = np.kron(blotch / max(blotch.std(), 1e-6), np.ones((8, 8)))[: c.H, : c.W]
    rgb += (blotch * 7.0)[..., None].astype(np.float32)
    rgb += grain((c.H, c.W), 42, 4.0)[..., None]

    band_y0, band_y1 = shoulder - 13, shoulder + 4
    finger_y0, finger_y1 = shoulder + 5, h - 6
    count = 72
    pitch = (strip_x1 - 10 - (strip_x0 + 10)) / count

    # Traces: from the contacts up into the chips, in bundles with 45-degree jogs.
    traces = c.layer()
    d = ImageDraw.Draw(traces)
    for k in range(0, count, 2):
        fx = strip_x0 + 10 + pitch * (k + 0.5)
        if 4 <= k < 24:      # to the battery side and the eight-leg chip
            tx, ty = 40 + (k - 4) * 1.6, 146
        elif 24 <= k < 50:   # up into the ROM
            tx, ty = 126 + (k - 24) * 4.1, 92
        elif 50 <= k < 62:   # the save RAM
            tx, ty = 150 + (k - 50) * 5.0, 143
        elif 62 <= k < 70:   # the mapper
            tx, ty = 266 + (k - 62) * 4.6, 76
        else:
            continue
        up = band_y0 - 3 - (k % 6) * 2.2
        route(d, [(fx, band_y0 + 2), (fx, up), (tx, up - 6), (tx, ty)], 1.3)
    for k in range(12):  # ROM to mapper
        y = 34 + k * 4.2
        route(d, [(240, y), (250 + k * 0.6, y), (258, y + 2)], 1.0)
    for k in range(6):   # mapper to save RAM
        x = 268 + k * 5.5
        route(d, [(x, 76), (x, 96 + k * 2), (220 + k * 1.5, 118 + k * 3), (214, 118 + k * 3)], 1.0)
    rgb = paint(rgb, soft(f(traces), 0.2), TRACE, 0.9)

    # Copper fill around the battery, a shade lighter.
    plane = soft(f(c.rrect(c.layer(), 22, 22, 112, 112, 8)), 0.5)
    rgb = paint(rgb, plane, [16, 102, 60], 0.35)

    # The coin cell in its clip.
    bx, by, br = 66, 66, 37
    rgb = silk_text(c, rgb, bx - 30, 22, "BT1", 7)
    ring = np.clip(f(c.ellipse(c.layer(), bx - br - 4, by - br - 4, bx + br + 4, by + br + 4))
                   - f(c.ellipse(c.layer(), bx - br - 3.2, by - br - 3.2, bx + br + 3.2, by + br + 3.2)), 0, 1)
    rgb = paint(rgb, ring, SILK, 0.9)
    for px in (bx - br - 7, bx + br + 1):
        rgb = paint(rgb, f(c.rrect(c.layer(), px, by + 6, px + 7, by + 18, 1)), [196, 160, 70])
    shadow = soft(f(c.ellipse(c.layer(), bx - br + 2, by - br + 3, bx + br + 2, by + br + 3)), 2.0)
    rgb = rgb * (1 - 0.5 * shadow[..., None])
    disc = f(c.ellipse(c.layer(), bx - br, by - br, bx + br, by + br))
    yy, xx = np.mgrid[0:c.H, 0:c.W] / S
    dist = np.hypot(xx - bx, yy - by) / br
    metal = 168 + 46 * (1 - dist) - 22 * ((xx - bx) + (yy - by)) / br
    metal = metal + 2.5 * np.sin(dist * 90)  # the fine lathe rings
    rgb = rgb * (1 - disc[..., None]) + np.dstack([metal, metal, metal * 1.02]) * disc[..., None]
    edge = np.clip(disc - f(c.ellipse(c.layer(), bx - br + 2, by - br + 2, bx + br - 2, by + br - 2)), 0, 1)
    rgb = paint(rgb, edge, [120, 122, 126], 0.8)
    plus = c.layer()
    c.rrect(plus, bx + 10, by - 26, bx + 20, by - 24.6, 0.4)
    c.rrect(plus, bx + 14.3, by - 30.3, bx + 15.7, by - 20.3, 0.4)
    rgb = paint(rgb, f(plus), [110, 112, 116], 0.8)
    strap = f(c.rrect(c.layer(), bx - br + 6, by - 7, bx + br - 6, by + 7, 2)) * disc
    sy = np.clip((yy - (by - 7)) / 14.0, 0, 1)
    steel = 206 + 22 * (1 - sy) - 30 * sy
    rgb = rgb * (1 - strap[..., None]) + np.dstack([steel, steel, steel * 1.02]) * strap[..., None]
    rgb = paint(rgb, np.clip(strap - f(c.rrect(c.layer(), bx - br + 6, by - 7, bx + br - 6, by + 6, 2)), 0, 1), [120, 122, 126], 0.8)
    for sx in (bx - 16, bx + 8):
        bump = f(c.rrect(c.layer(), sx, by - 4, sx + 8, by + 3, 2)) * strap
        rgb = paint(rgb, bump, [168, 170, 174], 0.6)
        hi = f(c.rrect(c.layer(), sx + 1, by - 4, sx + 7, by - 2.5, 1)) * strap
        rgb = paint(rgb, hi, [246, 248, 250], 0.8)
    tab = f(c.rrect(c.layer(), bx - 6, by + br - 3, bx + 6, by + br + 6, 2))
    rgb = paint(rgb, tab, [150, 152, 156])

    # The ROM, the mapper, the save RAM and the little eight-leg chip.
    rx0, ry0, rx1, ry1 = WS_ROM
    rgb = silk_text(c, rgb, rx0 + 4, ry0 - 8, "U1", 7)
    rgb = chip_with_legs(c, rgb, rx0, ry0, rx1, ry1, "lr", 2.6, seed=51)

    mx0, my0, mx1, my1 = 262, 30, 306, 74
    rgb = silk_text(c, rgb, mx1 + 8, my0 - 1, "U2", 7)
    rgb = chip_with_legs(c, rgb, mx0, my0, mx1, my1, "lrtb", 2.6, leg=3.0, seed=52)
    rgb = paint(rgb, f(c.text(c.layer(), (mx0 + mx1) / 2, my0 + 18, "2003", 7)), [104, 108, 116])
    rgb = paint(rgb, f(c.text(c.layer(), (mx0 + mx1) / 2, my0 + 27, "SW-2", 7)), [104, 108, 116])

    sx0, sy0, sx1, sy1 = 146, 104, 214, 140
    rgb = silk_text(c, rgb, sx0 - 10, sy0 + 2, "U3", 7)
    rgb = chip_with_legs(c, rgb, sx0, sy0, sx1, sy1, "tb", 2.6, seed=53)
    rgb = paint(rgb, f(c.text(c.layer(), (sx0 + sx1) / 2, sy0 + 13, "256K", 7)), [104, 108, 116])

    ex0, ey0, ex1, ey1 = 36, 122, 60, 140
    rgb = silk_text(c, rgb, ex0 - 1, ey0 - 9, "U4", 7)
    rgb = chip_with_legs(c, rgb, ex0, ey0, ex1, ey1, "tb", 5.2, leg=3.0, seed=54)
    rgb = paint(rgb, f(c.text(c.layer(), (ex0 + ex1) / 2, ey0 + 9, "8", 6)), [104, 108, 116])

    # Capacitors and resistors.
    for (x, y, v, lab, at) in [
        (100, 26, False, "C1", (106, 20)), (100, 40, False, "C2", None),
        (248, 84, False, "C3", (243, 80)), (248, 96, False, "C4", (243, 92)),
        (74, 128, True, "C5", (80, 124)), (86, 128, True, "C6", (92, 124)),
        (222, 112, False, "R1", (235, 114)), (222, 124, False, "R2", (235, 126)),
        (312, 84, True, "C7", (318, 80)), (114, 140, False, "C8", (110, 135)),
    ]:
        rgb = passive(c, rgb, x, y, v, lab, at)

    # Vias, scattered where the traces dive through.
    pts = [(fx, band_y0 - 3 - (k % 6) * 2.2) for k, fx in
           ((k, strip_x0 + 10 + pitch * (k + 0.5)) for k in range(1, count, 9))]
    pts += [(120, 100), (232, 98), (250, 108), (300, 92), (320, 64), (318, 40),
            (110, 116), (130, 150), (222, 148), (28, 140), (98, 100), (244, 30),
            (318, 110), (70, 148), (196, 96)]
    rgb = vias(c, rgb, pts)

    # The light band of vias over the contacts, then the contacts.
    band = f(c.rrect(c.layer(), strip_x0 + 6, band_y0, strip_x1 - 6, band_y1, 0.5))
    rgb = paint(rgb, band, [198, 204, 194])
    rgb = paint(rgb, band * (grain((c.H, c.W), 43, 1.0)[...] > 1.4), [190, 196, 186])
    dots = c.layer()
    for k in range(0, count, 3):
        x = strip_x0 + 10 + pitch * (k + 0.5) + rng.uniform(-1, 1)
        y = band_y0 + 4 + (k % 2) * 5
        if rng.random() < 0.7:
            c.ellipse(dots, x - 1.3, y - 1.3, x + 1.3, y + 1.3)
    rgb = paint(rgb, f(dots), [150, 132, 70])

    # The board's mark.
    rgb = silk_mark(c, rgb, 326, 145)

    # A dark rim round the board's edge.
    dist = ndimage.distance_transform_edt(region > 0.5) / S
    rgb = rgb * (1 - 0.35 * np.clip(1.6 - dist, 0, 1))[..., None]
    rgb, alpha = fingers(c, strip_x0 + 10, strip_x1 - 10, finger_y0, finger_y1, count, rgb, region)
    return rgb, alpha


# --------------------------------------------------------------------------
# WonderSwan: a wide shell with an oval nameplate, and the board's contacts
# showing below its shoulder.


def wonderswan():
    w, h = 352, 211
    c = Canvas(w, h)
    top_r = 13
    shoulder = round(h * 0.80)
    strip_x0, strip_x1 = round(w * 0.035), round(w * 0.965)

    upper = c.rrect(c.layer(), 0, 0, w, shoulder + 4, top_r)
    lower = c.rrect(c.layer(), strip_x0, shoulder - 6, strip_x1, h, 6)
    body = np.maximum(f(upper), f(lower))

    lx0, ly0 = round(w * 0.040), round(h * 0.140)
    lx1, ly1 = round(w * 0.960), round(h * 0.765)
    well = f(c.rrect(c.layer(), lx0, ly0, lx1, ly1, 2))

    plate = c.rrect(c.layer(), w * 0.31, h * 0.018, w * 0.69, h * 0.118, h * 0.05)
    name = c.text(c.layer(), w * 0.5, h * 0.068, "WonderSwan", 14.5, italic=0.24, weight=0)
    strip = c.rrect(c.layer(), strip_x0 + 7, shoulder + 1, strip_x1 - 7, h - 3, 3)

    height = bevel(body, 5.0) * 3.2
    height -= soft(f(plate), 0.6) * 3.4
    height += soft(f(name), 0.3) * 2.6 * f(plate)
    height -= soft(well, 0.8) * 1.4
    height -= soft(f(c.rrect(c.layer(), 0, shoulder, w, shoulder + 2, 0)), 0.6) * 0.8
    height -= soft(f(strip), 0.6) * 1.6
    occl = soft(f(plate), 1.4) * 10 + soft(well, 2.0) * 5

    neutral = shade(height, body, well, seed=11, occlusion=occl)
    rgba(neutral, body).save(os.path.join(ASSETS, "ws_neutral.png"))
    rgba(clear_from(neutral, body, 12), body).save(os.path.join(ASSETS, "ws_clear.png"))
    rgba(np.full_like(body, 255.0) * body, body).save(os.path.join(ASSETS, "ws_cart.png"))
    interior(c, body, 13, [(w * 0.5, h * 0.42, 9), (w * 0.12, h * 0.55, 6),
                           (w * 0.88, h * 0.55, 6)]).save(os.path.join(ASSETS, "ws_interior.png"))

    board, balpha = ws_pcb(c, shoulder, strip_x0, strip_x1)
    rgba_colour(board, balpha).save(os.path.join(ASSETS, "ws_board.png"))

    # The strip below the shoulder is the bare board, drawn over any shell.
    decal = pcb(c, strip, 15, [])
    decal, dalpha = fingers(c, strip_x0 + 10, strip_x1 - 10, shoulder + 5, h - 6, 72,
                            decal, f(strip))
    rgba_colour(decal, dalpha).save(os.path.join(ASSETS, "ws_decal.png"))


# --------------------------------------------------------------------------
# Atari Lynx (the curved-lip card): a flat card whose top edge rolls into a
# lip, a label almost edge to edge, and the contacts behind a clear strip at
# the foot.


def lynx():
    w, h = 312, 328
    c = Canvas(w, h)
    body = f(c.rrect(c.layer(), 0, 0, w, h, 9))

    lip_bottom = round(h * 0.215)
    lx0, ly0 = round(w * 0.045), round(h * 0.255)
    lx1, ly1 = round(w * 0.955), round(h * 0.835)
    well = f(c.rrect(c.layer(), lx0, ly0, lx1, ly1, 2))
    wx0, wy0 = round(w * 0.04), round(h * 0.858)
    wx1, wy1 = round(w * 0.96), round(h * 0.968)
    window = c.rrect(c.layer(), wx0, wy0, wx1, wy1, 3)

    # The lip: a low swell that rolls over the top edge and eases back down
    # to the face along a gentle curve.
    ys = (np.arange(c.H) / S)[:, None] * np.ones((1, c.W))
    xs = (np.arange(c.W) / S)[None, :] * np.ones((c.H, 1))
    curve = lip_bottom - 12 * (1 - ((xs - w / 2) / (w / 2)) ** 2)
    lip = np.clip((curve - ys) / 5.0, 0, 1)
    lip = np.sin(lip * np.pi / 2) * (1 - np.clip((8 - ys) / 8, 0, 1) * 0.6)

    height = bevel(body, 4.0) * 2.6
    height += lip * 3.6
    height -= soft(well, 0.8) * 1.2
    height -= soft(f(window), 0.6) * 1.8
    occl = soft(well, 2.0) * 5 + soft(f(window), 1.5) * 4

    neutral = shade(height, body, well, seed=21, occlusion=occl)
    rgba(neutral, body).save(os.path.join(ASSETS, "lynx_neutral.png"))
    rgba(clear_from(neutral, body, 22), body).save(os.path.join(ASSETS, "lynx_clear.png"))
    rgba(np.full_like(body, 255.0) * body, body).save(os.path.join(ASSETS, "lynx_cart.png"))
    interior(c, body, 23, [(w * 0.5, h * 0.12, 8), (w * 0.15, h * 0.5, 6),
                           (w * 0.85, h * 0.5, 6)]).save(os.path.join(ASSETS, "lynx_interior.png"))

    board_region = c.rrect(c.layer(), 8, h * 0.18, w - 8, h - 4, 4)
    board = pcb(c, board_region, 24, [(w * 0.2, h * 0.32, w * 0.55, h * 0.6),
                                       (w * 0.62, h * 0.4, w * 0.8, h * 0.55)])
    board = silk_mark(c, board, w - 22, h * 0.80)
    board, balpha = fingers(c, wx0 + 4, wx1 - 4, wy0 + 3, wy1 - 2, 34, board,
                            f(board_region))
    rgba_colour(board, balpha).save(os.path.join(ASSETS, "lynx_board.png"))

    # Through the clear strip: the dark board with its gold contacts.
    under = np.ones((c.H, c.W, 3), np.float32) * np.array([40, 46, 44], np.float32)
    decal, dalpha = fingers(c, wx0 + 4, wx1 - 4, wy0 + 3, wy1 - 2, 34, under,
                            f(window) * 0.92)
    rgba_colour(decal, dalpha).save(os.path.join(ASSETS, "lynx_decal.png"))


# --------------------------------------------------------------------------
# Sega Game Gear: a nearly square pak. A raised hood with three grip ridges
# runs across the top and down both sides as far as the flanges, the label
# sits in the U it makes, and an oval plaque with SEGA pressed into it sits
# at the foot. The contacts are hidden under the bottom edge, so there
# is no decal.


GG_LABEL_R = 14  # the label's corner radius, as slot-ui rounds the art


def gamegear():
    w, h = 352, 329
    c = Canvas(w, h)
    flange = round(h * 0.376)
    inset = 4
    upper = c.rrect(c.layer(), 0, 0, w, flange + 6, 15, corners=(True, True, False, False))
    lower = c.rrect(c.layer(), inset, flange - 6, w - inset, h, 8)
    body = np.maximum(f(upper), f(lower))

    lx0, ly0 = round(w * 0.088), round(h * 0.145)
    lx1, ly1 = round(w * 0.912), round(h * 0.755)
    well = f(c.rrect(c.layer(), lx0, ly0, lx1, ly1, GG_LABEL_R))

    # The U the hood leaves open: the label well and the face around it,
    # from just under the grip band down to the foot.
    side = round(w * 0.058)
    band = round(h * 0.118)
    # Its top corners are big, slightly tall arches, as on the real pak.
    rx, ry = 42, 50
    face = c.layer()
    d = ImageDraw.Draw(face)
    d.rectangle([(side + rx) * S, band * S, (w - side - rx) * S, (h + 20) * S], fill=255)
    d.rectangle([side * S, (band + ry) * S, (w - side) * S, (h + 20) * S], fill=255)
    for cx in (side, w - side - 2 * rx):
        d.ellipse([cx * S, band * S, (cx + 2 * rx) * S, (band + 2 * ry) * S], fill=255)
    # Below the flanges the sides drop to the face too.
    c.rrect(face, 0, flange + 3, w, h + 20, 0)
    face = f(face)

    grips = c.layer()
    for y in (0.064, 0.083, 0.102):
        c.rrect(grips, w * 0.30, h * y, w * 0.70, h * y + 2.2, 1.1)
    plate = c.rrect(c.layer(), w * 0.305, h * 0.805, w * 0.695, h * 0.925, h * 0.06)
    name = c.text(c.layer(), w * 0.5, h * 0.866, "SEGA", 25, weight=1)

    height = bevel(body, 4.5) * 2.8
    height += (1 - soft(face, 0.7)) * 3.4
    height += soft(f(grips), 0.4) * 1.2
    height -= soft(well, 0.8) * 1.3
    height -= soft(f(plate), 0.7) * 2.6
    height -= soft(f(name), 0.3) * 2.2 * f(plate)  # the lettering is pressed in
    occl = soft(well, 2.0) * 5 + soft(f(plate), 1.5) * 7 + soft(face, 2.5) * 3 * (1 - well)
    occl += soft(f(name), 0.5) * 6 * f(plate)

    neutral = shade(height, body, well, seed=31, occlusion=occl)
    rgba(neutral, body).save(os.path.join(ASSETS, "gg_neutral.png"))
    rgba(clear_from(neutral, body, 32), body).save(os.path.join(ASSETS, "gg_clear.png"))
    rgba(np.full_like(body, 255.0) * body, body).save(os.path.join(ASSETS, "gg_cart.png"))
    interior(c, body, 33, [(w * 0.5, h * 0.2, 9), (w * 0.14, h * 0.6, 6),
                           (w * 0.86, h * 0.6, 6)]).save(os.path.join(ASSETS, "gg_interior.png"))

    board_region = c.rrect(c.layer(), 12, h * 0.16, w - 12, h - 3, 5)
    board = pcb(c, board_region, 34, [(w * 0.2, h * 0.3, w * 0.5, h * 0.55),
                                       (w * 0.6, h * 0.36, w * 0.78, h * 0.5)])
    board, balpha = fingers(c, w * 0.08, w * 0.92, h * 0.94, h - 5, 50, board,
                            f(board_region))
    rgba_colour(board, balpha).save(os.path.join(ASSETS, "gg_board.png"))


# --------------------------------------------------------------------------
# NeoGeo Pocket: a pak a little wider than it is tall. A ridged cap runs
# across the top, its sides standing a touch proud of the body; the wide
# label sits in a recessed frame; and below a seam the foot carries two
# plaques, SNK and NGP, with a down arrow between them, all pressed in.
# The contacts are hidden, so there is no decal.

NGP_LABEL_R = 6  # the label's corner radius, as slot-ui rounds the art


def neogeo_pocket():
    w, h = 312, 268
    c = Canvas(w, h)
    cap_h = round(h * 0.12)
    inset = 3
    cap = c.rrect(c.layer(), 0, 0, w, cap_h + 4, 9, corners=(True, True, False, False))
    lower = c.rrect(c.layer(), inset, cap_h - 4, w - inset, h, 6)
    body = np.maximum(f(cap), f(lower))

    lx0, ly0 = round(w * 0.115), round(h * 0.215)
    lx1, ly1 = round(w * 0.905), round(h * 0.712)
    well = f(c.rrect(c.layer(), lx0, ly0, lx1, ly1, NGP_LABEL_R))
    frame = f(c.rrect(c.layer(), w * 0.092, h * 0.195, w * 0.928, h * 0.732, 9))

    grooves = c.layer()
    for k in range(8):
        y = h * (0.03 + k * 0.019)
        c.rrect(grooves, w * 0.09, y, w * 0.91, y + 1.6, 0.8)

    seam_y = h * 0.79
    seam = c.rrect(c.layer(), inset, seam_y - 1, w - inset, seam_y + 1, 0)
    foot = f(c.rrect(c.layer(), inset, seam_y, w - inset, h, 0))
    plaques = c.layer()
    c.rrect(plaques, w * 0.08, h * 0.835, w * 0.355, h * 0.948, 3)
    c.rrect(plaques, w * 0.645, h * 0.835, w * 0.92, h * 0.948, 3)
    left_word = c.text(c.layer(), w * 0.2175, h * 0.8915, "SNK", 21, weight=1)
    right_word = c.text(c.layer(), w * 0.7825, h * 0.8915, "NGP", 21, weight=1)
    words = np.maximum(f(left_word), f(right_word)) * f(plaques)
    arrow = f(c.poly(c.layer(), [(w * 0.425, h * 0.842), (w * 0.582, h * 0.842),
                                 (w * 0.5035, h * 0.948)]))
    # The cap stands proud of the body below it, with a ledge along its foot.
    capband = f(c.rrect(c.layer(), 0, 0, w, cap_h, 9, corners=(True, True, False, False)))

    height = bevel(body, 4.0) * 2.6
    # Shaded the same way as the band below: a groove or a recess is dark
    # along its top and left edges and light along its bottom and right, and
    # the raised cap the other way round, so its foot casts a dark ledge.
    height -= soft(capband, 0.5) * 2.4
    height += soft(f(grooves), 0.35) * 1.1
    height += soft(frame, 0.7) * 1.6
    height += soft(well, 0.6) * 0.6
    # The seam above the band is a groove, shaded like the plaques: dark along
    # its top edge and light along its bottom.
    height += soft(f(seam), 0.5) * 2.4
    height += soft(foot, 0.6) * 0.5
    # The plaques and the arrow are pressed into the band and the lettering
    # stands up out of the plaques. They are shaded so that a pressed-in
    # shape is dark along its top and left edges and light along its bottom
    # and right, and the raised letters the other way round.
    height += soft(f(plaques), 0.6) * 2.6
    height += soft(arrow, 0.7) * 3.6
    height -= soft(words, 0.3) * 2.0
    ledge = np.clip(soft(capband, 2.4) - capband, 0, 1)
    occl = soft(frame, 1.8) * 6 + soft(f(plaques), 1.2) * 4 + soft(f(seam), 1.0) * 6
    occl += ledge * 110 + soft(arrow, 0.8) * 10 + f(plaques) * 5 - soft(words, 0.4) * 5

    # The floors of the plaques and the arrow are smooth, not stippled.
    floor = soft(np.clip(f(plaques) - words, 0, 1) + arrow, 0.4)
    neutral = shade(height, body, well, seed=41, occlusion=occl, smooth=floor)
    rgba(neutral, body).save(os.path.join(ASSETS, "ngp_neutral.png"))
    rgba(clear_from(neutral, body, 42), body).save(os.path.join(ASSETS, "ngp_clear.png"))
    rgba(np.full_like(body, 255.0) * body, body).save(os.path.join(ASSETS, "ngp_cart.png"))
    interior(c, body, 43, [(w * 0.5, h * 0.16, 8), (w * 0.12, h * 0.6, 6),
                           (w * 0.88, h * 0.6, 6)]).save(os.path.join(ASSETS, "ngp_interior.png"))

    board_region = c.rrect(c.layer(), 10, h * 0.16, w - 10, h - 3, 5)
    board = pcb(c, board_region, 44, [(w * 0.18, h * 0.3, w * 0.5, h * 0.58),
                                       (w * 0.6, h * 0.34, w * 0.8, h * 0.5)])
    board, balpha = fingers(c, w * 0.07, w * 0.93, h * 0.935, h - 5, 44, board,
                            f(board_region))
    rgba_colour(board, balpha).save(os.path.join(ASSETS, "ngp_board.png"))


# --------------------------------------------------------------------------
# TurboGrafx-16 TurboChip (HuCard): a thin card the size of a credit card,
# 86 x 54 mm, its game art covering most of the face and its 38 gold
# contacts showing in a strip along the foot.

TG16_LABEL_R = 0  # HuCard labels are printed square, edge to edge


def turbochip():
    """The HuCard: a thin white card the size of a credit card. Gold contacts
    run along its top edge over a dark textured panel; the printed label fills
    the lower half, and a small ridged tab at the foot is the grip."""
    w, h = 214, 337
    c = Canvas(w, h)
    body = f(c.rrect(c.layer(), 0, 0, w, h, 9))

    # The label, printed flat on the card.
    lx0, ly0 = round(w * 0.06), round(h * 0.42)
    lx1, ly1 = round(w * 0.955), round(h * 0.905)
    well = f(c.rrect(c.layer(), lx0, ly0, lx1, ly1, TG16_LABEL_R))

    # The dark panel across the top, square along the card's edge and rounded
    # where it ends above the label; the contacts sit in a band at its top.
    px0, py0 = w * 0.035, h * 0.018
    px1, py1 = w * 0.965, h * 0.39
    panel = f(c.rrect(c.layer(), px0, py0, px1, py1, 13,
                      corners=(False, False, True, True)))
    cy0, cy1 = h * 0.024, h * 0.068

    # The grip: a low tab at the foot, peaked in the middle, with fine ridges
    # across it.
    gx0, gx1 = w * 0.365, w * 0.635
    gy0, gy1 = h * 0.925, h * 0.985
    tab = f(c.poly(c.layer(), [
        (gx0, gy1), (gx0 + w * 0.03, gy0 + h * 0.012), (w * 0.5, gy0),
        (gx1 - w * 0.03, gy0 + h * 0.012), (gx1, gy1)]))
    ridges = c.layer()
    for k in range(5):
        y = gy0 + h * 0.016 + k * (gy1 - gy0 - h * 0.02) / 5
        c.rrect(ridges, gx0, y, gx1, y + 1.1, 0.5)
    ridges = f(ridges) * tab

    height = bevel(body, 2.0) * 1.2
    height -= soft(tab, 0.5) * 1.8  # raised a little off the card
    height += soft(ridges, 0.25) * 1.2  # grooves pressed into it
    height += soft(panel, 0.4) * 0.15  # the panel is a thin film, barely in

    # The label is printed on the card, so the card's own texture runs under
    # it: a label shorter than the space leaves plain card above it.
    neutral = shade(height, body, np.zeros_like(well), seed=61, gain=20.0)
    rgba(neutral, body).save(os.path.join(ASSETS, "tg16_neutral.png"))
    rgba(clear_from(neutral, body, 62), body).save(os.path.join(ASSETS, "tg16_clear.png"))
    rgba(np.full_like(body, 255.0) * body, body).save(os.path.join(ASSETS, "tg16_cart.png"))
    interior(c, body, 63, [(w * 0.5, h * 0.66, 6)]).save(os.path.join(ASSETS, "tg16_interior.png"))

    board_region = c.rrect(c.layer(), 4, 4, w - 4, h - 4, 7)
    board = pcb(c, board_region, 64, [(w * 0.25, h * 0.2, w * 0.75, h * 0.5)])
    board, balpha = fingers(c, px0 + 3, px1 - 3, cy0, cy1, 38, board,
                            f(board_region))
    rgba_colour(board, balpha).save(os.path.join(ASSETS, "tg16_board.png"))

    # The panel and its contacts are printed and plated, not moulded, so they
    # go in the decal and keep their colour over any shell: a deep plum-brown
    # with a pebbled texture, and a black band behind the contacts.
    rng = np.random.default_rng(65)
    pebble = rng.normal(0.0, 1.0, (c.H // S, c.W // S)).astype(np.float32)
    pebble = ndimage.gaussian_filter(pebble, 0.7)
    pebble = np.kron(pebble / pebble.std(), np.ones((S, S), np.float32))
    ys = np.arange(c.H, dtype=np.float32)[:, None] / S
    base = np.array([44, 36, 42], np.float32)
    under = base + pebble[..., None] * 4.5
    band = (ys >= cy0 - 1) & (ys <= cy1 + 1)
    under = np.where(band[..., None], np.array([16, 14, 15], np.float32), under)
    under = np.broadcast_to(under, (c.H, c.W, 3)).copy()
    decal, dalpha = fingers(c, px0 + 3, px1 - 3, cy0, cy1, 38, under, panel)
    rgba_colour(decal, dalpha).save(os.path.join(ASSETS, "tg16_decal.png"))


# --------------------------------------------------------------------------
# Nintendo DS game card: a small card a touch taller than it is wide,
# 33 x 35 mm, one bottom corner cut off so it only goes in one way round.
# The label sits in a shallow recess over most of the face, and a little
# arrow pressed into the foot shows which way it goes in. The contacts are
# on the back, so there is no decal.

NDS_LABEL_R = 4  # the label's corner radius, as slot-ui rounds the art


def ds_card():
    w, h = 288, 304
    c = Canvas(w, h)
    cut = w * 0.075
    r = 8
    # The outline: a rounded card with its bottom left corner cut away.
    outline = c.layer()
    d = ImageDraw.Draw(outline)
    d.rounded_rectangle([0, 0, w * S - 1, h * S - 1], radius=r * S, fill=255)
    d.polygon([(0, (h - cut) * S), (cut * S, h * S), (-S, h * S + S), (-S, (h - cut) * S)],
              fill=0)
    body = f(outline)

    lx0, ly0 = round(w * 0.10), round(h * 0.045)
    lx1, ly1 = round(w * 0.90), round(h * 0.87)
    well = f(c.rrect(c.layer(), lx0, ly0, lx1, ly1, NDS_LABEL_R))
    rim_band = f(c.rrect(c.layer(), w * 0.075, h * 0.025, w * 0.925, h * 0.89, 7))

    arrow = f(c.poly(c.layer(), [(w * 0.465, h * 0.915), (w * 0.535, h * 0.915),
                                 (w * 0.5, h * 0.965)]))

    height = bevel(body, 3.5) * 2.4
    # The label's recess has a soft raised lip round it, then drops to the
    # well; the arrow is pressed into the foot.
    height += soft(rim_band, 0.8) * 0.8
    height -= soft(well, 0.6) * 1.6
    height += soft(arrow, 0.6) * 2.4
    occl = soft(well, 1.8) * 6 + soft(arrow, 0.7) * 8
    neutral = shade(height, body, well, seed=71, occlusion=occl, smooth=soft(arrow, 0.4))
    rgba(neutral, body).save(os.path.join(ASSETS, "nds_neutral.png"))
    rgba(clear_from(neutral, body, 72), body).save(os.path.join(ASSETS, "nds_clear.png"))
    rgba(np.full_like(body, 255.0) * body, body).save(os.path.join(ASSETS, "nds_cart.png"))
    interior(c, body, 73, [(w * 0.5, h * 0.5, 7)]).save(os.path.join(ASSETS, "nds_interior.png"))

    board_region = c.rrect(c.layer(), 8, 8, w - 8, h - 8, 5)
    board = pcb(c, board_region, 74, [(w * 0.22, h * 0.28, w * 0.62, h * 0.62)])
    board, balpha = fingers(c, w * 0.1, w * 0.9, 6, h * 0.05, 17, board,
                            f(board_region))
    rgba_colour(board, balpha).save(os.path.join(ASSETS, "nds_board.png"))


MOULDS = {
    "ws": wonderswan,
    "lynx": lynx,
    "gg": gamegear,
    "ngp": neogeo_pocket,
    "tg16": turbochip,
    "nds": ds_card,
}

if __name__ == "__main__":
    for name in sys.argv[1:] or MOULDS:
        MOULDS[name]()
        print("drew", name)
