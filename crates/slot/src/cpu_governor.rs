//! The CPU's frequency governor while a DS game plays. Slot leaves the
//! governor at the kernel's default, which on the H700 holds the CPU at its
//! full fixed speed; a DS game runs so close to its frame budget that any
//! governor clocking down mid-race costs frames, so while one plays every
//! policy is set to performance, and each is handed back to the governor it
//! had before, whatever that was.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const POLICIES: &str = "/sys/devices/system/cpu/cpufreq";
const FULL: &str = "performance";

static HELD: AtomicBool = AtomicBool::new(false);
/// Each policy's governor from before it was held, to give back.
static BEFORE: Mutex<Vec<(std::path::PathBuf, String)>> = Mutex::new(Vec::new());

/// Hold the CPU at full speed, or give each policy back its own governor.
/// Only on the handheld (slot is run from the card, which sets SLOT_ROOT),
/// and only when that changes, so it is cheap to call every frame.
pub fn hold_full(on: bool) {
    if HELD.swap(on, Ordering::Relaxed) == on || std::env::var_os("SLOT_ROOT").is_none() {
        return;
    }
    let mut before = BEFORE.lock().unwrap_or_else(|e| e.into_inner());
    match on {
        true => *before = hold(Path::new(POLICIES)),
        false => give_back(&std::mem::take(&mut *before)),
    }
}

/// Set every policy to performance; what each was set to before.
fn hold(policies: &Path) -> Vec<(std::path::PathBuf, String)> {
    let Ok(dirs) = std::fs::read_dir(policies) else {
        return Vec::new();
    };
    let mut was = Vec::new();
    for dir in dirs.flatten() {
        let name = dir.file_name();
        if !name.to_string_lossy().starts_with("policy") {
            continue;
        }
        let file = dir.path().join("scaling_governor");
        let Ok(old) = std::fs::read_to_string(&file) else {
            continue;
        };
        let old = old.trim().to_string();
        if old == FULL {
            continue;
        }
        match std::fs::write(&file, FULL) {
            Ok(()) => {
                eprintln!("slot: cpu: {} {FULL} (was {old})", name.to_string_lossy());
                was.push((file, old));
            }
            Err(e) => eprintln!("slot: cpu: {FULL} refused by {}: {e}", file.display()),
        }
    }
    was
}

fn give_back(was: &[(std::path::PathBuf, String)]) {
    for (file, old) in was {
        match std::fs::write(file, old) {
            Ok(()) => eprintln!("slot: cpu: {} back to {old}", file.display()),
            Err(e) => eprintln!("slot: cpu: {old} refused by {}: {e}", file.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_policy_goes_to_performance_and_back_to_its_own_governor() {
        let d = tempfile::tempdir().unwrap();
        for (p, gov) in [
            ("policy0", "schedutil"),
            ("policy4", "performance"),
            ("boost", "x"),
        ] {
            std::fs::create_dir(d.path().join(p)).unwrap();
            std::fs::write(d.path().join(p).join("scaling_governor"), gov).unwrap();
        }
        let read =
            |p: &str| std::fs::read_to_string(d.path().join(p).join("scaling_governor")).unwrap();
        let was = hold(d.path());
        assert_eq!(read("policy0"), "performance");
        assert_eq!(read("policy4"), "performance");
        assert_eq!(read("boost"), "x", "not a policy");
        assert_eq!(
            was.len(),
            1,
            "a policy already at performance is left alone"
        );
        give_back(&was);
        assert_eq!(read("policy0"), "schedutil", "back to what it had");
        assert_eq!(read("policy4"), "performance", "never touched");
    }
}
