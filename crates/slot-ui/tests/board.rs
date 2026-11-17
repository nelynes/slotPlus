use slot_store::{Cart, Core, Platform};
use slot_ui::{
    board_at, board_face, board_in, chip_face, chip_shadow_face, lid_at, lift_of, on_board, padded,
    rom_marking, rom_marking_face, shelf_cart, slide_of, socket_face, Canvas, CartFace, Placed,
    BOARD_H, BOARD_W, CART_H, CART_W, CHIP_H, CHIP_W, LID_TURN, ROM_H, ROM_W, ROM_X, ROM_Y,
    SHADOW_H, SHADOW_W, SLIDE_SHARE, SLIDE_UP, SOCKET_H, SOCKET_W, TURN_PAD,
};

fn cart(stem: &str, code: &str) -> Cart {
    Cart {
        platform: Platform::Gba,
        stem: stem.into(),
        rom: format!("Games/GBA/{stem}.gba").into(),
        label: None,
        code: code.into(),
        shell: None,
        title: stem.to_uppercase(),
        tags: Vec::new(),
    }
}

fn rgb(face: &CartFace, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * face.w + x) * 4) as usize;
    [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]]
}

const EMERALD: &str = "Pokemon - Emerald Version (USA, Europe)";

#[test]
fn the_marking_is_the_title_stacked() {
    assert_eq!(rom_marking(EMERALD), ["POKEMON", "EMERALD", "VERSION"]);
}

#[test]
fn a_long_title_folds_its_tail_into_the_third_line() {
    assert_eq!(
        rom_marking("Advance Wars 2 - Black Hole Rising"),
        ["ADVANCE", "WARS 2", "BLACK HOLE RISING"]
    );
}

#[test]
fn bracketed_tags_never_reach_the_chip() {
    assert_eq!(
        rom_marking("Pokemon - LeafGreen Version (USA, Europe) (Rev 1)"),
        ["POKEMON", "LEAFGREEN", "VERSION"]
    );
    assert_eq!(rom_marking("Metroid Fusion"), ["METROID", "FUSION"]);
}

#[test]
fn the_marking_is_drawn_and_stays_on_the_chip() {
    for stem in [
        EMERALD,
        "Metroid Fusion",
        "Advance Wars 2 - Black Hole Rising",
        "Pokemon - LeafGreen Version (USA, Europe) (Rev 1)",
    ] {
        let face = rom_marking_face(stem);
        assert_eq!((face.w, face.h), (ROM_W, ROM_H));
        let alpha = |x: u32, y: u32| face.rgba[((y * face.w + x) * 4 + 3) as usize];
        assert!(
            (0..face.h).any(|y| (0..face.w).any(|x| alpha(x, y) > 128)),
            "{stem} left no marking"
        );
        for y in 0..face.h {
            assert_eq!(alpha(0, y), 0, "{stem} ran off the left of the chip");
            assert_eq!(
                alpha(face.w - 1, y),
                0,
                "{stem} ran off the right of the chip"
            );
        }
    }
}

#[test]
fn the_board_bitmap_is_shared_outside_the_rom_window() {
    let emerald = board_face(&cart(EMERALD, "BPEE"));
    let other = board_face(&cart("Homebrew", ""));
    for y in 0..emerald.h {
        for x in 0..emerald.w {
            let in_window =
                (ROM_X..ROM_X + ROM_W).contains(&x) && (ROM_Y..ROM_Y + ROM_H).contains(&y);
            if in_window {
                continue;
            }
            assert_eq!(
                rgb(&emerald, x, y),
                rgb(&other, x, y),
                "the shell bitmap changed at {x},{y}"
            );
        }
    }
    assert_ne!(emerald.rgba, other.rgba, "the rom title was not printed");
}

#[test]
fn the_board_is_the_size_it_is_shown_at() {
    let face = board_face(&cart(EMERALD, "BPEE"));
    assert_eq!((face.w, face.h), (BOARD_W, BOARD_H));
    assert_eq!((BOARD_W, BOARD_H), (CART_W, CART_H));
}

#[test]
fn the_board_itself_is_drawn() {
    let face = board_face(&cart(EMERALD, "BPEE"));
    assert!(
        alpha(&face, BOARD_W / 2, BOARD_H / 2) > 200,
        "the circuit-board bitmap is empty"
    );
}

