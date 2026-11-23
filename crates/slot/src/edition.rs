//! Which slotPlus this build is, so an update fetches the same one. From 3.1 there are two
//! editions built from the same code: slotPlus Ultra, with every option, and slotPlus Core
//! (the `core` feature), set up for you with fewer rows. Check for Updates switches between
//! them.

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether this build is slotPlus Core.
pub const CORE: bool = cfg!(feature = "core");

#[cfg(not(feature = "core"))]
mod this {
    pub const KEY: &str = "ultra";
    pub const ZIP: &str = "slot-plusultra.zip";
    pub const NAME: &str = "slotPlus Ultra";
    pub const OTHER_KEY: &str = "core";
    pub const OTHER_ZIP: &str = "slot-core.zip";
    pub const OTHER_NAME: &str = "slotPlus Core";
}

#[cfg(feature = "core")]
mod this {
    pub const KEY: &str = "core";
    pub const ZIP: &str = "slot-core.zip";
    pub const NAME: &str = "slotPlus Core";
    pub const OTHER_KEY: &str = "ultra";
    pub const OTHER_ZIP: &str = "slot-plusultra.zip";
    pub const OTHER_NAME: &str = "slotPlus Ultra";
}

/// The key of this edition's zip in `versions.json`.
pub const KEY: &str = this::KEY;
/// This edition's zip on each release.
pub const ZIP: &str = this::ZIP;
/// The name the update screens use.
pub const NAME: &str = this::NAME;

/// The other edition, which Check for Updates offers to switch to.
pub const OTHER_KEY: &str = this::OTHER_KEY;
pub const OTHER_ZIP: &str = this::OTHER_ZIP;
pub const OTHER_NAME: &str = this::OTHER_NAME;

/// Whether Check for Updates offers the other edition. A release from before 3.1 has no Core
/// zip, so a card on it simply finds nothing to switch to.
pub const OFFERS_OTHER: bool = true;

/// What `Config/edition.txt` holds, so a card knows which edition last ran on it.
pub const MARK_FILE: &str = "edition.txt";

static RUNS_CORE: AtomicBool = AtomicBool::new(CORE);

/// Whether slotPlus runs as Core: the build's own edition, or what a test asked for.
pub fn is_core() -> bool {
    RUNS_CORE.load(Ordering::Relaxed)
}

/// For tests: run this process as Core (or not), whichever edition it was built as.
#[doc(hidden)]
pub fn run_as_core(on: bool) {
    RUNS_CORE.store(on, Ordering::Relaxed);
}
