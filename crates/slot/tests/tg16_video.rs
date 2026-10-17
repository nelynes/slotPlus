use slot::video_mode::{picture_of, source_rect_shown, VideoMode};
use slot_gfx::WHOLE_TEXTURE;
use slot_store::Platform;

const BW: f32 = 720.0;

#[test]
fn a_256_wide_hucard_picture_lands_at_2x_in_actual_size() {
    assert_eq!(
        source_rect_shown(Platform::Tg16, VideoMode::Actual, false, (256, 240)),
        [180.0 / BW, 0.0, 360.0 / BW, 1.0],
        "the 360x240 window in the middle of the buffer, drawn at 2x"
    );
}

#[test]
fn a_320_or_352_wide_picture_keeps_the_same_2x_window() {
    for w in [320, 336, 352] {
        assert_eq!(
            source_rect_shown(Platform::Tg16, VideoMode::Actual, false, (w, 240)),
            [180.0 / BW, 0.0, 360.0 / BW, 1.0],
            "{w} wide"
        );
    }
}

#[test]
fn a_512_wide_high_resolution_picture_shows_the_whole_buffer_at_1x_across() {
    assert_eq!(
        source_rect_shown(Platform::Tg16, VideoMode::Actual, false, (512, 240)),
        WHOLE_TEXTURE
    );
}

#[test]
fn stretch_fills_the_screen_with_whatever_width_the_game_draws() {
    for w in [256u32, 320, 352, 512] {
        let x = (720 - w) / 2;
        assert_eq!(
            source_rect_shown(Platform::Tg16, VideoMode::Stretch, false, (w, 240)),
            [x as f32 / BW, 0.0, w as f32 / BW, 1.0],
            "{w} wide"
        );
    }
}

#[test]
fn before_the_first_frame_the_usual_256_by_240_picture_is_assumed() {
    assert_eq!(picture_of(Platform::Tg16, (0, 0)), (256, 240));
    assert_eq!(
        source_rect_shown(Platform::Tg16, VideoMode::Stretch, false, (0, 0)),
        [232.0 / BW, 0.0, 256.0 / BW, 1.0]
    );
}

#[test]
fn other_platforms_ignore_the_frame_width() {
    for p in [Platform::Gba, Platform::Gg, Platform::Ngp, Platform::Ws] {
        for mode in [VideoMode::Actual, VideoMode::Stretch] {
            assert_eq!(
                source_rect_shown(p, mode, false, (512, 240)),
                slot::video_mode::source_rect(p, mode),
                "{p:?} {mode:?}"
            );
        }
    }
}