fn alpha(face: &CartFace, x: u32, y: u32) -> u8 {
    face.rgba[((y * face.w + x) * 4 + 3) as usize]
}

#[test]
fn the_open_cart_starts_as_the_shelf_cart_and_lands_where_the_mockup_has_it() {
    assert_eq!(
        shelf_cart(Canvas::FRAMED),
        Placed {
            x: 180.0,
            y: 134.0,
            w: 360.0,
            h: 212.0
        }
    );
    assert_eq!(
        board_at(Canvas::FRAMED, 0.0),
        board_in(shelf_cart(Canvas::FRAMED))
    );
    assert_eq!(
        board_at(Canvas::FRAMED, 0.0).x,
        board_at(Canvas::FRAMED, 1.0).x,
        "the board slid sideways while it opened"
    );
    assert_eq!(
        board_at(Canvas::FRAMED, 1.0),
        Placed {
            x: 180.0,
            y: 150.0,
            w: CART_W as f32,
            h: CART_H as f32,
        }
    );
    assert_eq!(
        board_at(Canvas::FRAMED, 1.7),
        board_at(Canvas::FRAMED, 1.0),
        "an overshoot kept growing"
    );
}

#[test]
fn the_lid_leaves_level_and_rests_turned() {
    assert_eq!(
        lid_at(Canvas::FRAMED, 0.0),
        (shelf_cart(Canvas::FRAMED), 0.0)
    );
    let (rest, turn) = lid_at(Canvas::FRAMED, 1.0);
    assert_eq!((rest.x, rest.w), (288.0, 144.0));
    assert!(
        (rest.h - 144.0 * CART_H as f32 / CART_W as f32).abs() < 1e-3,
        "the lid is squashed on landing: {rest:?}"
    );
    assert!(
        (rest.y + rest.h / 2.0 - 70.5).abs() < 1e-3,
        "the lid moved: {rest:?}"
    );
    assert_eq!(turn, LID_TURN);
    assert!(
        (LID_TURN.to_degrees() + 5.0).abs() < 1e-4,
        "{}",
        LID_TURN.to_degrees()
    );
}

#[test]
fn the_beats_split_one_progress() {
    assert_eq!((slide_of(0.0), lift_of(0.0)), (0.0, 0.0));
    assert_eq!((slide_of(SLIDE_SHARE), lift_of(SLIDE_SHARE)), (1.0, 0.0));
    assert_eq!((slide_of(1.0), lift_of(1.0)), (1.0, 1.0));
    assert!((slide_of(SLIDE_SHARE / 2.0) - 0.5).abs() < 1e-5);
}

#[test]
fn the_back_half_stays_on_the_shelf_while_the_front_slides() {
    for p in [0.0, 0.1, 0.25, SLIDE_SHARE] {
        assert_eq!(
            board_at(Canvas::FRAMED, p),
            board_in(shelf_cart(Canvas::FRAMED)),
            "the back moved at progress {p}"
        );
    }
}

#[test]
fn the_lid_keeps_its_proportions_all_the_way_open() {
    let lid_ratio = CART_W as f32 / CART_H as f32;
    for i in 0..=20 {
        let p = i as f32 / 20.0;
        let (l, _) = lid_at(Canvas::FRAMED, p);
        assert!(
            (l.w / l.h - lid_ratio).abs() < 0.01,
            "the lid is stretched at progress {p}: {l:?}"
        );
    }
}

#[test]
fn the_board_fills_the_cart_so_opening_does_not_squash_it() {
    let cart = shelf_cart(Canvas::FRAMED);
    let board = board_in(cart);
    assert_eq!(
        board, cart,
        "the back half is shorter than the lid: {board:?}"
    );
    assert_eq!(board_at(Canvas::FRAMED, 0.0), cart);
    assert!(
        board_at(Canvas::FRAMED, 1.0).h <= cart.h,
        "the resting board grew taller than the cart"
    );
}

#[test]
fn the_front_slides_up_a_third_before_it_lifts() {
    assert_eq!(SLIDE_UP, CART_H as f32 / 3.0);
    let shelf = shelf_cart(Canvas::FRAMED);
    let (slid, turn) = lid_at(Canvas::FRAMED, SLIDE_SHARE);
    assert_eq!(
        slid,
        Placed {
            y: shelf.y - SLIDE_UP,
            ..shelf
        }
    );
    assert_eq!(turn, 0.0);
    let (half, turn) = lid_at(Canvas::FRAMED, SLIDE_SHARE / 2.0);
    assert!(half.y < shelf.y && half.y > slid.y && turn == 0.0 && half.w == shelf.w);
}

