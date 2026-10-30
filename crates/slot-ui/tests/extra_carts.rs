//! The carts slotPlus adds to Slot's three: the WonderSwan, the Atari Lynx, the
//! Game Gear, the NeoGeo Pocket and the TurboGrafx-16, drawn from their own art in the same style as the GBA and
//! Game Boy carts.

use slot_store::{Cart, Platform};
use slot_ui::{
    cart_box, cart_face, cart_interior, is_clear, label_size, mould_backing, mould_for, mould_of,
    mould_shadow, seated_box, shell_for, shows_interior, Finish, Mould, Shell, GB_CART_H,
    GG_CART_H, GG_CART_W, GG_LABEL_H, GG_LABEL_W, GG_LABEL_X, GG_LABEL_Y, GG_SHELL, LYNX_CART_H,
    LYNX_CART_W, LYNX_LABEL_H, LYNX_LABEL_W, LYNX_LABEL_X, LYNX_LABEL_Y, NGP_CART_H, NGP_CART_W,
    NGP_LABEL_H, NGP_LABEL_W, NGP_LABEL_X, NGP_LABEL_Y, NGP_SHELL, TG16_CART_H, TG16_CART_W,
    TG16_LABEL_H, TG16_LABEL_W, TG16_LABEL_X, TG16_LABEL_Y, TG16_SHELL, WS_CART_H, WS_CART_W,
    WS_LABEL_H, WS_LABEL_W, WS_LABEL_X, WS_LABEL_Y,
};

fn cart(platform: Platform, stem: &str) -> Cart {
    Cart {
        platform,
        stem: stem.into(),
        rom: format!("Games/x/{stem}.rom").into(),
        label: None,
        title: String::new(),
        code: String::new(),
        shell: None,
    }
}

/// A WonderSwan Color game: its cart is the frosted clear one.
fn ws() -> Cart {
    Cart {
        rom: "Games/WS/Gunpey EX.wsc".into(),
        ..cart(Platform::Ws, "Gunpey EX")
    }
}

/// A game for the original WonderSwan: its cart is black.
fn ws_mono() -> Cart {
    Cart {
        rom: "Games/WS/Gunpey.ws".into(),
        ..cart(Platform::Ws, "Gunpey")
    }
}

fn lynx() -> Cart {
    cart(Platform::Lynx, "California Games")
}

fn gg() -> Cart {
    cart(Platform::Gg, "Sonic Chaos")
}

fn ngp() -> Cart {
    cart(Platform::Ngp, "Metal Slug 1st Mission")
}

fn tg16() -> Cart {
    cart(Platform::Tg16, "Bonk's Adventure")
}

fn nds() -> Cart {
    cart(Platform::Nds, "Mario Kart DS")
}

fn at(face: &slot_ui::CartFace, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * face.w + x) * 4) as usize;
    [
        face.rgba[i],
        face.rgba[i + 1],
        face.rgba[i + 2],
        face.rgba[i + 3],
    ]
}

fn gold(px: [u8; 4]) -> bool {
    px[0] > 150 && px[1] > 110 && px[2] < 110 && px[0] > px[2] + 60
}

#[test]
fn each_cart_has_its_own_mould() {
    assert_eq!(mould_of(&ws()), Mould::Ws);
    assert_eq!(mould_of(&lynx()), Mould::Lynx);
    assert_eq!(mould_of(&gg()), Mould::Gg);
    assert_eq!(mould_for(Platform::Gg), Mould::Gg);
    assert_eq!(mould_of(&ngp()), Mould::Ngp);
    assert_eq!(mould_for(Platform::Ngp), Mould::Ngp);
    assert_eq!(mould_of(&tg16()), Mould::Tg16);
    assert_eq!(mould_for(Platform::Tg16), Mould::Tg16);
    assert_eq!(mould_of(&nds()), Mould::Nds);
    assert_eq!(mould_for(Platform::Nds), Mould::Nds);
    assert_eq!(mould_for(Platform::Ws), Mould::Ws);
    assert_eq!(mould_for(Platform::Lynx), Mould::Lynx);
    assert_eq!(mould_for(Platform::Gba), Mould::Gba);
    assert_eq!(
        Mould::EXTRA,
        [
            Mould::Ws,
            Mould::Lynx,
            Mould::Gg,
            Mould::Ngp,
            Mould::Tg16,
            Mould::Nds
        ]
    );
    for (i, m) in Mould::ALL.iter().enumerate() {
        assert_eq!(m.index(), i);
    }
}

