use std::path::Path;

use slot_gfx::{Sizing, SRC_H, SRC_W, WHOLE_TEXTURE};
use slot_store::{ini, Platform, Scale, Stretch};

pub const VIDEO_MODE_FILE: &str = "Config/video_mode.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VideoMode {
    #[default]
    Actual,
    Stretch,
}

impl VideoMode {
    pub fn as_str(self) -> &'static str {
        match self {
            VideoMode::Actual => "actual",
            VideoMode::Stretch => "stretch",
        }
    }

    pub fn parse(s: &str) -> Option<VideoMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "actual" => Some(VideoMode::Actual),
            "stretch" => Some(VideoMode::Stretch),
            _ => None,
        }
    }
}

pub fn video_mode_for(root: &Path, stem: &str) -> VideoMode {
    ini::value(root, VIDEO_MODE_FILE, stem)
        .as_deref()
        .and_then(VideoMode::parse)
        .unwrap_or_default()
}

pub fn write_video_mode(root: &Path, stem: &str, mode: VideoMode) -> std::io::Result<()> {
    ini::write(root, VIDEO_MODE_FILE, stem, mode.as_str())
}

/// Platforms whose stretch widens the picture only, keeping every line and
/// the band above and below it: the Game Boy and Game Boy Color, as Slot
/// 1.7.0 does. Every other platform's stretch fills the screen both ways.
fn widens_only(platform: Platform) -> bool {
    matches!(platform, Platform::Gb | Platform::Gbc)
}

pub fn source_rect(platform: Platform, mode: VideoMode) -> [f32; 4] {
    source_rect_for(platform, mode, false)
}

/// Which part of the platform's video buffer to stretch over the canvas.
/// A vertical frame (`portrait`) always shows the whole buffer, which is
/// sized so it lands at an exact 2x.
pub fn source_rect_for(platform: Platform, mode: VideoMode, portrait: bool) -> [f32; 4] {
    source_rect_shown(platform, mode, portrait, (0, 0))
}

/// The picture a game draws, for a platform whose games change their width:
/// a TurboGrafx-16 game draws 256, 320, 336 or 512 pixels wide, and the frame
/// it last drew (`shown`) says which. Every other platform's picture is fixed.
pub fn picture_of(platform: Platform, shown: (u32, u32)) -> (u32, u32) {
    let (bw, bh) = platform.source_size();
    match platform {
        Platform::Tg16 if shown.0 >= 64 && shown.1 >= 64 => (shown.0.min(bw), shown.1.min(bh)),
        _ => platform.picture(),
    }
}

/// The part of the buffer an "actual size" picture is drawn into. A
/// TurboGrafx-16 picture up to 360 wide gets the 360x240 window, so it lands
/// at 2x; a wider, high resolution one gets the whole 720x240 buffer, so its
/// narrow pixels are drawn at 1x across and 2x down.
fn actual_window_of(platform: Platform, shown: (u32, u32)) -> (u32, u32) {
    match platform {
        Platform::Tg16 if picture_of(platform, shown).0 > platform.actual_window().0 => {
            platform.source_size()
        }
        _ => platform.actual_window(),
    }
}

/// As `source_rect_for`, given the size of the frame the core last drew.
pub fn source_rect_shown(
    platform: Platform,
    mode: VideoMode,
    portrait: bool,
    shown: (u32, u32),
) -> [f32; 4] {
    let (bw, bh) = platform.source_size();
    if portrait {
        return WHOLE_TEXTURE;
    }
    let (w, h) = match mode {
        VideoMode::Actual => actual_window_of(platform, shown),
        VideoMode::Stretch => picture_of(platform, shown),
    };
    if (w, h) == (bw, bh) {
        return WHOLE_TEXTURE;
    }
    let x = bw.saturating_sub(w) / 2;
    if mode == VideoMode::Stretch && widens_only(platform) {
        // A Game Boy picture is stretched sideways only, as a GBA's is,
        // keeping all 144 lines and the dark band above and below them.
        return [x as f32 / bw as f32, 0.0, w as f32 / bw as f32, 1.0];
    }
    let y = bh.saturating_sub(h) / 2;
    [
        x as f32 / bw as f32,
        y as f32 / bh as f32,
        w as f32 / bw as f32,
        h as f32 / bh as f32,
    ]
}