#[test]
fn the_lift_starts_where_the_slide_ends() {
    let (slid, _) = lid_at(Canvas::FRAMED, SLIDE_SHARE);
    let (next, _) = lid_at(Canvas::FRAMED, SLIDE_SHARE + 0.001);
    assert!((next.y - slid.y).abs() < 0.5 && (next.w - slid.w).abs() < 0.5);
}

#[test]
fn board_units_land_on_the_panel_at_one_and_a_half_times() {
    let board = board_at(Canvas::FRAMED, 1.0);
    assert_eq!(on_board(board, 0.0, 0.0), (180.0, 150.0));
    assert_eq!(on_board(board, 240.0, 135.0), (540.0, 362.0));
}

#[test]
fn a_socket_names_its_core_and_stays_open_in_the_middle() {
    for core in Core::ALL {
        let face = socket_face(core);
        assert_eq!((face.w, face.h), (SOCKET_W, SOCKET_H));
    }
    assert_ne!(
        socket_face(Core::Mgba).rgba,
        socket_face(Core::Gpsp).rgba,
        "both sockets wear the same name"
    );
    let open = socket_face(Core::Mgba)
        .rgba
        .chunks_exact(4)
        .any(|p| p[3] == 0);
    assert!(open, "the socket bitmap has no opening");
}

#[test]
fn the_chip_is_padded_clear_on_every_side() {
    for core in [None, Some(Core::Mgba), Some(Core::Gpsp)] {
        let face = chip_face(core);
        assert_eq!(
            (face.w, face.h),
            (CHIP_W + 2 * TURN_PAD, CHIP_H + 2 * TURN_PAD)
        );
        for y in 0..face.h {
            for x in 0..face.w {
                let ring = x < TURN_PAD
                    || y < TURN_PAD
                    || x >= face.w - TURN_PAD
                    || y >= face.h - TURN_PAD;
                if ring {
                    assert_eq!(alpha(&face, x, y), 0, "{core:?} is not clear at {x},{y}");
                }
            }
        }
    }
}

#[test]
fn a_seated_chip_wears_its_name_and_a_flying_one_is_blank() {
    let differs = |a: &CartFace, b: &CartFace| {
        a.rgba
            .chunks_exact(4)
            .zip(b.rgba.chunks_exact(4))
            .filter(|(p, q)| p[..3] != q[..3])
            .count()
    };
    let blank = chip_face(None);
    assert!(
        differs(&chip_face(Some(Core::Mgba)), &blank) > 20,
        "mGBA left no name"
    );
    assert_ne!(
        chip_face(Some(Core::Mgba)).rgba,
        chip_face(Some(Core::Gpsp)).rgba,
        "both cores wear the same mark"
    );
}

#[test]
fn padding_moves_nothing() {
    let face = chip_face(Some(Core::Gpsp));
    let more = padded(&face, 3);
    assert_eq!((more.w, more.h), (face.w + 6, face.h + 6));
    for (x, y) in [(0, 0), (10, 7), (face.w - 1, face.h - 1)] {
        let a = ((y * face.w + x) * 4) as usize;
        let b = (((y + 3) * more.w + x + 3) * 4) as usize;
        assert_eq!(face.rgba[a..a + 4], more.rgba[b..b + 4]);
    }
}

#[test]
fn the_shadow_is_darkest_in_the_middle_and_gone_at_the_corners() {
    let s = chip_shadow_face();
    assert_eq!((s.w, s.h), (SHADOW_W, SHADOW_H));
    assert!(alpha(&s, SHADOW_W / 2, SHADOW_H / 2) > alpha(&s, SHADOW_W / 8, SHADOW_H / 2));
    assert_eq!(alpha(&s, 0, 0), 0);
}

#[test]
fn the_seated_chips_name_is_antialiased_against_the_body() {
    let red = |face: &CartFace, x: u32, y: u32| face.rgba[((y * face.w + x) * 4) as usize];
    for core in [Core::Mgba, Core::Gpsp] {
        let named = chip_face(Some(core));
        let blank = chip_face(None);
        let soft = (0..named.h)
            .flat_map(|y| (0..named.w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                alpha(&named, x, y) == 255
                    && red(&named, x, y) != red(&blank, x, y)
                    && (0x40..0xc0).contains(&red(&named, x, y))
            })
            .count();
        assert!(
            soft > 5,
            "{core:?}'s name has only {soft} antialiased pixels"
        );
    }
}

