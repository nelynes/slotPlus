use slot::video_mode::{source_rect, source_rect_for, VideoMode};
use slot_gfx::{OUT_H, OUT_W, SRC_H, SRC_W, WHOLE_TEXTURE};
use slot_store::Platform;

#[test]
fn every_platform_but_the_lynx_still_actual_sizes_into_the_whole_3x_window() {
    for p in [Platform::Gba, Platform::Gb, Platform::Gbc, Platform::Ws] {
        assert_eq!(
            p.actual_window(),
            (SRC_W, SRC_H),
            "{p:?} changed its window"
        );
    }
    assert_eq!(source_rect(Platform::Gba, VideoMode::Actual), WHOLE_TEXTURE);
    assert_eq!(source_rect(Platform::Gb, VideoMode::Actual), WHOLE_TEXTURE);
}

#[test]
fn a_landscape_lynx_is_shown_at_4x() {
    let (w, h) = Platform::Lynx.actual_window();
    assert_eq!((OUT_W / w, OUT_H / h), (4, 4));
    assert_eq!(
        (OUT_W % w, OUT_H % h),
        (0, 0),
        "4x has to land on the canvas"
    );
    assert_eq!(
        source_rect(Platform::Lynx, VideoMode::Actual),
        [30.0 / 240.0, 20.0 / 160.0, 180.0 / 240.0, 120.0 / 160.0]
    );
}

#[test]
fn the_lynx_picture_sits_in_the_middle_of_its_4x_window() {
    let (pw, ph) = Platform::Lynx.picture();
    let (ww, wh) = Platform::Lynx.actual_window();
    assert_eq!((pw, ph), (160, 102));
    assert!(ww >= pw && wh >= ph);
    assert_eq!((ww - pw) % 2, 0, "the picture would land between columns");
    assert_eq!((wh - ph) % 2, 0, "the picture would land between rows");
}

#[test]
fn a_landscape_lynx_can_still_be_stretched_to_its_picture() {
    assert_eq!(
        source_rect(Platform::Lynx, VideoMode::Stretch),
        [40.0 / 240.0, 29.0 / 160.0, 160.0 / 240.0, 102.0 / 160.0]
    );
}

#[test]
fn a_turned_lynx_shows_the_whole_buffer_whichever_mode() {
    for mode in [VideoMode::Actual, VideoMode::Stretch] {
        assert_eq!(source_rect_for(Platform::Lynx, mode, true), WHOLE_TEXTURE);
    }
}

#[test]
fn a_turned_lynx_frame_fits_the_buffer_it_shares_with_the_gba() {
    assert_eq!(Platform::Lynx.source_size(), (240, 160));
    let (pw, ph) = Platform::Lynx.picture();
    assert!(ph <= 160 && pw <= 240, "the turned frame is {ph} by {pw}");
    assert_eq!((OUT_W / 240, OUT_H / 160), (3, 3), "the buffer lands at 3x");
}

#[test]
fn only_the_wonderswan_and_the_lynx_can_be_turned() {
    for p in Platform::ALL {
        assert_eq!(
            p.turns(),
            matches!(p, Platform::Ws | Platform::Lynx),
            "{p:?}"
        );
    }
}