#[test]
fn the_wonderswan_cart_is_wide_and_the_lynx_card_nearly_square() {
    assert_eq!(cart_box(Platform::Ws), (WS_CART_W, WS_CART_H));
    assert_eq!(cart_box(Platform::Lynx), (LYNX_CART_W, LYNX_CART_H));
    assert!(
        WS_CART_W * 10 >= WS_CART_H * 16,
        "the WonderSwan cart is wide"
    );
    assert!(LYNX_CART_H * 100 <= LYNX_CART_W * 110 && LYNX_CART_H >= LYNX_CART_W);
    assert!(LYNX_CART_H <= GB_CART_H, "no taller than the Game Boy pak");
    for c in [ws(), lynx(), gg(), ngp(), tg16()] {
        let f = cart_face(&c);
        assert_eq!((f.w, f.h), cart_box(c.platform), "{:?}", c.platform);
        let (sw, sh) = seated_box(c.platform);
        assert_eq!(sw, 240);
        let (cw, ch) = cart_box(c.platform);
        assert!((sh as f32 - ch as f32 * 240.0 / cw as f32).abs() <= 1.0);
    }
}

#[test]
fn a_ds_card_seats_narrower_keeping_its_shape() {
    let c = nds();
    let (sw, sh) = seated_box(c.platform);
    assert_eq!(sw, slot_ui::NDS_SEATED_W);
    assert!(sw < 240, "a DS card is smaller than a GBA cart");
    let (cw, ch) = cart_box(c.platform);
    assert!((sh as f32 - ch as f32 * sw as f32 / cw as f32).abs() <= 1.0);
}

#[test]
fn the_label_sits_inside_the_shell_and_is_the_size_the_cart_asks_art_for() {
    for (c, (lx, ly, lw, lh)) in [
        (ws(), (WS_LABEL_X, WS_LABEL_Y, WS_LABEL_W, WS_LABEL_H)),
        (
            lynx(),
            (LYNX_LABEL_X, LYNX_LABEL_Y, LYNX_LABEL_W, LYNX_LABEL_H),
        ),
        (gg(), (GG_LABEL_X, GG_LABEL_Y, GG_LABEL_W, GG_LABEL_H)),
        (ngp(), (NGP_LABEL_X, NGP_LABEL_Y, NGP_LABEL_W, NGP_LABEL_H)),
        (
            tg16(),
            (TG16_LABEL_X, TG16_LABEL_Y, TG16_LABEL_W, TG16_LABEL_H),
        ),
    ] {
        assert_eq!(label_size(&c), (lw, lh));
        let f = cart_face(&c);
        for (x, y) in [
            (lx, ly),
            (lx + lw - 1, ly),
            (lx, ly + lh - 1),
            (lx + lw - 1, ly + lh - 1),
        ] {
            assert_eq!(
                at(&f, x, y)[3],
                255,
                "{:?} label corner {x},{y}",
                c.platform
            );
        }
        assert_eq!(at(&f, 0, 0)[3], 0, "{:?} has a square corner", c.platform);
    }
    // The WonderSwan label is a wide strip, the Lynx one nearly square.
    assert!(WS_LABEL_W > WS_LABEL_H * 2);
    assert!(LYNX_LABEL_W * 10 < LYNX_LABEL_H * 18);
}

#[test]
fn the_wonderswan_foot_is_narrower_than_its_shell() {
    let f = cart_face(&ws());
    let width = |y: u32| (0..f.w).filter(|&x| at(&f, x, y)[3] > 128).count();
    let body = width(f.h / 2);
    let foot = width(f.h - 4);
    assert!(foot + 12 < body, "body {body}px, foot {foot}px");
}