#[test]
fn a_versioned_chip_and_socket_keep_their_lowercase_v() {
    let chip = slot_ui::chip_face_text("v1.12.0");
    assert_eq!(
        (chip.w, chip.h),
        (CHIP_W + 2 * TURN_PAD, CHIP_H + 2 * TURN_PAD)
    );
    assert_ne!(
        chip.rgba,
        slot_ui::chip_face_text("V1.12.0").rgba,
        "the chip's v was capitalised"
    );
    assert_ne!(
        chip.rgba,
        chip_face(None).rgba,
        "the chip carries no version"
    );
    let socket = slot_ui::socket_face_text("v1.10.0");
    assert_eq!((socket.w, socket.h), (SOCKET_W, SOCKET_H));
    assert_ne!(
        socket.rgba,
        slot_ui::socket_face_text("V1.10.0").rgba,
        "the socket's v was capitalised"
    );
}

#[test]
fn the_bare_board_is_the_board_art_unmarked() {
    let bare = slot_ui::bare_board_face();
    assert_eq!((bare.w, bare.h), (BOARD_W, BOARD_H));
    let i = (((ROM_Y + ROM_H / 2) * BOARD_W + ROM_X + ROM_W / 2) * 4) as usize;
    assert!(bare.rgba[i] < 60, "the ROM chip is not its own dark art");
}

#[test]
fn the_update_cart_is_clear_purple_with_a_charcoal_label() {
    let cart = slot_ui::update_cart_face();
    assert_eq!((cart.w, cart.h), (360, 212));
    let [r, g, b] = rgb(&cart, 20, 100);
    assert!(b > r && r > g, "the shell is not purple: {r},{g},{b}");
    let [r, g, b] = rgb(&cart, 75, 70);
    assert!(
        r.abs_diff(g) < 12 && b.abs_diff(g) < 12 && g < 90,
        "the label is not charcoal: {r},{g},{b}"
    );
}

#[test]
fn the_baseos_cart_is_black_with_a_white_wordmark() {
    let cart = slot_ui::baseos_cart_face();
    assert_eq!((cart.w, cart.h), (360, 212));
    let [r, g, b] = rgb(&cart, 20, 100);
    assert!(
        r < 60 && g < 60 && b < 60,
        "the shell is not black: {r},{g},{b}"
    );
    let white = (60..300)
        .flat_map(|x| (60..190).map(move |y| (x, y)))
        .filter(|&(x, y)| rgb(&cart, x, y).iter().all(|&c| c > 220))
        .count();
    assert!(white > 100, "no white wordmark: {white}");
}

#[test]
fn a_game_boy_cart_shows_its_own_board() {
    let gb = Cart {
        platform: Platform::Gb,
        rom: "Games/GB/Tetris.gb".into(),
        ..cart("Tetris", "")
    };
    let face = board_face(&gb);
    let backing = slot_ui::cart_backing(slot_ui::gb_shell_of(&gb));
    assert_eq!((face.w, face.h), (slot_ui::GB_CART_W, slot_ui::GB_CART_H));
    let at = |f: &slot_ui::CartFace, x: u32, y: u32| {
        let i = ((y * f.w + x) * 4) as usize;
        [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2]]
    };
    let navy = |p: [u8; 3]| p[2] > p[1] && p[0] < 45;
    for (x, y) in [(100, 215), (218, 210)] {
        assert!(
            navy(at(&backing, x, y)),
            "the pak board lost its chip at {x},{y}"
        );
        assert!(
            !navy(at(&face, x, y)),
            "a printed chip still fills the socket at {x},{y}"
        );
    }
    assert_eq!(
        at(&face, 140, 60),
        at(&backing, 140, 60),
        "the pak opened on another board"
    );
}

#[test]
fn a_gba_cart_keeps_the_gba_picker_geometry() {
    let canvas = Canvas { w: 720.0, h: 480.0 };
    let shelf = shelf_cart(canvas);
    for platform in [Platform::Gba, Platform::Ws] {
        let size = slot_ui::cart_box(platform);
        for p in [0.0, 0.3, 0.7, 1.0] {
            assert_eq!(
                slot_ui::board_from_shaped(canvas, None, size, shelf, p),
                slot_ui::board_from(canvas, shelf, p)
            );
            assert_eq!(
                slot_ui::lid_from_shaped(canvas, None, size, shelf, p),
                slot_ui::lid_from(canvas, shelf, p)
            );
        }
    }
    assert_eq!(slot_ui::seats(None), slot_ui::GBA_SEATS);
}

