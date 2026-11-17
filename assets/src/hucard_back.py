#!/usr/bin/env python3
"""Draw what is printed on the back of a slotPlus HuCard.

A HuCard has nothing inside to show, so START turns it over, the way the Lynx
card turns. Its back is printed like the real cards' backs: a band across the
top, the handling notes in Japanese and English, a line to write your name on,
and the slotPlus wordmark low down, with a serial number pressed into the
plastic beside the grip tab. No maker's name or logo is drawn.

It writes `assets/tg16_back.png`: the print alone, on transparency, at the
card's size. Slot lays it over the back plastic in the card's own colour
(`slot_ui::hucard_back_face`), writes the Nickname on the NAME line and puts
the stickers to the right of the Notes; the spots for those are in
`crates/slot-ui/src/board.rs` and `shelf_life.rs` and follow this layout.

Needs the Inter and Noto Sans CJK fonts (Debian: fonts-inter, fonts-noto-cjk).

usage: python3 assets/src/hucard_back.py
"""

import os

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.dirname(HERE)
INTER = "/usr/share/fonts/opentype/inter/"
CJK = "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc"

W, H = 214, 337  # the TurboGrafx-16 card, as tg16_cart.png
K = 4  # drawn this many times over, then scaled down
NAVY = (31, 46, 140, 255)
WHITE = (255, 255, 255, 255)
# Pressed into the plastic, not printed: a little darker than whatever colour
# the card is.
PRESSED = (0, 0, 0, 70)


def inter(name, px):
    return ImageFont.truetype(INTER + name, round(px * K))


def cjk(px):
    return ImageFont.truetype(CJK, round(px * K), index=0)


JAPANESE = [
    "●抜き差しは、必ず本体の電源を切ってから。",
    "●曲げたり、落としたり、衝撃を与えないで。",
    "●熱、水分、直射日光をさけてください。",
    "●遊んだあとは、棚に戻してください。",
]
# Kept narrow: the stickers go to the right of them.
NOTES = [
    ["Don't bend, drop or", "blow on the contacts."],
    ["Keep away from static,", "heat, water and direct", "sunlight."],
    ["Please put back on the", "shelf after you play."],
]


def draw():
    ink = Image.new("RGBA", (W * K, H * K), (0, 0, 0, 0))
    d = ImageDraw.Draw(ink)

    def text(x, y, s, font, fill=NAVY, anchor="la"):
        d.text((x * K, y * K), s, font=font, fill=fill, anchor=anchor)

    d.rectangle((15 * K, 16 * K, 199 * K, 44 * K), fill=NAVY)
    text(107, 30.5, "GAME CARD", inter("Inter-BlackItalic.otf", 19), WHITE, "mm")
    text(107, 53, "取扱い上のご注意", cjk(8.2), anchor="mm")
    for i, line in enumerate(JAPANESE):
        text(24, 61 + i * 8.6, line, cjk(5.6))

    text(22, 108, "NAME:", inter("Inter-Bold.otf", 8.5))
    d.rectangle((22 * K, 119 * K, 194 * K, round(119.7 * K)), fill=NAVY)

    text(107, 132, "Notes", inter("Inter-SemiBold.otf", 11), anchor="mm")
    y = 143.0
    for note in NOTES:
        text(22, y, "●", inter("Inter-Medium.otf", 6.4))
        for line in note:
            text(30, y, line, inter("Inter-Medium.otf", 6.6))
            y += 9.2
        y += 2.0

    text(107, 262, "slotPlus", inter("Inter-Bold.otf", 23), anchor="mm")
    text(107, 280, "MADE FOR THE RG SP", inter("Inter-SemiBold.otf", 5.6), anchor="mm")

    # The serial, pressed in from the front, reads backwards from the back.
    serial = Image.new("RGBA", (W * K, H * K), (0, 0, 0, 0))
    ImageDraw.Draw(serial).text(
        ((W - 48 - 30) * K, 318 * K), "SP2026", font=inter("Inter-Medium.otf", 7), fill=PRESSED
    )
    ink.alpha_composite(serial.transpose(Image.FLIP_LEFT_RIGHT))
    return ink.resize((W, H), Image.LANCZOS)


if __name__ == "__main__":
    out = os.path.join(ASSETS, "tg16_back.png")
    draw().save(out, optimize=True)
    print("wrote", out)
