//! Which slotPlus this build is, so an update fetches the same one. slotPlusUltra's
//! branch keeps its own copy of this file.

/// The key of this edition's zip in `versions.json`.
pub const KEY: &str = "plus";
/// This edition's zip on each release.
pub const ZIP: &str = "slot-plus.zip";
/// The name the update screens use.
pub const NAME: &str = "slotPlus";

/// slotPlus: the other edition, which Check for Updates offers to switch to.
pub const OTHER_KEY: &str = "ultra";
pub const OTHER_ZIP: &str = "slot-plusultra.zip";
pub const OTHER_NAME: &str = "slotPlusUltra";
