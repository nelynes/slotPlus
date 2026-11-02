//! Earned banners and a passive shelf sync icon. Rasterization is off the UI thread.
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use slot_achievements::{Notice, NoticeKind, SyncStatus};
use slot_gfx::{Canvas, Compositor, Draw, TexId};
use slot_ui::{text, CartFace, SyncIndicator};

use crate::app::Phase;
use crate::session::Session;

// slotPlus: drawn like Slot's own HUD plate: a dark translucent band, the label face in
// capitals with Slot's tracking, the title on one line and the description under it.
const WIDTH: u32 = 460;
const HEIGHT: u32 = 56;
const DURATION: Duration = Duration::from_millis(1800);
const BADGE_PX: u32 = 40;
const PAD: u32 = 8;
const TEXT_X: u32 = PAD + BADGE_PX + 12;
const TEXT_W: u32 = WIDTH - TEXT_X - 14;
const TITLE_PX: f32 = 16.0;
const DETAIL_PX: f32 = 12.0;
/// Slot's HUD plate, black at 72%.
const PLATE: [u8; 4] = [0, 0, 0, 184];
const TITLE_INK: [u8; 3] = slot_ui::HUD_INK;
const DETAIL_INK: [u8; 3] = [0xb4, 0xb1, 0xae];

pub(crate) struct Notifications {
    requests: Sender<Request>,
    built: Receiver<Built>,
    waiting: bool,
    texture: Option<TexId>,
    badge: Option<TexId>,
    shown: Option<Instant>,
    sync: Option<[TexId; 2]>,
    started: Instant,
    requested_progress: Option<u8>,
    displayed_progress: Option<u8>,
    progress: Option<[TexId; 2]>,
}

enum Request {
    Earned(Notice, u32),
    Progress(u8),
}

enum Built {
    Progress(u8, [CartFace; 2]),
    Sync([CartFace; 2]),
    Banner(CartFace, CartFace, Vec<i16>),
}

impl Notifications {
    pub fn new() -> Self {
        let (requests, inbox) = mpsc::channel();
        let (outbox, built) = mpsc::channel();
        let _ = std::thread::Builder::new()
            .name("slot-ra-banner".into())
            .spawn(move || {
                if outbox
                    .send(Built::Sync([false, true].map(slot_ui::sync_icon_face)))
                    .is_err()
                {
                    return;
                }
                while let Ok(request) = inbox.recv() {
                    let (notice, rate) = match request {
                        Request::Earned(notice, rate) => (notice, rate),
                        Request::Progress(percent) => {
                            if outbox
                                .send(Built::Progress(
                                    percent,
                                    [false, true].map(|light| progress_face(percent, light)),
                                ))
                                .is_err()
                            {
                                return;
                            }
                            continue;
                        }
                    };
                    let sound = if rate == 0 {
                        Vec::new()
                    } else {
                        crate::audio::Sfx::Achievement.render(rate)
                    };
                    if outbox
                        .send(Built::Banner(face(&notice), badge_face(&notice), sound))
                        .is_err()
                    {
                        return;
                    }
                }
            });
        Self {
            requests,
            built,
            waiting: false,
            texture: None,
            badge: None,
            shown: None,
            sync: None,
            started: Instant::now(),
            requested_progress: None,
            displayed_progress: None,
            progress: None,
        }
    }

