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

/// One of the DS's two screens.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DsScreen {
    #[default]
    Top,
    Bottom,
}

impl DsScreen {
    pub fn other(self) -> DsScreen {
        match self {
            DsScreen::Top => DsScreen::Bottom,
            DsScreen::Bottom => DsScreen::Top,
        }
    }

    /// Where the screen sits in the DS's 720x480 buffer, as a fraction of
    /// it: DSperate draws the two stacked, 256x384, at 232, 48.
    pub fn source_rect(self) -> [f32; 4] {
        let y = match self {
            DsScreen::Top => 48.0,
            DsScreen::Bottom => 240.0,
        };
        [232.0 / 720.0, y / 480.0, 256.0 / 720.0, 192.0 / 480.0]
    }
}

/// How a DS game's two screens are laid out over the 720x480 canvas.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DsLayout {
    /// Both, the main one above the other, filling the height at 1.25x.
    #[default]
    Stacked,
    /// The main screen alone at 2x.
    Single,
    /// The main screen alone, as big as the canvas takes at 4:3: 2.5x.
    Fill,
    /// The main screen at 2x, the other small beside it.
    Side,
    /// The main screen filling the height, the other at 1x in its corner.
    Corner,
}

impl DsLayout {
    pub const ALL: [DsLayout; 5] = [
        DsLayout::Stacked,
        DsLayout::Single,
        DsLayout::Fill,
        DsLayout::Side,
        DsLayout::Corner,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            DsLayout::Stacked => "stacked",
            DsLayout::Single => "single",
            DsLayout::Fill => "fill",
            DsLayout::Side => "side",
            DsLayout::Corner => "corner",
        }
    }

    /// Its place in `DsLayout::ALL`, which slot-ui's `DS_LAYOUTS` names.
    pub fn index(self) -> usize {
        DsLayout::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }

    fn parse(s: &str) -> Option<DsLayout> {
        DsLayout::ALL.into_iter().find(|l| l.as_str() == s)
    }

    /// The layout SELECT + L2 steps to.
    pub fn next(self) -> DsLayout {
        DsLayout::ALL[(self.index() + 1) % DsLayout::ALL.len()]
    }
}

/// One screen drawn: the part of the buffer it is, and where on the 720x480
/// canvas it goes, in pixels.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DsPart {
    pub screen: DsScreen,
    pub src: [f32; 4],
    pub dst: [f32; 4],
}

/// A DS game's view of its two screens: which is the main one, L2 swaps
/// them, and how they are laid out, which SELECT + L2 steps through.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DsView {
    pub main: DsScreen,
    pub layout: DsLayout,
}

impl DsView {
    pub fn as_str(self) -> String {
        let main = match self.main {
            DsScreen::Top => "top",
            DsScreen::Bottom => "bottom",
        };
        format!("{},{main}", self.layout.as_str())
    }

    /// Reads a saved view, the views before layouts among them: "both" was
    /// both stacked, "top" and "bottom" one screen at 2x.
    pub fn parse(s: &str) -> Option<DsView> {
        let s = s.trim().to_ascii_lowercase();
        let view = |layout, main| Some(DsView { main, layout });
        match s.as_str() {
            "both" => return view(DsLayout::Stacked, DsScreen::Top),
            "top" => return view(DsLayout::Single, DsScreen::Top),
            "bottom" => return view(DsLayout::Single, DsScreen::Bottom),
            _ => {}
        }
        let (layout, main) = s.split_once(',')?;
        let main = match main {
            "top" => DsScreen::Top,
            "bottom" => DsScreen::Bottom,
            _ => return None,
        };
        view(DsLayout::parse(layout)?, main)
    }

    pub fn swapped(self) -> DsView {
        DsView {
            main: self.main.other(),
            ..self
        }
    }

    pub fn next_layout(self) -> DsView {
        DsView {
            layout: self.layout.next(),
            ..self
        }
    }

    /// The screens drawn: the main one, and the other where the layout shows
    /// it too, drawn over the main one.
    pub fn parts(self) -> (DsPart, Option<DsPart>) {
        let part = |screen: DsScreen, dst| DsPart {
            screen,
            src: screen.source_rect(),
            dst,
        };
        let (main, other) = (self.main, self.main.other());
        match self.layout {
            DsLayout::Stacked => (
                part(main, [200.0, 0.0, 320.0, 240.0]),
                Some(part(other, [200.0, 240.0, 320.0, 240.0])),
            ),
            DsLayout::Single => (part(main, [104.0, 48.0, 512.0, 384.0]), None),
            DsLayout::Fill => (part(main, [40.0, 0.0, 640.0, 480.0]), None),
            // 512 at 2x leaves 208 beside it: the other screen at 208x156.
            DsLayout::Side => (
                part(main, [0.0, 48.0, 512.0, 384.0]),
                Some(part(other, [512.0, 48.0, 208.0, 156.0])),
            ),
            DsLayout::Corner => (
                part(main, [40.0, 0.0, 640.0, 480.0]),
                Some(part(other, [424.0, 288.0, 256.0, 192.0])),
            ),
        }
    }

    /// Whether the main screen is drawn big enough for a shader: from 2x.
    /// Both stacked, at 1.25x, has no room for its cells, and the other
    /// screen, drawn over the main one, is always plain.
    pub fn shaded(self) -> bool {
        self.layout != DsLayout::Stacked
    }
}

pub fn ds_view_for(root: &Path, stem: &str) -> DsView {
    ini::value(root, DS_VIEW_FILE, stem)
        .as_deref()
        .and_then(DsView::parse)
        .unwrap_or_default()
}

pub fn write_ds_view(root: &Path, stem: &str, view: DsView) -> std::io::Result<()> {
    ini::write(root, DS_VIEW_FILE, stem, &view.as_str())
}
