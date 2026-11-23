//! Which slotPlus this build is, so an update fetches the same one. From 3.1 there is one
//! line, slotPlus Ultra; slotPlus Core will be built from the same code.

/// The key of this edition's zip in `versions.json`.
pub const KEY: &str = "ultra";
/// This edition's zip on each release.
pub const ZIP: &str = "slot-plusultra.zip";
/// The name the update screens use.
pub const NAME: &str = "slotPlus Ultra";

/// slotPlus: the other edition, which Check for Updates offers to switch to.
pub const OTHER_KEY: &str = "plus";
pub const OTHER_ZIP: &str = "slot-plus.zip";
pub const OTHER_NAME: &str = "slotPlus";

/// Whether Check for Updates offers the other edition. Not until SlotPlus Core is published:
/// a release's `slot-plus.zip` is Ultra itself under its old name, there so 3.0 slotPlus cards
/// update into Ultra, and switching to it would only install Ultra again.
pub const OFFERS_OTHER: bool = false;