/// slotPlus: a squarer cart opens at its own shape, from where it sits on the shelf to
/// where its board rests, and its lid flies off at its own shape too.
#[test]
fn a_squarer_cart_opens_at_its_own_shape() {
    let canvas = Canvas { w: 720.0, h: 480.0 };
    for platform in [Platform::Pm, Platform::Gg, Platform::Ngp, Platform::Nds] {
        let size = slot_ui::cart_box(platform);
        let aspect = size.0 as f32 / size.1 as f32;
        let shelf = slot_ui::shelf_cart_shaped(canvas, None, size, 200.0, 0.8);
        for p in [0.0, 0.5, 1.0] {
            let board = slot_ui::board_from_shaped(canvas, None, size, shelf, p);
            let (lid, _) = slot_ui::lid_from_shaped(canvas, None, size, shelf, p);
            for (what, r) in [("board", board), ("lid", lid)] {
                assert!(
                    (r.w / r.h - aspect).abs() < 0.02,
                    "{platform:?} {what} at {p}: {} x {}",
                    r.w,
                    r.h
                );
            }
            assert!(
                board.y + board.h <= canvas.h - 40.0,
                "{platform:?} clear of the legends"
            );
        }
    }
}

fn hucard() -> slot_store::Cart {
    slot_store::Cart {
        platform: slot_store::Platform::Tg16,
        stem: "Bonk's Adventure (USA)".into(),
        rom: "/c/Games/TG16/Bonk's Adventure (USA).pce".into(),
        label: None,
        title: "Bonk's Adventure".into(),
        code: String::new(),
        shell: None,
        tags: vec![],
    }
}

/// How many pixels differ between two faces inside a box.
fn changed(a: &CartFace, b: &CartFace, (x0, y0, x1, y1): (u32, u32, u32, u32)) -> usize {
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let at = ((y * a.w + x) * 4) as usize;
            if a.rgba[at..at + 3] != b.rgba[at..at + 3] {
                n += 1;
            }
        }
    }
    n
}

/// The NAME line printed on a HuCard's back (assets/src/hucard_back.py).
const NAME_LINE_Y: u32 = 119;

#[test]
fn a_hucards_back_is_printed_and_has_the_nickname_on_its_name_line() {
    let cart = hucard();
    let plain = slot_ui::cart_back_shell(&cart);
    let blank = slot_ui::hucard_back_face(&cart, "");
    let named = slot_ui::hucard_back_face(&cart, "Nelson");
    assert_eq!((blank.w, blank.h), (plain.w, plain.h));
    assert!(
        changed(&plain, &blank, (15, 16, 199, 44)) > 3000,
        "the band is printed"
    );
    let line = (58, NAME_LINE_Y - 19, 194, NAME_LINE_Y);
    assert!(
        changed(&blank, &named, line) > 50,
        "the name is written on the line"
    );
    let elsewhere = (0, NAME_LINE_Y + 4, blank.w, blank.h);
    assert_eq!(changed(&blank, &named, elsewhere), 0, "and nowhere else");
    assert_eq!(
        slot_ui::hucard_back_face(&cart, "   ").rgba,
        blank.rgba,
        "a name of spaces leaves the line blank"
    );
}

#[test]
fn the_stickers_on_a_hucards_back_sit_beside_the_notes_clear_of_the_name() {
    use slot_ui::shelf_life::{gold_sticker_at, sticker_at, STICKER_H, STICKER_W};
    let cart = hucard();
    let back = slot_ui::hucard_back_face(&cart, "Nelson");
    let play = sticker_at(&cart, &back).expect("a play-time spot");
    let gold = gold_sticker_at(&cart, &back).expect("a gold spot");
    for (x, y) in [play, gold] {
        assert!(y > NAME_LINE_Y + 4, "below the NAME line");
        assert!(
            x >= 120 && x + STICKER_W <= back.w - 8,
            "to the right of the Notes"
        );
    }
    assert!(
        gold.1 >= play.1 + STICKER_H,
        "one above the other, not overlapping"
    );
}