    pub fn update(&mut self, session: &mut Session, compositor: &mut Compositor) {
        if matches!(session.app().phase(), Phase::Doze { .. }) || session.app().shutting_down() {
            return;
        }
        let percent = session.achievement_sync_progress();
        if percent != self.requested_progress {
            self.requested_progress = percent;
            if let Some(percent) = percent {
                let _ = self.requests.send(Request::Progress(percent));
            }
        }
        while let Ok(built) = self.built.try_recv() {
            match built {
                Built::Progress(percent, face) => {
                    if self.requested_progress != Some(percent) {
                        continue;
                    }
                    self.progress = Some(match self.progress {
                        Some(ids) => {
                            for (id, face) in ids.into_iter().zip(face) {
                                compositor.update_texture(id, face.w, face.h, &face.rgba);
                            }
                            ids
                        }
                        None => {
                            face.map(|face| compositor.create_texture(face.w, face.h, &face.rgba))
                        }
                    });
                    self.displayed_progress = Some(percent);
                }
                Built::Sync(face) => {
                    self.sync = Some(
                        face.map(|face| compositor.create_texture(face.w, face.h, &face.rgba)),
                    );
                }
                Built::Banner(face, badge, sound) => {
                    self.waiting = false;
                    match self.texture {
                        Some(id) => compositor.update_texture(id, face.w, face.h, &face.rgba),
                        None => {
                            self.texture =
                                Some(compositor.create_texture(face.w, face.h, &face.rgba))
                        }
                    }
                    match self.badge {
                        Some(id) => compositor.update_texture(id, badge.w, badge.h, &badge.rgba),
                        None => {
                            self.badge =
                                Some(compositor.create_texture(badge.w, badge.h, &badge.rgba))
                        }
                    }
                    self.shown = Some(Instant::now());
                    session.mix_sfx(sound);
                }
            }
        }
        let status = session.achievement_sync_status();
        let visible = status == SyncStatus::Syncing || session.achievement_sync_pending();
        let theme = usize::from(session.app().light());
        let icon = self.sync.and_then(|faces| {
            visible.then_some(SyncIndicator {
                face: faces[theme],
                turn: if status == SyncStatus::Syncing {
                    self.started.elapsed().as_secs_f32() % 2.0 * std::f32::consts::PI
                } else {
                    0.0
                },
                alpha: match status {
                    SyncStatus::Syncing => 0.9,
                    _ => 0.35,
                },
                attention: matches!(status, SyncStatus::Offline | SyncStatus::Attention),
                progress: if percent.is_some() && self.displayed_progress.is_some() {
                    self.progress.map(|faces| faces[theme])
                } else {
                    None
                },
            })
        });
        session.app_mut().set_achievement_sync(icon);
    }

    pub fn draw(&mut self, session: &Session, draws: &mut Vec<Draw>) {
        if matches!(session.app().phase(), Phase::Doze { .. }) || session.app().shutting_down() {
            return;
        }
        if self.shown.is_some_and(|time| time.elapsed() >= DURATION) {
            self.shown = None;
        }
        if !self.waiting && self.shown.is_none() {
            while let Some(notice) = session.take_achievement_notice() {
                if notice.kind == NoticeKind::Earned {
                    self.waiting = self
                        .requests
                        .send(Request::Earned(notice, session.audio_ring().sample_rate()))
                        .is_ok();
                    break;
                }
            }
        }
        if let (Some(tex), Some(badge), Some(shown)) = (self.texture, self.badge, self.shown) {
            draw_banner(
                session.app().canvas(),
                tex,
                badge,
                shown.elapsed().as_secs_f32(),
                draws,
            );
        }
    }
}

fn progress_face(percent: u8, light: bool) -> CartFace {
    let mut rgba = vec![0; (36 * slot_ui::HINT_H * 4) as usize];
    if let Some(font) = text::label_font() {
        let layout = text::fit(font, &format!("{percent}%"), 36.0, 1, 12.0, 12.0);
        text::draw_centred(
            &mut rgba,
            36,
            slot_ui::HINT_H,
            &layout,
            slot_ui::footer_ink(light),
        );
    }
    CartFace {
        rgba,
        w: 36,
        h: slot_ui::HINT_H,
    }
}

fn draw_banner(canvas: Canvas, tex: TexId, badge: TexId, elapsed: f32, draws: &mut Vec<Draw>) {
    if elapsed >= DURATION.as_secs_f32() {
        return;
    }
    let enter = (elapsed / 0.20).clamp(0.0, 1.0);
    let exit = ((elapsed - 1.55) / 0.25).clamp(0.0, 1.0);
    let ease = 1.0 - (1.0 - enter).powi(3);
    let alpha = ease * (1.0 - exit * exit);
    let x = canvas.centred(WIDTH as f32);
    let y = canvas.h - HEIGHT as f32 - 12.0 + (1.0 - ease) * 12.0 + exit * exit * 6.0;
    draws.push(Draw::Tex {
        x,
        y,
        w: WIDTH as f32,
        h: HEIGHT as f32,
        tex,
        alpha,
    });
    // One restrained overshoot: the badge settles while the card finishes sliding in.
    let pop = (elapsed / 0.32).clamp(0.0, 1.0);
    let scale = 1.0 + 0.12 * (std::f32::consts::PI * pop).sin();
    let size = BADGE_PX as f32 * scale;
    draws.push(Draw::Tex {
        x: x + PAD as f32 + (BADGE_PX as f32 - size) / 2.0,
        y: y + (HEIGHT as f32 - size) / 2.0,
        w: size,
        h: size,
        tex: badge,
        alpha,
    });
}

