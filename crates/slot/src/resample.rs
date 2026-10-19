const TAPS: usize = 48;
const HALF: usize = TAPS / 2;
const PHASES: usize = 256;
const CUTOFF: f64 = 22_000.0 / 24_000.0;
const BETA: f64 = 7.0;

pub struct Resampler {
    step: f64,
    scaled: f64,
    pos: f64,
    kernel: Vec<f32>,
    history: Vec<f32>,
}

impl Resampler {
    pub fn new(src_hz: f64, dst_hz: f64) -> Self {
        let usable = |hz: f64| hz.is_finite() && hz > 0.0;
        let step = if usable(src_hz) && usable(dst_hz) {
            src_hz / dst_hz
        } else {
            1.0
        };
        Resampler {
            step,
            scaled: step,
            pos: HALF as f64,
            kernel: kernel(CUTOFF * (1.0 / step).min(1.0)),
            history: vec![0.0; TAPS * 2],
        }
    }

    pub fn set_ratio(&mut self, ratio: f64) {
        if ratio > 0.0 {
            self.scaled = self.step / ratio;
        }
    }

    pub fn process(&mut self, src: &[i16], out: &mut Vec<i16>) {
        out.clear();
        if src.len() < 2 {
            return;
        }
        self.history
            .extend(src[..src.len() / 2 * 2].iter().map(|&s| f32::from(s)));
        let frames = self.history.len() / 2;
        while self.pos + (HALF as f64) < frames as f64 {
            let i = self.pos as usize;
            let at = (self.pos - i as f64) * PHASES as f64;
            let phase = at as usize;
            let blend = (at - phase as f64) as f32;
            let a = &self.kernel[phase * TAPS..(phase + 1) * TAPS];
            let b = &self.kernel[(phase + 1) * TAPS..(phase + 2) * TAPS];
            let window = &self.history[(i + 1 - HALF) * 2..(i + 1 + HALF) * 2];
            let (mut l, mut r) = (0.0f32, 0.0f32);
            for t in 0..TAPS {
                let w = a[t] + (b[t] - a[t]) * blend;
                l += window[t * 2] * w;
                r += window[t * 2 + 1] * w;
            }
            out.push(l.round().clamp(-32768.0, 32767.0) as i16);
            out.push(r.round().clamp(-32768.0, 32767.0) as i16);
            self.pos += self.scaled;
        }
        let drop = self.pos as usize + 1 - HALF;
        self.history.drain(..drop * 2);
        self.pos -= drop as f64;
    }
}

fn kernel(cutoff: f64) -> Vec<f32> {
    let mut k = vec![0.0f32; (PHASES + 1) * TAPS];
    for phase in 0..=PHASES {
        let frac = phase as f64 / PHASES as f64;
        let row = &mut k[phase * TAPS..(phase + 1) * TAPS];
        let mut sum = 0.0;
        let mut taps = [0.0f64; TAPS];
        for (t, tap) in taps.iter_mut().enumerate() {
            let x = t as f64 + 1.0 - HALF as f64 - frac;
            let window = bessel_i0(BETA * (1.0 - (x / HALF as f64).powi(2)).max(0.0).sqrt())
                / bessel_i0(BETA);
            *tap = cutoff * sinc(cutoff * x) * window;
            sum += *tap;
        }
        for (dst, tap) in row.iter_mut().zip(taps) {
            *dst = (tap / sum) as f32;
        }
    }
    k
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        let px = std::f64::consts::PI * x;
        px.sin() / px
    }
}

fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    for k in 1..32 {
        term *= (x / (2.0 * k as f64)).powi(2);
        sum += term;
    }
    sum
}
