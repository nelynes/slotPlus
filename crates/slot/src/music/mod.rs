//! slotPlus music cart: a cart on the Game Boy Advance shelf that plays the songs in the
//! card's `Music/` folder, in the spirit of the music carts sold for the GBA. Nothing about
//! it loads until it goes in the slot.

pub mod decode;
pub mod library;
pub mod player;
pub mod screen;

pub use library::{has_music, music_dir, MUSIC_DIR};
pub use player::MusicPlayer;