fn badge_face(notice: &Notice) -> CartFace {
    let Some(image) = notice
        .badge
        .as_deref()
        .and_then(slot_achievements::load_badge)
        .filter(|i| i.width > 0 && i.height > 0)
    else {
        return slot_ui::achievement_icon_face(BADGE_PX);
    };
    // Badges are 64 px; each output pixel averages the source pixels it covers.
    let mut rgba = vec![0; (BADGE_PX * BADGE_PX * 4) as usize];
    for y in 0..BADGE_PX {
        let (y0, y1) = span(y, image.height);
        for x in 0..BADGE_PX {
            let (x0, x1) = span(x, image.width);
            let mut sum = [0u32; 4];
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let at = ((sy * image.width + sx) * 4) as usize;
                    for (c, total) in sum.iter_mut().enumerate() {
                        *total += image.rgba[at + c] as u32;
                    }
                }
            }
            let n = ((y1 - y0) * (x1 - x0)).max(1);
            let dst = ((y * BADGE_PX + x) * 4) as usize;
            for (c, total) in sum.iter().enumerate() {
                rgba[dst + c] = (total / n) as u8;
            }
        }
    }
    CartFace {
        rgba,
        w: BADGE_PX,
        h: BADGE_PX,
    }
}

/// The source rows (or columns) under output pixel `i` of `BADGE_PX`.
fn span(i: u32, size: u32) -> (u32, u32) {
    let a = i * size / BADGE_PX;
    let b = ((i + 1) * size).div_ceil(BADGE_PX).min(size);
    (a, b.max(a + 1).min(size))
}

/// "Title (+5)" as the service words it: the name and its points.
fn split_points(title: &str) -> (&str, &str) {
    match title.rfind(" (+") {
        Some(at) if title.ends_with(')') => (&title[..at], &title[at..]),
        _ => (title, ""),
    }
}

fn tracking(px: f32) -> f32 {
    (px * 0.10).round()
}

/// `text` in capitals on one line no wider than `max_w`, cut short with "..." when it runs
/// over. `keep` (the points) always stays whole at the end.
fn one_line(font: &fontdue::Font, text: &str, keep: &str, px: f32, max_w: f32) -> String {
    let t = tracking(px);
    let text = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase();
    let whole = format!("{text}{keep}");
    if text::line_width(font, &whole, px, t) <= max_w {
        return whole;
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let cut: String = chars.iter().collect();
        let line = format!("{}...{keep}", cut.trim_end());
        if text::line_width(font, &line, px, t) <= max_w {
            return line;
        }
    }
    keep.trim_start().to_string()
}

/// Draw `line` left-aligned at `x` with its baseline at `baseline`.
#[allow(clippy::too_many_arguments)]
fn print(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    font: &fontdue::Font,
    line: &str,
    px: f32,
    x: f32,
    baseline: f32,
    ink: [u8; 3],
) {
    let t = tracking(px);
    let mut pen = x;
    for ch in line.chars() {
        let (m, cov) = font.rasterize(ch, px);
        let gx = (pen + m.xmin as f32).round() as i32;
        let gy = (baseline - (m.height as f32 + m.ymin as f32)).round() as i32;
        for row in 0..m.height {
            for col in 0..m.width {
                let (dx, dy) = (gx + col as i32, gy + row as i32);
                if dx < 0 || dy < 0 || dx >= w as i32 || dy >= h as i32 {
                    continue;
                }
                let a = cov[row * m.width + col] as u32;
                if a == 0 {
                    continue;
                }
                let at = ((dy as u32 * w + dx as u32) * 4) as usize;
                for c in 0..3 {
                    let under = rgba[at + c] as u32;
                    rgba[at + c] = ((ink[c] as u32 * a + under * (255 - a)) / 255) as u8;
                }
                let under = rgba[at + 3] as u32;
                rgba[at + 3] = (a + under * (255 - a) / 255).min(255) as u8;
            }
        }
        pen += m.advance_width + t;
    }
}

