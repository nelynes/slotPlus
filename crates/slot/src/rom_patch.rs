//! slotPlus: romhacks and translations. A patch named like the game and kept beside it
//! (`Games/GBA/Golden Sun.ips`, `.bps` or `.ups`) is applied as the game loads, on the
//! emulator's thread. The ROM on the card is never changed: the patched game goes to a
//! scratch file the core loads from, so cores that read the file themselves (gpSP) get it
//! too. Labels, saves and the shelf still go by the original file.
//!
//! RetroAchievements hashes what the core loaded, so a patched game only finds a set made
//! for that hack, and never the original game's set.

use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum PatchError {
    /// Not a patch file of the kind its name says.
    Format,
    /// The patch was made for a different ROM.
    WrongRom,
    /// The patched game didn't come out as the patch says it should.
    Result,
    /// The patch itself is damaged.
    Damaged,
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            PatchError::Format => "not a patch",
            PatchError::WrongRom => "made for a different ROM",
            PatchError::Result => "the patched game didn't check out",
            PatchError::Damaged => "the patch is damaged",
        })
    }
}

const KINDS: [&str; 3] = ["ips", "bps", "ups"];

/// The patch beside a game, if there is one.
pub fn patch_for(rom: &Path) -> Option<PathBuf> {
    let stem = rom.file_stem()?;
    let dir = rom.parent()?;
    KINDS.iter().find_map(|ext| {
        [ext.to_string(), ext.to_uppercase()]
            .into_iter()
            .find_map(|e| {
                let mut name = stem.to_os_string();
                name.push(".");
                name.push(e);
                let path = dir.join(name);
                path.is_file().then_some(path)
            })
    })
}