#[test]
fn a_pokemon_mini_cart_shows_its_own_board_with_the_game_on_its_chip() {
    let pm = Cart {
        platform: Platform::Pm,
        rom: "Games/PokeMini/Pokemon Puzzle Collection.min".into(),
        ..cart("Pokemon Puzzle Collection", "")
    };
    let other = Cart {
        stem: "Pokemon Party mini".into(),
        ..pm.clone()
    };
    let face = board_face(&pm);
    assert_eq!((face.w, face.h), (300, 262), "the board fills the cart");
    let at = |f: &CartFace, x: u32, y: u32| {
        let i = ((y * f.w + x) * 4) as usize;
        [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]]
    };
    let green = at(&face, 60, 150);
    assert!(
        green[1] > green[0] + 40 && green[3] == 255,
        "green board: {green:?}"
    );
    assert_eq!(at(&face, 5, 130)[3], 0, "nothing outside the board");
    let chip = (
        slot_ui::PM_ROM_X,
        slot_ui::PM_ROM_Y,
        slot_ui::PM_ROM_W,
        slot_ui::PM_ROM_H,
    );
    let other = board_face(&other);
    let differs = (chip.0..chip.0 + chip.2)
        .flat_map(|x| (chip.1..chip.1 + chip.3).map(move |y| (x, y)))
        .filter(|&(x, y)| at(&face, x, y) != at(&other, x, y))
        .count();
    assert!(differs > 30, "each game's name is on its chip: {differs}");
    let (sw, sh) = (
        slot_ui::shelf_life::STICKER_W,
        slot_ui::shelf_life::STICKER_H,
    );
    let play = slot_ui::shelf_life::sticker_at(&pm, &face).expect("a play-time spot");
    let gold = slot_ui::shelf_life::gold_sticker_at(&pm, &face).expect("a gold spot");
    assert!(play.0 + sw <= gold.0, "side by side");
    for (x, y) in [play, gold] {
        assert!(
            y >= chip.1 + chip.3 && y + sh < 160,
            "below the chip, above the contacts"
        );
        assert!(x >= 30 && x + sw <= 270, "on the board");
    }
}

#[test]
fn a_game_gear_cart_shows_its_own_board_with_the_game_on_its_chip() {
    let pm = Cart {
        platform: Platform::Gg,
        rom: "Games/GG/Sonic & Tails.gg".into(),
        ..cart("Sonic & Tails", "")
    };
    let other = Cart {
        stem: "Shinobi".into(),
        ..pm.clone()
    };
    let face = board_face(&pm);
    assert_eq!((face.w, face.h), (352, 329), "the board fills the cart");
    let at = |f: &CartFace, x: u32, y: u32| {
        let i = ((y * f.w + x) * 4) as usize;
        [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]]
    };
    let green = at(&face, 30, 250);
    assert!(
        green[1] > green[0] + 40 && green[3] == 255,
        "green board: {green:?}"
    );
    assert_eq!(at(&face, 4, 20)[3], 0, "nothing outside the board");
    let chip = (
        slot_ui::GG_ROM_X,
        slot_ui::GG_ROM_Y,
        slot_ui::GG_ROM_W,
        slot_ui::GG_ROM_H,
    );
    let other = board_face(&other);
    let differs = (chip.0..chip.0 + chip.2)
        .flat_map(|x| (chip.1..chip.1 + chip.3).map(move |y| (x, y)))
        .filter(|&(x, y)| at(&face, x, y) != at(&other, x, y))
        .count();
    assert!(differs > 30, "each game's name is on its chip: {differs}");
    let (sw, sh) = (
        slot_ui::shelf_life::STICKER_W,
        slot_ui::shelf_life::STICKER_H,
    );
    let play = slot_ui::shelf_life::sticker_at(&pm, &face).expect("a play-time spot");
    let gold = slot_ui::shelf_life::gold_sticker_at(&pm, &face).expect("a gold spot");
    assert!(play.0 + sw <= gold.0, "side by side");
    for (x, y) in [play, gold] {
        assert!(
            y >= chip.1 + chip.3 && y + sh < 286,
            "below the chip, above the contacts"
        );
        assert!(x >= 12 && x + sw <= 340, "on the board");
    }
}

