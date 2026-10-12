use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const FOOTER: u64 = 10;
const MIN_ROM: u64 = 64 * 1024;

/// WonderSwan carts end in a 10-byte footer. Bit 0 of its seventh byte is the
/// orientation the game was made for: 1 means vertical.
pub fn ws_vertical(rom: &Path) -> bool {
    let Ok(mut f) = std::fs::File::open(rom) else {
        return false;
    };
    let Ok(len) = f.seek(SeekFrom::End(0)) else {
        return false;
    };
    if len < MIN_ROM || f.seek(SeekFrom::Start(len - FOOTER)).is_err() {
        return false;
    }
    let mut footer = [0u8; FOOTER as usize];
    f.read_exact(&mut footer).is_ok() && footer[6] & 1 != 0
}