/// How the game picture is sized on the panel, and which part of the buffer
/// it comes from. The Game Boy family follows Scale and Stretch (Slot's).
/// slotPlus's systems keep their own buffers and their own stretch, from
/// `Config/video_mode.txt`, drawn over the band as before.
pub fn picture_for(
    platform: Platform,
    mode: VideoMode,
    scale: Scale,
    stretch: Stretch,
) -> (Sizing, [f32; 4]) {
    if !matches!(
        platform.hardware(),
        Platform::Gba | Platform::Gb | Platform::Gbc
    ) {
        return (Sizing::Band, source_rect(platform, mode));
    }
    let (w, h) = platform.picture();
    let x = SRC_W.saturating_sub(w) as f32 / 2.0 / SRC_W as f32;
    let y = SRC_H.saturating_sub(h) as f32 / 2.0 / SRC_H as f32;
    let (fw, fh) = (w as f32 / SRC_W as f32, h as f32 / SRC_H as f32);
    let crop = [x, y, fw, fh];
    let window = [x, 0.0, fw, 1.0];
    if platform.hardware() == Platform::Gba {
        let sizing = match scale {
            Scale::Integer => Sizing::Integer,
            Scale::FullScreen => Sizing::Panel,
            Scale::SpFrame | Scale::Fit => Sizing::Band,
        };
        return (sizing, WHOLE_TEXTURE);
    }
    match (mode, scale, stretch) {
        (VideoMode::Actual, Scale::Integer, _) => (Sizing::Integer, crop),
        (VideoMode::Actual, Scale::Fit, _) => (Sizing::Fit, crop),
        (VideoMode::Actual, _, _) => (Sizing::Band, WHOLE_TEXTURE),
        (VideoMode::Stretch, _, Stretch::Classic) => (Sizing::Band, window),
        (VideoMode::Stretch, _, Stretch::FullScreen) => (Sizing::Panel, crop),
    }
}

/// Which of a DS game's screens are shown, kept per game.
pub const DS_VIEW_FILE: &str = "Config/nds_view.txt";

/// A DS game's view of its two screens. DSperate draws them stacked, 256x384,
/// in the middle of the DS's 720x480 buffer, and each view is a window onto it:
/// both screens filling the height at 1.25x, or one of them at 2x.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DsView {
    #[default]
    Both,
    Top,
    Bottom,
}

impl DsView {
    pub fn as_str(self) -> &'static str {
        match self {
            DsView::Both => "both",
            DsView::Top => "top",
            DsView::Bottom => "bottom",
        }
    }

    pub fn parse(s: &str) -> Option<DsView> {
        match s.trim().to_ascii_lowercase().as_str() {
            "both" => Some(DsView::Both),
            "top" => Some(DsView::Top),
            "bottom" => Some(DsView::Bottom),
            _ => None,
        }
    }

    /// The view L2 steps to: both, the top screen, the bottom screen, round.
    pub fn next(self) -> DsView {
        match self {
            DsView::Both => DsView::Top,
            DsView::Top => DsView::Bottom,
            DsView::Bottom => DsView::Both,
        }
    }

    /// The part of the 720x480 buffer the view shows over the canvas.
    pub fn source_rect(self) -> [f32; 4] {
        match self {
            // 576x384 of the buffer: the 256x384 pair lands at 1.25x, 320x480.
            DsView::Both => [0.1, 0.1, 0.8, 0.8],
            // 360x240 around the screen at 232..488 x 48..240: 2x, 512x384.
            DsView::Top => [0.25, 0.05, 0.5, 0.5],
            // 360x240 around the screen at 232..488 x 240..432.
            DsView::Bottom => [0.25, 0.45, 0.5, 0.5],
        }
    }
}

pub fn ds_view_for(root: &Path, stem: &str) -> DsView {
    ini::value(root, DS_VIEW_FILE, stem)
        .as_deref()
        .and_then(DsView::parse)
        .unwrap_or_default()
}

pub fn write_ds_view(root: &Path, stem: &str, view: DsView) -> std::io::Result<()> {
    ini::write(root, DS_VIEW_FILE, stem, view.as_str())
}