#[test]
fn the_contacts_stay_gold_whatever_colour_the_shell() {
    let mut red = ws();
    red.shell = Some(slot_store::ShellChoice {
        colour: [0xc0, 0x20, 0x20],
        finish: slot_store::ShellFinish::Solid,
        outline: slot_store::Outline::Auto,
        back: None,
    });
    for (c, y) in [(ws(), WS_CART_H * 92 / 100), (red, WS_CART_H * 92 / 100)] {
        let f = cart_face(&c);
        let n = (0..f.w).filter(|&x| gold(at(&f, x, y))).count();
        assert!(
            n > 100,
            "only {n} gold pixels across the WonderSwan contacts"
        );
    }
    let f = cart_face(&lynx());
    let y = LYNX_CART_H * 91 / 100;
    let n = (0..f.w).filter(|&x| gold(at(&f, x, y))).count();
    assert!(n > 100, "only {n} gold pixels across the Lynx contacts");
    let above = LYNX_CART_H * 10 / 100;
    assert_eq!((0..f.w).filter(|&x| gold(at(&f, x, above))).count(), 0);
}

#[test]
fn the_wonderswan_is_smoke_clear_and_shows_its_board_and_the_lynx_is_solid() {
    assert!(is_clear(&ws()));
    assert!(shows_interior(&ws()));
    assert_ne!(shell_for(&ws()).finish, Finish::Solid);
    assert!(!is_clear(&lynx()));
    let inside = cart_interior(&ws());
    assert_eq!((inside.w, inside.h), (WS_CART_W, WS_CART_H));
    for m in Mould::EXTRA {
        let (w, h) = m.size();
        let back = mould_backing(m);
        assert_eq!((back.w, back.h), (w, h), "{m:?} backing");
        assert!(
            back.rgba.chunks(4).any(|p| p[3] > 0),
            "{m:?} board is blank"
        );
        let shadow = mould_shadow(m);
        assert_eq!((shadow.w, shadow.h), (w, h), "{m:?} shadow");
        assert!(shadow.rgba.chunks(4).all(|p| p[..3] == [0, 0, 0]));
    }
}

#[test]
fn a_shell_colour_tints_the_plastic_but_not_the_label() {
    let plain = cart_face(&lynx());
    let mut tinted = lynx();
    tinted.shell = Some(slot_store::ShellChoice {
        colour: [0x20, 0x60, 0xd0],
        finish: slot_store::ShellFinish::Solid,
        outline: slot_store::Outline::Auto,
        back: None,
    });
    let blue = cart_face(&tinted);
    let (x, y) = (LYNX_CART_W / 2, LYNX_CART_H * 10 / 100);
    assert!(
        at(&blue, x, y)[2] > at(&blue, x, y)[0] + 30,
        "the lip is not blue"
    );
    let (lx, ly) = (
        LYNX_LABEL_X + LYNX_LABEL_W / 2,
        LYNX_LABEL_Y + LYNX_LABEL_H / 2,
    );
    assert_eq!(
        at(&blue, lx, ly),
        at(&plain, lx, ly),
        "the label took the shell colour"
    );
    let _: Shell = shell_for(&tinted);
}

#[test]
fn the_game_gear_pak_is_a_little_wider_than_tall_with_a_squarish_label() {
    assert_eq!(cart_box(Platform::Gg), (GG_CART_W, GG_CART_H));
    assert!(GG_CART_W * 100 >= GG_CART_H * 102 && GG_CART_W * 100 <= GG_CART_H * 112);
    assert!(GG_CART_H <= GB_CART_H, "no taller than the Game Boy pak");
    // Real labels run about 1.4 to 1 across, not the WonderSwan's wide strip.
    assert!(GG_LABEL_W * 100 >= GG_LABEL_H * 130 && GG_LABEL_W * 100 <= GG_LABEL_H * 150);
}

