use crate::surface::{OUT_H, OUT_W};

const LINE_PX: f32 = 2.0;

const DOT_T: f32 = 0.12;

const OVERSHOOT: f32 = 0.6;

pub fn screen_scale(t: f32) -> f32 {
    let left = 1.0 - t.clamp(0.0, 1.0);
    1.0 - (1.0 - LINE_PX / OUT_H as f32) * left * left
}

pub fn screen_width(t: f32) -> f32 {
    let left = 1.0 - (t.clamp(0.0, 1.0) / DOT_T).min(1.0);
    1.0 - (1.0 - LINE_PX / OUT_W as f32) * left
}

pub fn screen_brightness(t: f32) -> f32 {
    let left = 1.0 - t.clamp(0.0, 1.0);
    1.0 + OVERSHOOT * left * left
}

/// The picture's place while a footer of `footer` px stays put along the bottom.
/// The scale is centred in the space above that strip, so the strip is not part of it.
pub fn screen_rect_above(t: f32, footer: f32) -> (f32, f32, f32, f32) {
    let view = OUT_H as f32 - footer.max(0.0);
    let w = OUT_W as f32 * screen_width(t);
    let h = view * screen_scale(t);
    ((OUT_W as f32 - w) / 2.0, (view - h) / 2.0, w, h)
}
