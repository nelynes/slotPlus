"""slotPlus: the shrink wrap tear, synthesized: a crackle of cellophane ripping open.

Writes crates/slot/assets/wrap_tear.pcm (signed 16-bit little-endian, mono, 48 kHz).
Made from noise alone, seeded, so it builds the same every time:
  * crackles: hundreds of tiny, bright clicks, each a few samples of high-passed noise with
    a fast decay, thick at the rip's start and thinning as the film comes away;
  * the rip: a short burst of band-passed noise that sweeps down as the film splits;
  * rustle: a soft, low crinkle under it all as the halves peel off.
Run: python3 tools/sfx/wrap_tear.py
"""
import math
import random
import struct
import sys

RATE = 48_000
LENGTH = 0.48
OUT = "crates/slot/assets/wrap_tear.pcm"


def one_pole_hp(x, cut):
    a = math.exp(-2 * math.pi * cut / RATE)
    y, px, py = [], 0.0, 0.0
    for v in x:
        o = a * (py + v - px)
        y.append(o)
        px, py = v, o
    return y


def biquad_bp(x, f, q):
    w = 2 * math.pi * f / RATE
    al = math.sin(w) / (2 * q)
    b0, b1, b2 = al, 0.0, -al
    a0, a1, a2 = 1 + al, -2 * math.cos(w), 1 - al
    y, x1, x2, y1, y2 = [], 0.0, 0.0, 0.0, 0.0
    for v in x:
        o = (b0 * v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2) / a0
        y.append(o)
        x2, x1, y2, y1 = x1, v, y1, o
    return y


def main():
    rnd = random.Random(0x5EA1)
    n = int(RATE * LENGTH)
    out = [0.0] * n

    # Crackles.
    t = 0.0
    while t < LENGTH:
        # Dense while ripping (first ~150 ms), sparser as the halves peel away.
        rate = 900 if t < 0.15 else 900 * math.exp(-(t - 0.15) / 0.12) + 40
        t += rnd.expovariate(rate)
        if t >= LENGTH:
            break
        at = int(t * RATE)
        amp = rnd.uniform(0.25, 1.0) * (1.0 if t < 0.2 else math.exp(-(t - 0.2) / 0.18))
        decay = rnd.uniform(0.0002, 0.0012)
        length = int(decay * 6 * RATE)
        for k in range(length):
            if at + k >= n:
                break
            out[at + k] += amp * rnd.uniform(-1, 1) * math.exp(-k / (decay * RATE))

    out = one_pole_hp(out, 1800)

    # The rip: band-passed noise sweeping down from ~5 kHz.
    rip = [0.0] * n
    rip_len = int(0.13 * RATE)
    noise = [rnd.uniform(-1, 1) for _ in range(rip_len)]
    chunk = 256
    for s in range(0, rip_len, chunk):
        f = 5200 - 2600 * (s / rip_len)
        seg = biquad_bp(noise[max(0, s - 64): s + chunk], f, 2.5)[-(min(chunk, rip_len - s)):]
        for k, v in enumerate(seg):
            i = s + k
            env = min(1.0, i / (0.006 * RATE)) * math.exp(-i / (0.07 * RATE))
            rip[i + int(0.01 * RATE)] += v * env * 2.2

    # Rustle: low crinkle, slow swell and fade.
    rustle = biquad_bp([rnd.uniform(-1, 1) for _ in range(n)], 900, 0.8)
    for i in range(n):
        x = i / n
        rustle[i] *= 0.10 * math.sin(math.pi * min(1.0, x * 1.4)) ** 2

    mix = [a + b + c for a, b, c in zip(out, rip, rustle)]
    # Soft fade at both ends so it starts and stops without a click.
    fade = int(0.004 * RATE)
    for i in range(fade):
        mix[i] *= i / fade
        mix[n - 1 - i] *= i / fade
    peak = max(abs(v) for v in mix) or 1.0
    gain = 0.55 * 32767 / peak
    data = b"".join(struct.pack("<h", int(max(-32768, min(32767, v * gain)))) for v in mix)
    with open(sys.argv[1] if len(sys.argv) > 1 else OUT, "wb") as f:
        f.write(data)


if __name__ == "__main__":
    main()
