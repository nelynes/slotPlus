use slot_retro::{GBA_H, GBA_W};

pub fn png(xrgb8888: &[u8]) -> Option<Vec<u8>> {
    png_of(xrgb8888, (GBA_W, GBA_H))
}

/// Thumbnail of a video buffer that is `size` pixels. The thumbnail is always
/// GBA-sized: a bigger buffer (WonderSwan's) is cropped to its centre window.
pub fn png_of(xrgb8888: &[u8], size: (u32, u32)) -> Option<Vec<u8>> {
    let (bw, bh) = (size.0 as usize, size.1 as usize);
    let (tw, th) = (GBA_W as usize, GBA_H as usize);
    if bw < tw || bh < th || xrgb8888.len() < bw * bh * 4 {
        return None;
    }
    let (ox, oy) = ((bw - tw) / 2, (bh - th) / 2);
    let mut rgb = Vec::with_capacity(tw * th * 3);
    for y in 0..th {
        let row = ((y + oy) * bw + ox) * 4;
        for px in xrgb8888[row..row + tw * 4].chunks_exact(4) {
            rgb.extend_from_slice(&[px[2], px[1], px[0]]);
        }
    }

    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, GBA_W, GBA_H);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().ok()?;
    writer.write_image_data(&rgb).ok()?;
    writer.finish().ok()?;
    Some(out)
}