/// Apply `patch` (its kind from its file name) to `rom`.
pub fn apply(kind: &str, rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, PatchError> {
    match kind.to_ascii_lowercase().as_str() {
        "ips" => ips(rom, patch),
        "bps" => bps(rom, patch),
        "ups" => ups(rom, patch),
        _ => Err(PatchError::Format),
    }
}

/// Where the patched game is put for the core to load.
fn scratch_for(rom: &Path) -> PathBuf {
    let name = rom
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    std::env::temp_dir().join("slot-patched").join(name)
}

/// The file the core should load: the game patched, when it has a patch that applies,
/// or the game itself.
pub fn prepare(rom: &Path) -> PathBuf {
    let Some(patch) = patch_for(rom) else {
        return rom.to_path_buf();
    };
    let kind = patch
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_string();
    let patched = std::fs::read(rom)
        .and_then(|r| std::fs::read(&patch).map(|p| (r, p)))
        .map_err(|e| e.to_string())
        .and_then(|(r, p)| apply(&kind, &r, &p).map_err(|e| e.to_string()));
    let out = scratch_for(rom);
    let written = patched.and_then(|game| {
        let dir = out.parent().ok_or("no scratch folder")?;
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        std::fs::write(&out, &game).map_err(|e| e.to_string())?;
        Ok(game.len())
    });
    match written {
        Ok(n) => {
            eprintln!(
                "slot: patch: {} applied, {} bytes, at {:.1} ms",
                patch.display(),
                n,
                crate::boot_time::ms()
            );
            out
        }
        Err(e) => {
            eprintln!(
                "slot: patch: {} not applied ({e}); playing the game as it is",
                patch.display()
            );
            rom.to_path_buf()
        }
    }
}

fn ips(rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, PatchError> {
    let body = patch.strip_prefix(b"PATCH").ok_or(PatchError::Format)?;
    let mut out = rom.to_vec();
    let mut at = 0usize;
    let take = |at: &mut usize, n: usize| -> Result<&[u8], PatchError> {
        let s = body.get(*at..*at + n).ok_or(PatchError::Damaged)?;
        *at += n;
        Ok(s)
    };
    loop {
        let offset = take(&mut at, 3)?;
        if offset == b"EOF" {
            // An optional truncation size may follow.
            if let Ok(size) = take(&mut at, 3) {
                let size = u32::from_be_bytes([0, size[0], size[1], size[2]]) as usize;
                out.truncate(size);
            }
            return Ok(out);
        }
        let offset = u32::from_be_bytes([0, offset[0], offset[1], offset[2]]) as usize;
        let len = take(&mut at, 2)?;
        let len = u16::from_be_bytes([len[0], len[1]]) as usize;
        let (bytes, n): (Vec<u8>, usize) = if len == 0 {
            let run = take(&mut at, 2)?;
            let n = u16::from_be_bytes([run[0], run[1]]) as usize;
            let value = take(&mut at, 1)?[0];
            (vec![value; n], n)
        } else {
            (take(&mut at, len)?.to_vec(), len)
        };
        if out.len() < offset + n {
            out.resize(offset + n, 0);
        }
        out[offset..offset + n].copy_from_slice(&bytes);
    }
}

fn crc(bytes: &[u8]) -> u32 {
    crc32fast::hash(bytes)
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// BPS and UPS's variable-length numbers.
fn number(p: &[u8], at: &mut usize) -> Result<u64, PatchError> {
    let mut data = 0u64;
    let mut shift = 1u64;
    loop {
        let x = *p.get(*at).ok_or(PatchError::Damaged)? as u64;
        *at += 1;
        data = data
            .checked_add((x & 0x7f).checked_mul(shift).ok_or(PatchError::Damaged)?)
            .ok_or(PatchError::Damaged)?;
        if x & 0x80 != 0 {
            return Ok(data);
        }
        shift = shift.checked_shl(7).ok_or(PatchError::Damaged)?;
        data = data.checked_add(shift).ok_or(PatchError::Damaged)?;
    }
}

/// The checksums at the end of a BPS or UPS patch: source, target, and the patch itself.
fn footer(patch: &[u8]) -> Result<(u32, u32), PatchError> {
    if patch.len() < 16 {
        return Err(PatchError::Format);
    }
    let end = patch.len() - 12;
    if crc(&patch[..patch.len() - 4]) != le32(&patch[patch.len() - 4..]) {
        return Err(PatchError::Damaged);
    }
    Ok((le32(&patch[end..]), le32(&patch[end + 4..])))
}

fn bps(rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, PatchError> {
    if !patch.starts_with(b"BPS1") {
        return Err(PatchError::Format);
    }
    let (source_crc, target_crc) = footer(patch)?;
    if crc(rom) != source_crc {
        return Err(PatchError::WrongRom);
    }
    let end = patch.len() - 12;
    let mut at = 4;
    let source_size = number(patch, &mut at)? as usize;
    let target_size = number(patch, &mut at)? as usize;
    let meta = number(patch, &mut at)? as usize;
    at = at
        .checked_add(meta)
        .filter(|a| *a <= end)
        .ok_or(PatchError::Damaged)?;
    if source_size != rom.len() {
        return Err(PatchError::WrongRom);
    }
    let mut out = vec![0u8; target_size];
    let (mut o, mut src_rel, mut dst_rel) = (0usize, 0i64, 0i64);
    while at < end {
        let data = number(patch, &mut at)?;
        let len = (data >> 2) as usize + 1;
        if o + len > target_size {
            return Err(PatchError::Damaged);
        }
        match data & 3 {
            0 => {
                // Source read: the same bytes as the source at this spot.
                let s = rom.get(o..o + len).ok_or(PatchError::Damaged)?;
                out[o..o + len].copy_from_slice(s);
            }
            1 => {
                let s = patch
                    .get(at..at + len)
                    .filter(|_| at + len <= end)
                    .ok_or(PatchError::Damaged)?;
                out[o..o + len].copy_from_slice(s);
                at += len;
            }
            kind => {
                let d = number(patch, &mut at)?;
                let step = if d & 1 != 0 {
                    -((d >> 1) as i64)
                } else {
                    (d >> 1) as i64
                };
                if kind == 2 {
                    src_rel += step;
                    for i in 0..len {
                        let from = usize::try_from(src_rel).map_err(|_| PatchError::Damaged)?;
                        out[o + i] = *rom.get(from).ok_or(PatchError::Damaged)?;
                        src_rel += 1;
                    }
                } else {
                    // Target copy: from what's already been written; may overlap itself.
                    dst_rel += step;
                    for i in 0..len {
                        let from = usize::try_from(dst_rel).map_err(|_| PatchError::Damaged)?;
                        if from >= o + i {
                            return Err(PatchError::Damaged);
                        }
                        out[o + i] = out[from];
                        dst_rel += 1;
                    }
                }
            }
        }
        o += len;
    }
    if crc(&out) != target_crc {
        return Err(PatchError::Result);
    }
    Ok(out)
}

fn ups(rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, PatchError> {
    if !patch.starts_with(b"UPS1") {
        return Err(PatchError::Format);
    }
    let (source_crc, target_crc) = footer(patch)?;
    let end = patch.len() - 12;
    let mut at = 4;
    let size_a = number(patch, &mut at)? as usize;
    let size_b = number(patch, &mut at)? as usize;
    // UPS works both ways: from A to B, or back from B to A.
    let rom_crc = crc(rom);
    let (target_size, want) = if rom.len() == size_a && rom_crc == source_crc {
        (size_b, target_crc)
    } else if rom.len() == size_b && rom_crc == target_crc {
        (size_a, source_crc)
    } else {
        return Err(PatchError::WrongRom);
    };
    let mut out = rom.to_vec();
    out.resize(target_size, 0);
    let mut o = 0usize;
    while at < end {
        o = o
            .checked_add(number(patch, &mut at)? as usize)
            .ok_or(PatchError::Damaged)?;
        loop {
            let x = *patch
                .get(at)
                .filter(|_| at < end)
                .ok_or(PatchError::Damaged)?;
            at += 1;
            if x == 0 {
                o += 1;
                break;
            }
            if o < target_size {
                out[o] ^= x;
            }
            o += 1;
        }
    }
    if crc(&out) != want {
        return Err(PatchError::Result);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(mut n: u64, out: &mut Vec<u8>) {
        loop {
            let x = (n & 0x7f) as u8;
            n >>= 7;
            if n == 0 {
                out.push(0x80 | x);
                return;
            }
            out.push(x);
            n -= 1;
        }
    }

    fn sealed(mut body: Vec<u8>, source: &[u8], target: &[u8]) -> Vec<u8> {
        body.extend(crc(source).to_le_bytes());
        body.extend(crc(target).to_le_bytes());
        let c = crc(&body);
        body.extend(c.to_le_bytes());
        body
    }

    #[test]
    fn ips_writes_records_runs_and_grows_the_rom() {
        let rom = vec![0u8; 8];
        let mut p = b"PATCH".to_vec();
        p.extend([0, 0, 2, 0, 3, 1, 2, 3]); // 3 bytes at 2
        p.extend([0, 0, 9, 0, 0, 0, 2, 7]); // a run of two 7s at 9: grows to 11
        p.extend(b"EOF");
        assert_eq!(
            apply("ips", &rom, &p).unwrap(),
            vec![0, 0, 1, 2, 3, 0, 0, 0, 0, 7, 7]
        );
    }

    #[test]
    fn ips_can_truncate() {
        let mut p = b"PATCH".to_vec();
        p.extend(b"EOF");
        p.extend([0, 0, 4]);
        assert_eq!(apply("IPS", &[9; 8], &p).unwrap(), vec![9; 4]);
    }

    #[test]
    fn a_cut_short_ips_is_damaged() {
        let mut p = b"PATCH".to_vec();
        p.extend([0, 0, 2, 0, 3, 1]);
        assert_eq!(apply("ips", &[0; 8], &p), Err(PatchError::Damaged));
        assert_eq!(apply("ips", &[0; 8], b"NOPE"), Err(PatchError::Format));
    }

    fn bps_patch(source: &[u8], target: &[u8], actions: &[(u8, usize, &[u8], i64)]) -> Vec<u8> {
        let mut p = b"BPS1".to_vec();
        enc(source.len() as u64, &mut p);
        enc(target.len() as u64, &mut p);
        enc(0, &mut p);
        for &(kind, len, data, rel) in actions {
            enc((((len - 1) as u64) << 2) | kind as u64, &mut p);
            match kind {
                1 => p.extend(data),
                2 | 3 => enc(((rel.unsigned_abs()) << 1) | (rel < 0) as u64, &mut p),
                _ => {}
            }
        }
        sealed(p, source, target)
    }

    #[test]
    fn bps_reads_source_writes_new_bytes_and_copies() {
        let source = b"HELLO WORLD".to_vec();
        let target = b"HELLO HELLO WORLD!!".to_vec();
        let p = bps_patch(
            &source,
            &target,
            &[
                (0, 6, b"", 0),  // "HELLO "
                (2, 11, b"", 0), // source copy from 0: "HELLO WORLD"
                (1, 1, b"!", 0), // "!"
                (3, 1, b"", 17), // target copy from 17: "!"
            ],
        );
        assert_eq!(apply("bps", &source, &p).unwrap(), target);
    }

    #[test]
    fn bps_refuses_the_wrong_rom_and_a_damaged_patch() {
        let source = b"ABCD".to_vec();
        let target = b"ABCE".to_vec();
        let mut p = bps_patch(&source, &target, &[(0, 3, b"", 0), (1, 1, b"E", 0)]);
        assert_eq!(apply("bps", &source, &p).unwrap(), target);
        assert_eq!(apply("bps", b"ABCX", &p), Err(PatchError::WrongRom));
        let n = p.len();
        p[n - 13] ^= 1;
        assert_eq!(apply("bps", &source, &p), Err(PatchError::Damaged));
    }

    fn ups_patch(a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut p = b"UPS1".to_vec();
        enc(a.len() as u64, &mut p);
        enc(b.len() as u64, &mut p);
        let n = a.len().max(b.len());
        let byte = |s: &[u8], i: usize| s.get(i).copied().unwrap_or(0);
        let mut i = 0;
        let mut last = 0;
        while i < n {
            if byte(a, i) == byte(b, i) {
                i += 1;
                continue;
            }
            enc((i - last) as u64, &mut p);
            while i < n && byte(a, i) != byte(b, i) {
                p.push(byte(a, i) ^ byte(b, i));
                i += 1;
            }
            p.push(0);
            i += 1;
            last = i;
        }
        sealed(p, a, b)
    }

    #[test]
    fn ups_works_both_ways() {
        let a = b"Pocket Monsters".to_vec();
        let b = b"Pocket Monsters (EN)".to_vec();
        let p = ups_patch(&a, &b);
        assert_eq!(apply("ups", &a, &p).unwrap(), b);
        assert_eq!(apply("ups", &b, &p).unwrap(), a);
        assert_eq!(
            apply("ups", b"something else", &p),
            Err(PatchError::WrongRom)
        );
    }

    #[test]
    fn a_patch_beside_the_game_is_found_and_applied_to_a_scratch_copy() {
        let d = tempfile::tempdir().unwrap();
        let rom = d.path().join("Golden Sun.gba");
        std::fs::write(&rom, [0u8; 8]).unwrap();
        assert_eq!(prepare(&rom), rom);
        let mut p = b"PATCH".to_vec();
        p.extend([0, 0, 1, 0, 1, 5]);
        p.extend(b"EOF");
        std::fs::write(d.path().join("Golden Sun.IPS"), &p).unwrap();
        let loaded = prepare(&rom);
        assert_ne!(loaded, rom);
        assert_eq!(loaded.file_name(), rom.file_name());
        assert_eq!(
            std::fs::read(&loaded).unwrap(),
            vec![0, 5, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(std::fs::read(&rom).unwrap(), vec![0; 8]);
    }

    #[test]
    fn a_patch_that_doesnt_apply_leaves_the_game_as_it_is() {
        let d = tempfile::tempdir().unwrap();
        let rom = d.path().join("Zelda.gba");
        std::fs::write(&rom, [0u8; 8]).unwrap();
        std::fs::write(d.path().join("Zelda.bps"), b"BPS1 broken").unwrap();
        assert_eq!(prepare(&rom), rom);
    }
}