#[test]
fn the_game_gear_flanges_stand_proud_of_the_body() {
    let f = cart_face(&gg());
    let width = |y: u32| (0..f.w).filter(|&x| at(&f, x, y)[3] > 128).count();
    let hood = width(f.h / 5);
    let body = width(f.h * 2 / 3);
    assert!(body < hood, "hood {hood}px, body {body}px");
    assert!(
        body + 12 > hood,
        "hood {hood}px, body {body}px: only a small step"
    );
    assert!(at(&f, 0, 0)[3] < 64, "the top corner is not rounded");
}

#[test]
fn a_game_gear_cart_is_black_solid_plastic_with_no_contacts_showing() {
    assert_eq!(shell_for(&gg()), GG_SHELL);
    assert_eq!(GG_SHELL.finish, Finish::Solid);
    assert!(!is_clear(&gg()));
    assert!(slot_ui::mould_label_rect(Mould::Gg).2 > 0.8);
    let f = cart_face(&gg());
    let n = (0..f.h)
        .flat_map(|y| (0..f.w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let on_label = (GG_LABEL_X..GG_LABEL_X + GG_LABEL_W).contains(&x)
                && (GG_LABEL_Y..GG_LABEL_Y + GG_LABEL_H).contains(&y);
            !on_label && gold(at(&f, x, y))
        })
        .count();
    assert_eq!(n, 0, "{n} gold pixels on a Game Gear pak");
}

#[test]
fn the_neogeo_pocket_pak_is_a_little_wider_than_tall_with_a_wide_label() {
    assert_eq!(cart_box(Platform::Ngp), (NGP_CART_W, NGP_CART_H));
    assert!(NGP_CART_W * 100 >= NGP_CART_H * 110 && NGP_CART_W * 100 <= NGP_CART_H * 125);
    assert!(NGP_CART_H <= GB_CART_H, "no taller than the Game Boy pak");
    assert!(NGP_LABEL_W * 100 >= NGP_LABEL_H * 170 && NGP_LABEL_W * 100 <= NGP_LABEL_H * 200);
}

#[test]
fn the_neogeo_pocket_cap_stands_proud_of_the_body() {
    let f = cart_face(&ngp());
    let width = |y: u32| (0..f.w).filter(|&x| at(&f, x, y)[3] > 128).count();
    let cap = width(f.h / 20);
    let body = width(f.h / 2);
    assert!(body < cap, "cap {cap}px, body {body}px");
    assert!(
        body + 10 > cap,
        "cap {cap}px, body {body}px: only a small step"
    );
}

#[test]
fn a_neogeo_pocket_cart_is_dark_grey_solid_plastic_with_no_contacts_showing() {
    assert_eq!(shell_for(&ngp()), NGP_SHELL);
    assert_eq!(NGP_SHELL.finish, Finish::Solid);
    assert!(!is_clear(&ngp()));
    let f = cart_face(&ngp());
    let n = (0..f.h)
        .flat_map(|y| (0..f.w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let on_label = (NGP_LABEL_X..NGP_LABEL_X + NGP_LABEL_W).contains(&x)
                && (NGP_LABEL_Y..NGP_LABEL_Y + NGP_LABEL_H).contains(&y);
            !on_label && gold(at(&f, x, y))
        })
        .count();
    assert_eq!(n, 0, "{n} gold pixels on a NeoGeo Pocket pak");
}

#[test]
fn a_wonderswan_game_gets_a_black_cart_and_a_wonderswan_color_game_a_clear_one() {
    let mono = shell_for(&ws_mono());
    assert_eq!(mono.finish, Finish::Solid);
    assert!(
        mono.colour.iter().all(|c| *c < 0x30),
        "not black: {:?}",
        mono.colour
    );
    assert!(!is_clear(&ws_mono()));
    assert!(!shows_interior(&ws_mono()));

    assert_ne!(shell_for(&ws()).finish, Finish::Solid);
    assert!(is_clear(&ws()));

    // The extension decides, whatever its case.
    let shouty = Cart {
        rom: "Games/WS/GUNPEY EX.WSC".into(),
        ..ws()
    };
    assert!(is_clear(&shouty));

    // A shell picked for the game in cart_shell.txt still wins.
    let mut picked = ws_mono();
    picked.shell = Some(slot_store::ShellChoice {
        colour: [0x20, 0x60, 0xd0],
        finish: slot_store::ShellFinish::Clear,
        outline: slot_store::Outline::Auto,
        back: None,
    });
    assert!(is_clear(&picked));

    // The black cart still draws as a whole WonderSwan cart.
    let f = cart_face(&ws_mono());
    assert_eq!((f.w, f.h), (WS_CART_W, WS_CART_H));
}

