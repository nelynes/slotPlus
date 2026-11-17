use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use slot_store::{Cart, Platform};
use slot_ui::shelf_life::{play_sticker, Look};
use slot_ui::{
    board_face, cart_face_with, cart_interior, hucard_back_face, lynx_back_face, padded, CartFace,
    TURN_PAD,
};

use crate::label_cache::label_art;

pub struct BuiltFaces {
    pub platform: Platform,
    pub stem: String,
    pub interior: CartFace,
    pub board: CartFace,
    pub lid: CartFace,
    /// slotPlus: the game's achievement set is on the card, so it can be printed.
    pub has_set: bool,
}

/// A cart to build the picker's faces for: how it has been kept, and how long it has been
/// played, so its lid matches the shelf and its back carries the time.
#[derive(Clone, Debug)]
pub struct FaceAsk {
    pub cart: Cart,
    pub look: Look,
    pub played: Option<u64>,
}

impl From<Cart> for FaceAsk {
    fn from(cart: Cart) -> Self {
        FaceAsk {
            cart,
            look: Look::default(),
            played: None,
        }
    }
}

pub struct FaceBuilder {
    requests: Sender<FaceAsk>,
    built: Receiver<BuiltFaces>,
}

impl FaceBuilder {
    pub fn spawn(root: PathBuf) -> Self {
        let (requests, inbox) = mpsc::channel::<FaceAsk>();
        let (outbox, built) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("slot-faces".into())
            .spawn(move || {
                while let Ok(mut ask) = inbox.recv() {
                    while let Ok(newer) = inbox.try_recv() {
                        ask = newer;
                    }
                    let cart = &ask.cart;
                    let mut board = match cart.platform {
                        Platform::Lynx => lynx_back_face(cart),
                        // slotPlus: the back of a HuCard, with Slot's Nickname on its
                        // NAME line.
                        Platform::Tg16 => {
                            hucard_back_face(cart, &slot_store::read_slot_state(&root).nickname)
                        }
                        _ => board_face(cart),
                    };
                    if let Some(secs) = ask.played {
                        play_sticker(&mut board, cart, &slot_store::play_time_text(secs));
                    }
                    let progress = crate::ACHIEVEMENTS
                        .then(|| slot_achievements::progress(&root, &cart.rom))
                        .flatten();
                    let has_set = progress.is_some();
                    if let Some(p) = progress {
                        let words = match p.mastered() {
                            true => "Mastered".to_string(),
                            false => p.text(),
                        };
                        slot_ui::shelf_life::achievement_sticker(&mut board, cart, &words);
                    }
                    let lid = crate::cart_faces::looked(
                        cart_face_with(cart, label_art(&root, cart)),
                        cart,
                        ask.look,
                        &root,
                    );
                    let faces = BuiltFaces {
                        platform: cart.platform,
                        stem: cart.stem.clone(),
                        interior: cart_interior(cart),
                        // A Lynx cart has no board to show: the hold picker
                        // turns it over to its back instead.
                        board,
                        lid: padded(&lid, TURN_PAD),
                        has_set,
                    };
                    if outbox.send(faces).is_err() {
                        return;
                    }
                }
            });
        if let Err(e) = spawned {
            eprintln!("slot: faces: worker thread failed to start: {e}");
        }
        FaceBuilder { requests, built }
    }

    pub fn request(&self, ask: impl Into<FaceAsk>) {
        let _ = self.requests.send(ask.into());
    }

    pub fn take(&self) -> Option<BuiltFaces> {
        let mut newest = None;
        while let Ok(faces) = self.built.try_recv() {
            newest = Some(faces);
        }
        newest
    }
}
