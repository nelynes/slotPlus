//! The CPU's frequency governor. The launch script sets schedutil, which
//! lets the H700 clock down when it is idle; a DS game runs so close to its
//! frame budget that schedutil's dips mid-race cost frames, so while one
//! plays the CPU is held at full speed, and handed back after.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

const POLICIES: &str = "/sys/devices/system/cpu/cpufreq";
const FULL: &str = "performance";
const NORMAL: &str = "schedutil";

static HELD: AtomicBool = AtomicBool::new(false);

/// Hold the CPU at full speed, or hand it back to schedutil. Only on the
/// handheld (slot is run from the card, which sets SLOT_ROOT), and only when
/// that changes, so it is cheap to call every frame.
pub fn hold_full(on: bool) {
    if HELD.swap(on, Ordering::Relaxed) == on || std::env::var_os("SLOT_ROOT").is_none() {
        return;
    }
    set(Path::new(POLICIES), if on { FULL } else { NORMAL });
}

fn set(policies: &Path, governor: &str) {
    let Ok(dirs) = std::fs::read_dir(policies) else {
        return;
    };
    for dir in dirs.flatten() {
        let name = dir.file_name();
        if !name.to_string_lossy().starts_with("policy") {
            continue;
        }
        let file = dir.path().join("scaling_governor");
        match std::fs::write(&file, governor) {
            Ok(()) => eprintln!("slot: cpu: {} {governor}", name.to_string_lossy()),
            Err(e) => eprintln!("slot: cpu: {governor} refused by {}: {e}", file.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_policy_is_set_and_nothing_else() {
        let d = tempfile::tempdir().unwrap();
        for p in ["policy0", "policy4", "boost"] {
            std::fs::create_dir(d.path().join(p)).unwrap();
            std::fs::write(d.path().join(p).join("scaling_governor"), "schedutil").unwrap();
        }
        set(d.path(), FULL);
        let read =
            |p: &str| std::fs::read_to_string(d.path().join(p).join("scaling_governor")).unwrap();
        assert_eq!(read("policy0"), "performance");
        assert_eq!(read("policy4"), "performance");
        assert_eq!(read("boost"), "schedutil");
    }
}