#[test]
fn the_hucard_is_a_portrait_card_with_a_wide_label_in_its_lower_half() {
    assert_eq!(cart_box(Platform::Tg16), (TG16_CART_W, TG16_CART_H));
    const {
        // A credit card stood on end: about 0.63 wide to 1 tall.
        assert!(TG16_CART_W * 100 >= TG16_CART_H * 60 && TG16_CART_W * 100 <= TG16_CART_H * 67);
        assert!(TG16_CART_H <= GB_CART_H, "no taller than the Game Boy pak");
        assert!(
            TG16_LABEL_Y > TG16_CART_H * 2 / 5,
            "the label is in the lower half"
        );
        assert!(TG16_LABEL_W > TG16_LABEL_H, "the label is wider than tall");
    }
}

#[test]
fn a_hucard_is_white_with_gold_contacts_along_its_top_whatever_the_colour() {
    assert_eq!(shell_for(&tg16()), TG16_SHELL);
    assert_eq!(TG16_SHELL.finish, Finish::Solid);
    assert!(TG16_SHELL.colour.iter().all(|c| *c > 0xc0), "not white");
    assert!(!is_clear(&tg16()));
    let mut red = tg16();
    red.shell = Some(slot_store::ShellChoice {
        colour: [0xc0, 0x20, 0x20],
        finish: slot_store::ShellFinish::Solid,
        outline: slot_store::Outline::Auto,
        back: None,
    });
    for c in [tg16(), red] {
        let f = cart_face(&c);
        let top = TG16_CART_H * 45 / 1000;
        let n = (0..f.w).filter(|&x| gold(at(&f, x, top))).count();
        assert!(n > 60, "only {n} gold pixels across the HuCard contacts");
        let below = (TG16_CART_H / 5..TG16_CART_H)
            .flat_map(|y| (0..f.w).map(move |x| (x, y)))
            .filter(|&(x, y)| gold(at(&f, x, y)))
            .count();
        assert_eq!(below, 0, "gold below the contacts");
        // The panel under the contacts is dark, whatever the shell colour.
        let p = at(&f, TG16_CART_W / 2, TG16_CART_H / 5);
        assert!(p[..3].iter().all(|c| *c < 80), "panel {p:?}");
    }
}

/// A HuCard with a solid red label `w` x `h` pixels across.
fn hucard_with_label(dir: &std::path::Path, w: u32, h: u32) -> Cart {
    let path = dir.join(format!("{w}x{h}.png"));
    let f = std::fs::File::create(&path).expect("create label");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .expect("png header")
        .write_image_data(&[0xd0, 0x10, 0x10].repeat((w * h) as usize))
        .expect("png data");
    Cart {
        label: Some(path),
        ..tg16()
    }
}

fn red(px: [u8; 4]) -> bool {
    px[0] > 0xb0 && px[1] < 0x40 && px[2] < 0x40
}

/// The rows of the label space the art covers, top to bottom.
fn red_rows(f: &slot_ui::CartFace) -> (u32, u32) {
    let x = TG16_LABEL_X + TG16_LABEL_W / 2;
    let rows: Vec<u32> = (TG16_LABEL_Y..TG16_LABEL_Y + TG16_LABEL_H)
        .filter(|&y| red(at(f, x, y)))
        .collect();
    (*rows.first().expect("no label"), *rows.last().unwrap())
}

