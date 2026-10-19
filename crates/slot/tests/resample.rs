use slot::resample::Resampler;

fn ramp(frames: usize) -> Vec<i16> {
    (0..frames * 2)
        .map(|i| (i as i16).wrapping_mul(37).wrapping_sub(400))
        .collect()
}

#[test]
fn chunked_resampling_matches_one_pass() {
    let src = ramp(1000);
    let mut out = Vec::new();

    let mut whole = Resampler::new(32768.0, 48000.0);
    whole.process(&src, &mut out);
    let want = out.clone();

    let mut chunked = Resampler::new(32768.0, 48000.0);
    let mut got = Vec::new();
    for chunk in src.chunks(2 * 137) {
        chunked.process(chunk, &mut out);
        got.extend_from_slice(&out);
    }
    assert_eq!(got, want, "the seam between core frames is not continuous");
}

#[test]
fn a_starved_ratio_stretches_and_a_full_one_compresses() {
    let src = ramp(4096);
    let mut out = Vec::new();

    let mut flat = Resampler::new(32768.0, 32768.0);
    flat.process(&src, &mut out);
    let neutral = out.len();
    assert_eq!(neutral, src.len());

    let mut starved = Resampler::new(32768.0, 32768.0);
    starved.set_ratio(1.005);
    starved.process(&src, &mut out);
    assert!(
        out.len() > neutral,
        "starved ratio produced {} frames",
        out.len() / 2
    );

    let mut full = Resampler::new(32768.0, 32768.0);
    full.set_ratio(0.995);
    full.process(&src, &mut out);
    assert!(
        out.len() < neutral,
        "full ratio produced {} frames",
        out.len() / 2
    );
}

#[test]
fn a_core_rate_that_is_not_a_rate_passes_the_samples_through() {
    for (name, hz) in [
        ("nan", f64::NAN),
        ("infinity", f64::INFINITY),
        ("zero", 0.0),
        ("negative", -32768.0),
    ] {
        let mut r = Resampler::new(hz, 48_000.0);
        let mut out = Vec::new();
        let src = ramp(64);
        r.process(&src, &mut out);
        assert_eq!(
            out.len(),
            src.len(),
            "a core rate of {name} produced {} frames from {}",
            out.len() / 2,
            src.len() / 2
        );
    }
}

#[test]
fn the_output_rate_follows_the_device_not_the_core() {
    let mut r = Resampler::new(32768.0, 48000.0);
    let mut out = Vec::new();
    let mut frames = 0;
    for _ in 0..60 {
        r.process(&ramp(546), &mut out);
        frames += out.len() / 2;
    }
    let want = 546 * 60 * 48000 / 32768;
    assert!(
        (frames as i64 - want as i64).abs() <= 2,
        "produced {frames} frames, expected about {want}"
    );
}

fn tone(hz: f64, rate: f64, frames: usize, amp: f64) -> Vec<i16> {
    (0..frames)
        .flat_map(|i| {
            let s =
                (amp * (2.0 * std::f64::consts::PI * hz * i as f64 / rate).sin()).round() as i16;
            [s, s]
        })
        .collect()
}

fn rms(samples: &[i16]) -> f64 {
    let left: Vec<f64> = samples.iter().step_by(2).map(|&s| f64::from(s)).collect();
    (left.iter().map(|s| s * s).sum::<f64>() / left.len() as f64).sqrt()
}

fn resampled(hz: f64) -> (f64, f64) {
    let src = tone(hz, 65536.0, 65536, 16000.0);
    let mut r = Resampler::new(65536.0, 48000.0);
    let mut out = Vec::new();
    let mut all = Vec::new();
    for chunk in src.chunks(2 * 1097) {
        r.process(chunk, &mut out);
        all.extend_from_slice(&out);
    }
    let settled = &all[2 * 1024..all.len() - 2 * 1024];
    (rms(&src), rms(settled))
}

#[test]
fn a_tone_above_the_device_nyquist_does_not_fold_back_into_hearing() {
    for hz in [26_000.0, 30_000.0, 32_000.0] {
        let (input, output) = resampled(hz);
        let db = 20.0 * (output / input).log10();
        assert!(db < -50.0, "a {hz} Hz tone came through at {db:.1} dB");
    }
}

#[test]
fn a_tone_in_hearing_keeps_its_level() {
    for hz in [440.0, 4_000.0, 12_000.0, 18_000.0] {
        let (input, output) = resampled(hz);
        let db = 20.0 * (output / input).log10();
        assert!(db.abs() < 0.5, "a {hz} Hz tone changed level by {db:.2} dB");
    }
}
