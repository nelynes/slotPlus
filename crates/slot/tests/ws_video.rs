use slot::frames::Frames;
use slot::thumb;
use slot::video_mode::{source_rect, source_rect_for, VideoMode};
use slot_gfx::WHOLE_TEXTURE;
use slot_store::Platform;

#[test]
fn gba_and_game_boy_views_are_unchanged() {
    assert_eq!(source_rect(Platform::Gba, VideoMode::Actual), WHOLE_TEXTURE);
    assert_eq!(
        source_rect(Platform::Gba, VideoMode::Stretch),
        WHOLE_TEXTURE
    );
    assert_eq!(source_rect(Platform::Gb, VideoMode::Actual), WHOLE_TEXTURE);
    assert_eq!(
        source_rect(Platform::Gb, VideoMode::Stretch),
        [40.0 / 240.0, 8.0 / 160.0, 160.0 / 240.0, 144.0 / 160.0]
    );
}

#[test]
fn a_landscape_wonderswan_shows_the_centre_240_by_160_at_3x() {
    assert_eq!(
        source_rect(Platform::Ws, VideoMode::Actual),
        [60.0 / 360.0, 40.0 / 240.0, 240.0 / 360.0, 160.0 / 240.0]
    );
}

#[test]
fn a_landscape_wonderswan_can_still_be_stretched_to_its_picture() {
    assert_eq!(
        source_rect(Platform::Ws, VideoMode::Stretch),
        [68.0 / 360.0, 48.0 / 240.0, 224.0 / 360.0, 144.0 / 240.0]
    );
}

#[test]
fn a_turned_wonderswan_shows_the_whole_buffer_whichever_mode() {
    for mode in [VideoMode::Actual, VideoMode::Stretch] {
        assert_eq!(source_rect_for(Platform::Ws, mode, true), WHOLE_TEXTURE);
    }
}

#[test]
fn frames_know_their_buffer_and_the_picture_in_it() {
    let f = Frames::sized(360, 240);
    assert_eq!(f.dims(), (360, 240));
    assert_eq!(f.shown(), (360, 240));
    f.set_shown((144, 224));
    let mut buf = f.take_write();
    buf.resize(360 * 240 * 4, 7);
    f.publish(buf);
    let frame = f.latest().expect("a frame");
    assert_eq!(frame.dims(), (360, 240));
    assert_eq!(frame.shown(), (144, 224));
}

#[test]
fn a_big_buffer_is_thumbnailed_from_its_centre() {
    let (bw, bh) = (360usize, 240usize);
    let mut frame = vec![0u8; bw * bh * 4];
    for y in 0..bh {
        for x in 0..bw {
            let o = (y * bw + x) * 4;
            let inside = (60..300).contains(&x) && (40..200).contains(&y);
            frame[o..o + 4].copy_from_slice(if inside {
                &[0x20, 0x40, 0xd0, 0xff]
            } else {
                &[0, 0, 0, 0xff]
            });
        }
    }
    let png_bytes = thumb::png_of(&frame, (360, 240)).expect("encode");
    let mut reader = png::Decoder::new(png_bytes.as_slice())
        .read_info()
        .expect("read info");
    let mut rgb = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut rgb).expect("decode");
    assert_eq!((info.width, info.height), (240, 160));
    assert_eq!(
        &rgb[..3],
        &[0xd0, 0x40, 0x20],
        "the corner is not the centre window's"
    );
    assert!(thumb::png_of(&frame[..100], (360, 240)).is_none());
}