fn face(notice: &Notice) -> CartFace {
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&PLATE);
    }
    if let Some(font) = text::label_font() {
        let (name, points) = split_points(&notice.title);
        let title = one_line(font, name, points, TITLE_PX, TEXT_W as f32);
        let detail = one_line(font, &notice.detail, "", DETAIL_PX, TEXT_W as f32);
        let ascent = |px| {
            font.horizontal_line_metrics(px)
                .map_or(px * 0.8, |m| m.ascent)
        };
        // Cap heights only: the label face is all capitals, so no descenders to allow for.
        let (title_cap, detail_cap) = (ascent(TITLE_PX) * 0.72, ascent(DETAIL_PX) * 0.72);
        let gap = 7.0;
        let block = if detail.is_empty() {
            title_cap
        } else {
            title_cap + gap + detail_cap
        };
        let top = (HEIGHT as f32 - block) / 2.0;
        let x = TEXT_X as f32;
        print(
            &mut rgba,
            WIDTH,
            HEIGHT,
            font,
            &title,
            TITLE_PX,
            x,
            top + title_cap,
            TITLE_INK,
        );
        if !detail.is_empty() {
            let baseline = top + title_cap + gap + detail_cap;
            print(
                &mut rgba, WIDTH, HEIGHT, font, &detail, DETAIL_PX, x, baseline, DETAIL_INK,
            );
        }
    }
    CartFace {
        rgba,
        w: WIDTH,
        h: HEIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slot_gfx::OUT_H;
    #[cfg(target_os = "macos")]
    use slot_gfx::OUT_W;

    #[test]
    fn notification_enters_settles_and_is_gone_before_two_seconds() {
        let tex = TexId::from_raw(0);
        let mut draws = Vec::new();
        draw_banner(Canvas::FRAMED, tex, tex, 0.1, &mut draws);
        assert!(
            matches!(draws[0], Draw::Tex { alpha, y, .. } if alpha > 0.0 && alpha < 1.0 && y > (OUT_H - HEIGHT - 12) as f32)
        );
        draws.clear();
        draw_banner(Canvas::FRAMED, tex, tex, 0.5, &mut draws);
        assert!(matches!(draws[0], Draw::Tex { alpha: 1.0, .. }));
        assert!(matches!(draws[1], Draw::Tex { w, .. } if w == BADGE_PX as f32));
        draws.clear();
        draw_banner(Canvas::FRAMED, tex, tex, 1.7, &mut draws);
        assert!(matches!(draws[0], Draw::Tex { alpha, .. } if alpha > 0.0 && alpha < 1.0));
        draws.clear();
        draw_banner(Canvas::FRAMED, tex, tex, 1.8, &mut draws);
        assert!(draws.is_empty());
    }

    #[test]
    fn notification_is_centred_on_a_native_four_by_three_canvas() {
        let tex = TexId::from_raw(0);
        let mut draws = Vec::new();
        draw_banner(Canvas { w: 640.0, h: 480.0 }, tex, tex, 0.5, &mut draws);
        let Draw::Tex { x, w, .. } = draws[0] else {
            panic!("no banner");
        };
        assert_eq!(x + w / 2.0, 320.0);
    }

    #[test]
    fn sync_progress_uses_the_shelf_theme_ink() {
        for light in [false, true] {
            let f = progress_face(62, light);
            let pixel = f.rgba.chunks_exact(4).max_by_key(|p| p[3]).unwrap();
            assert_eq!(&pixel[..3], &slot_ui::footer_ink(light));
        }
        assert_ne!(
            slot_ui::sync_icon_face(false).rgba,
            slot_ui::sync_icon_face(true).rgba
        );
    }

    #[test]
    fn achievement_banner_renders_a_title_in_a_bounded_panel() {
        let notice = Notice {
            badge: std::env::var_os("SLOT_RA_PREVIEW_BADGE").map(std::path::PathBuf::from),
            generation: 1,
            title: "First Steps (+5)".into(),
            detail: String::new(),
            kind: NoticeKind::Earned,
        };
        let banner = face(&notice);
        assert_eq!(banner.rgba.len(), (WIDTH * HEIGHT * 4) as usize);
        assert!(banner.rgba.chunks_exact(4).filter(|p| p[0] > 150).count() > 100);
        if let Some(dir) = std::env::var_os("SCRATCH_PNG_DIR") {
            let file =
                std::fs::File::create(std::path::Path::new(&dir).join("slot-achievement.png"))
                    .unwrap();
            let mut encoder = png::Encoder::new(file, WIDTH, HEIGHT);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&banner.rgba)
                .unwrap();
        }
    }

    /// Optional native UI exports, using a recorded game frame when one is supplied.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "writes UI previews; set SCRATCH_PNG_DIR and optionally SLOT_RA_PREVIEW_FRAME"]
    fn render_achievement_previews() {
        use crate::app::App;
        use slot_gfx::HeadlessSurface;
        use slot_power::{Power, SimPlatform};

        let dir = std::path::PathBuf::from(std::env::var_os("SCRATCH_PNG_DIR").unwrap());
        let surface = HeadlessSurface::new().unwrap();
        let mut compositor = Compositor::new(&surface).unwrap();
        let save = |name: &str, pixels: &[u8]| {
            let file = std::fs::File::create(dir.join(format!("slot-ra-{name}.png"))).unwrap();
            let mut encoder = png::Encoder::new(file, OUT_W, OUT_H);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(pixels)
                .unwrap();
        };
        let root = tempfile::tempdir().unwrap();
        let carts: Vec<_> = ["Metroid Fusion", "Pokemon Emerald", "Golden Sun"]
            .iter()
            .map(|name| slot_store::Cart {
                platform: slot_store::Platform::Gba,
                stem: (*name).into(),
                title: (*name).into(),
                rom: root.path().join(format!("{name}.gba")),
                label: None,
                code: String::new(),
                shell: None,
                sealed: false,
            })
            .collect();
        let faces = carts
            .iter()
            .map(|cart| {
                let f = slot_ui::cart_face(cart);
                compositor.create_texture(f.w, f.h, &f.rgba)
            })
            .collect();
        let mut app = App::new(carts);
        app.set_faces(faces);
        app.set_power(Power::new(
            Box::new(SimPlatform::at(root.path().into())),
            Duration::from_secs(180),
        ));
        app.update(1.0);
        let clock = slot_ui::word_face("14:32");
        app.set_shelf_clock_face(
            compositor.create_texture(clock.w, clock.h, &clock.rgba),
            clock.w,
        );
        let percent = slot_ui::word_face("100%");
        app.set_battery_percent_face(
            compositor.create_texture(percent.w, percent.h, &percent.rgba),
            percent.w,
        );
        let icon = slot_ui::sync_icon_face(false);
        let icon = compositor.create_texture(icon.w, icon.h, &icon.rgba);
        let mut draws = Vec::new();
        for (name, turn, alpha, attention) in [
            ("sync", 0.3, 0.9, false),
            ("ready", 0.0, 0.55, false),
            ("waiting", 0.0, 0.35, true),
        ] {
            app.set_achievement_sync(Some(SyncIndicator {
                face: icon,
                turn,
                alpha,
                attention,
                progress: if name == "sync" {
                    let f = progress_face(62, false);
                    Some(compositor.create_texture(f.w, f.h, &f.rgba))
                } else {
                    None
                },
            }));
            draws.clear();
            app.draw(&mut draws);
            compositor.begin_frame();
            compositor.draw_list(&draws);
            save(name, &compositor.read_frame());
        }
        let notice = Notice {
            badge: std::env::var_os("SLOT_RA_PREVIEW_BADGE").map(std::path::PathBuf::from),
            generation: 1,
            title: "First Steps (+5)".into(),
            detail: String::new(),
            kind: NoticeKind::Earned,
        };
        let banner = face(&notice);
        let mut notifications = Notifications::new();
        notifications.texture = Some(compositor.create_texture(banner.w, banner.h, &banner.rgba));
        let badge = badge_face(&notice);
        notifications.badge = Some(compositor.create_texture(badge.w, badge.h, &badge.rgba));
        notifications.shown = Some(Instant::now() - Duration::from_millis(500));
        draws.clear();
        if let Some(path) = std::env::var_os("SLOT_RA_PREVIEW_FRAME") {
            let rgba = slot_ui::wallpaper_face(std::path::Path::new(&path), OUT_W, OUT_H).unwrap();
            let tex = compositor.create_texture(OUT_W, OUT_H, &rgba);
            draws.push(Draw::Tex {
                x: 0.0,
                y: 0.0,
                w: OUT_W as f32,
                h: OUT_H as f32,
                tex,
                alpha: 1.0,
            });
        }
        notifications.draw(&Session::boot(root.path().into()), &mut draws);
        compositor.begin_frame();
        compositor.draw_list(&draws);
        save("earned", &compositor.read_frame());
        if std::env::var_os("SLOT_RA_PREVIEW_ANIMATE").is_some() {
            // Capture the actual drawing path at 30 fps, including a clean frame at the end.
            draws.truncate(draws.len() - 2);
            for frame in 0..=60 {
                let mut animated = draws.clone();
                draw_banner(
                    app.canvas(),
                    notifications.texture.unwrap(),
                    notifications.badge.unwrap(),
                    frame as f32 / 30.0,
                    &mut animated,
                );
                compositor.begin_frame();
                compositor.draw_list(&animated);
                save(&format!("animation-{frame:03}"), &compositor.read_frame());
            }
        }
    }
}