#[test]
fn a_ds_card_shows_its_own_board_with_the_game_on_its_chip() {
    let pm = Cart {
        platform: Platform::Nds,
        rom: "Games/NDS/Mario Kart DS.nds".into(),
        ..cart("Mario Kart DS", "")
    };
    let other = Cart {
        stem: "Animal Crossing - Wild World".into(),
        ..pm.clone()
    };
    let face = board_face(&pm);
    assert_eq!((face.w, face.h), (288, 304), "the board fills the cart");
    let at = |f: &CartFace, x: u32, y: u32| {
        let i = ((y * f.w + x) * 4) as usize;
        [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]]
    };
    let green = at(&face, 30, 120);
    assert!(
        green[1] > green[0] + 40 && green[3] == 255,
        "green board: {green:?}"
    );
    assert_eq!(at(&face, 4, 20)[3], 0, "nothing outside the board");
    let chip = (
        slot_ui::NDS_ROM_X,
        slot_ui::NDS_ROM_Y,
        slot_ui::NDS_ROM_W,
        slot_ui::NDS_ROM_H,
    );
    let other = board_face(&other);
    let differs = (chip.0..chip.0 + chip.2)
        .flat_map(|x| (chip.1..chip.1 + chip.3).map(move |y| (x, y)))
        .filter(|&(x, y)| at(&face, x, y) != at(&other, x, y))
        .count();
    assert!(differs > 30, "each game's name is on its chip: {differs}");
    let (sw, sh) = (
        slot_ui::shelf_life::STICKER_W,
        slot_ui::shelf_life::STICKER_H,
    );
    let play = slot_ui::shelf_life::sticker_at(&pm, &face).expect("a play-time spot");
    let gold = slot_ui::shelf_life::gold_sticker_at(&pm, &face).expect("a gold spot");
    assert!(play.0 + sw <= gold.0, "side by side");
    for (x, y) in [play, gold] {
        assert!(
            y >= chip.1 + chip.3 && y + sh <= 202,
            "below the chip, above the contacts"
        );
        assert!(x >= 20 && x + sw <= 204, "on the board");
    }
}

#[test]
fn a_neogeo_pocket_cart_shows_its_own_board_under_its_foot() {
    let pm = Cart {
        platform: Platform::Ngp,
        rom: "Games/NGP/Metal Slug 1st Mission.ngc".into(),
        ..cart("Metal Slug 1st Mission", "")
    };
    let other = Cart {
        stem: "SNK vs. Capcom".into(),
        ..pm.clone()
    };
    let face = board_face(&pm);
    assert_eq!((face.w, face.h), (312, 268), "the board fills the cart");
    let at = |f: &CartFace, x: u32, y: u32| {
        let i = ((y * f.w + x) * 4) as usize;
        [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]]
    };
    let green = at(&face, 60, 60);
    assert!(
        green[1] > green[0] + 40 && green[3] == 255,
        "green board: {green:?}"
    );
    assert_eq!(at(&face, 4, 20)[3], 0, "nothing outside the board");
    let chip = (
        slot_ui::NGP_ROM_X,
        slot_ui::NGP_ROM_Y,
        slot_ui::NGP_ROM_W,
        slot_ui::NGP_ROM_H,
    );
    let other = board_face(&other);
    let differs = (chip.0..chip.0 + chip.2)
        .flat_map(|x| (chip.1..chip.1 + chip.3).map(move |y| (x, y)))
        .filter(|&(x, y)| at(&face, x, y) != at(&other, x, y))
        .count();
    assert!(differs > 30, "each game's name is on its chip: {differs}");
    // The foot stays on, in the cart's own face, hiding the contacts.
    let front = slot_ui::cart_face_with(&pm, None);
    for (x, y) in [(60, 240), (156, 230), (250, 255)] {
        assert_eq!(at(&face, x, y), at(&front, x, y), "the foot at {x},{y}");
    }
    let (sw, sh) = (
        slot_ui::shelf_life::STICKER_W,
        slot_ui::shelf_life::STICKER_H,
    );
    let play = slot_ui::shelf_life::sticker_at(&pm, &face).expect("a play-time spot");
    let gold = slot_ui::shelf_life::gold_sticker_at(&pm, &face).expect("a gold spot");
    assert!(play.1 + sh <= gold.1, "one above the other");
    for (x, y) in [play, gold] {
        assert!(y > 43 && y + sh < 211, "on the board, above the foot");
        assert!(x >= 10 && x + sw < slot_ui::NGP_ROM_X, "left of the chip");
    }
}