/// The columns of the label space the art covers, left to right.
fn red_cols(f: &slot_ui::CartFace) -> (u32, u32) {
    let y = TG16_LABEL_Y + TG16_LABEL_H - 2;
    let cols: Vec<u32> = (TG16_LABEL_X..TG16_LABEL_X + TG16_LABEL_W)
        .filter(|&x| red(at(f, x, y)))
        .collect();
    (*cols.first().expect("no label"), *cols.last().unwrap())
}

#[test]
fn a_short_hucard_label_spans_the_width_on_the_foot_with_card_above() {
    let d = tempfile::tempdir().unwrap();
    // Twice as wide as tall: about 95 of the space's 163 rows.
    let c = hucard_with_label(d.path(), 400, 200);
    let f = cart_face(&c);
    let (top, bottom) = red_rows(&f);
    assert_eq!(bottom, TG16_LABEL_Y + TG16_LABEL_H - 1, "not on the foot");
    let tall = bottom + 1 - top;
    assert!(
        (tall as i32 - (TG16_LABEL_W / 2) as i32).abs() <= 1,
        "{tall} rows: lost its shape"
    );
    assert_eq!(
        red_cols(&f),
        (TG16_LABEL_X, TG16_LABEL_X + TG16_LABEL_W - 1),
        "not full width"
    );
    // Above it is the card itself: white, opaque and as pale as the card
    // beside the label space.
    let above = at(&f, TG16_LABEL_X + TG16_LABEL_W / 2, TG16_LABEL_Y + 4);
    let beside = at(&f, TG16_LABEL_X / 2, TG16_LABEL_Y + 4);
    assert_eq!(above[3], 255);
    assert!(
        above[..3].iter().all(|c| *c > 0xc0),
        "above the label {above:?}"
    );
    for c in 0..3 {
        assert!(
            above[c].abs_diff(beside[c]) < 40,
            "{above:?} vs the card {beside:?}"
        );
    }
}

#[test]
fn a_hucard_label_too_tall_for_the_space_is_shrunk_whole_on_the_foot() {
    let d = tempfile::tempdir().unwrap();
    let c = hucard_with_label(d.path(), 300, 400);
    let f = cart_face(&c);
    assert_eq!(
        red_rows(&f),
        (TG16_LABEL_Y, TG16_LABEL_Y + TG16_LABEL_H - 1),
        "not the full height"
    );
    let (left, right) = red_cols(&f);
    let wide = right + 1 - left;
    let want = TG16_LABEL_H * 3 / 4;
    assert!(
        wide.abs_diff(want) <= 1,
        "{wide} columns, not {want}: lost its shape"
    );
    let (gap_l, gap_r) = (left - TG16_LABEL_X, TG16_LABEL_X + TG16_LABEL_W - 1 - right);
    assert!(
        gap_l.abs_diff(gap_r) <= 1,
        "not centred: {gap_l} and {gap_r}"
    );
}

#[test]
fn a_hucard_label_takes_the_shell_colour_where_it_leaves_the_card_showing() {
    let d = tempfile::tempdir().unwrap();
    let mut c = hucard_with_label(d.path(), 400, 200);
    c.shell = Some(slot_store::ShellChoice {
        colour: [0x20, 0x60, 0xd0],
        finish: slot_store::ShellFinish::Solid,
        outline: slot_store::Outline::Auto,
        back: None,
    });
    let f = cart_face(&c);
    let above = at(&f, TG16_LABEL_X + TG16_LABEL_W / 2, TG16_LABEL_Y + 4);
    assert!(
        above[2] > above[0] + 30,
        "the card above the label is not blue: {above:?}"
    );
}

#[test]
fn other_labels_still_fill_their_space() {
    let d = tempfile::tempdir().unwrap();
    let mut c = hucard_with_label(d.path(), 400, 200);
    c.platform = Platform::Ngp;
    let f = cart_face(&c);
    for (x, y) in [
        (NGP_LABEL_X + 2, NGP_LABEL_Y + 2),
        (NGP_LABEL_X + NGP_LABEL_W / 2, NGP_LABEL_Y + NGP_LABEL_H / 2),
    ] {
        assert!(
            red(at(&f, x, y)),
            "the NeoGeo Pocket label left a gap at {x},{y}"
        );
    }
}
