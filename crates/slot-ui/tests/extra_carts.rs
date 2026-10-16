//! The carts slotPlus adds to Slot's three: the WonderSwan, the Atari Lynx, the
//! Game Gear and the NeoGeo Pocket, drawn from their own art in the same style as the GBA and
//! Game Boy carts.

use slot_store::{Cart, Platform};
use slot_ui::{
    cart_box, cart_face, cart_interior, is_clear, label_size, mould_backing, mould_for, mould_of,
    mould_shadow, seated_box, shell_for, shows_interior, Finish, Mould, Shell, GB_CART_H,
    GG_CART_H, GG_CART_W, GG_LABEL_H, GG_LABEL_W, GG_LABEL_X, GG_LABEL_Y, GG_SHELL, LYNX_CART_H,
    LYNX_CART_W, LYNX_LABEL_H, LYNX_LABEL_W, LYNX_LABEL_X, LYNX_LABEL_Y, NGP_CART_H, NGP_CART_W,
    NGP_LABEL_H, NGP_LABEL_W, NGP_LABEL_X, NGP_LABEL_Y, NGP_SHELL, WS_CART_H, WS_CART_W,
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

fn ws() -> Cart {
    cart(Platform::Ws, "Gunpey")
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
    assert_eq!(mould_for(Platform::Ws), Mould::Ws);
    assert_eq!(mould_for(Platform::Lynx), Mould::Lynx);
    assert_eq!(mould_for(Platform::Gba), Mould::Gba);
    assert_eq!(
        Mould::EXTRA,
        [Mould::Ws, Mould::Lynx, Mould::Gg, Mould::Ngp]
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
    for c in [ws(), lynx(), gg(), ngp()] {
        let f = cart_face(&c);
        assert_eq!((f.w, f.h), cart_box(c.platform), "{:?}", c.platform);
        let (sw, sh) = seated_box(c.platform);
        assert_eq!(sw, 240);
        let (cw, ch) = cart_box(c.platform);
        assert!((sh as f32 - ch as f32 * 240.0 / cw as f32).abs() <= 1.0);
    }
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
