use slot_gfx::{Canvas, Draw};
use slot_ui::{RebuildScreen, TexId};

fn stack_middle(out: &[Draw]) -> (f32, f32) {
    let (mut top, mut bottom, mut left, mut right) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for d in &out[1..] {
        let (x, y, w, h) = match d {
            Draw::Tex { x, y, w, h, .. }
            | Draw::Part { x, y, w, h, .. }
            | Draw::Rect { x, y, w, h, .. } => (*x, *y, *w, *h),
            _ => continue,
        };
        top = top.min(y);
        bottom = bottom.max(y + h);
        left = left.min(x);
        right = right.max(x + w);
    }
    ((left + right) / 2.0, (top + bottom) / 2.0)
}

#[test]
fn the_message_sits_in_the_middle_of_the_panel_whatever_its_shape() {
    let screen = RebuildScreen {
        title: Some((TexId::from_raw(1), 401, 29)),
        count: Some((TexId::from_raw(2), 117, 19)),
        fraction: 0.25,
    };
    for canvas in [Canvas::FRAMED, Canvas { w: 640.0, h: 480.0 }] {
        let mut out = Vec::new();
        screen.draw(canvas, &mut out);
        let (x, y) = stack_middle(&out);
        assert_eq!(x, canvas.w / 2.0, "off centre across a {canvas:?} canvas");
        assert_eq!(y, 240.0, "off centre down a {canvas:?} canvas");
    }
}
