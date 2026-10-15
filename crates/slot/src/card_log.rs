use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use slot_store::{atomic_write, format_stamp};

pub const CARD_LOG_NAME: &str = "slot-card.log";

#[derive(Default)]
pub struct CardWrites {
    failing: AtomicBool,
    last_error: Mutex<Option<String>>,
}

impl CardWrites {
    pub fn wrote<T>(&self, what: &str, written: &std::io::Result<T>) {
        match written {
            Ok(_) => self.ok(),
            Err(e) => self.failed(what, e),
        }
    }

    pub fn ok(&self) {
        self.failing.store(false, Ordering::Relaxed);
        *self.last_error.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    pub fn failed(&self, what: &str, e: &std::io::Error) {
        *self.last_error.lock().unwrap_or_else(|e| e.into_inner()) = Some(format!("{what}: {e}"));
        self.failing.store(true, Ordering::Relaxed);
    }

    pub fn failing(&self) -> bool {
        self.failing.load(Ordering::Relaxed)
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

pub struct CardEvent<'a> {
    pub secs: i64,
    pub alarm: bool,
    pub read_only: bool,
    pub write_error: Option<&'a str>,
    pub cart: Option<&'a str>,
    pub kernel: &'a [String],
}

pub fn entry(e: &CardEvent) -> String {
    let mut out = format!(
        "{} alarm {} read-only {} build {}\n",
        format_stamp(e.secs),
        if e.alarm { "on" } else { "off" },
        e.read_only,
        crate::build_info::Build::current().serial(),
    );
    if let Some(err) = e.write_error {
        out.push_str(&format!("  write failed: {err}\n"));
    }
    if let Some(cart) = e.cart {
        out.push_str(&format!("  cart: {cart}\n"));
    }
    for line in e.kernel {
        out.push_str(&format!("  kernel: {line}\n"));
    }
    out
}

pub fn append(log: &Path, entry: &str) -> std::io::Result<()> {
    let mut f = OpenOptions::new().create(true).append(true).open(log)?;
    f.write_all(entry.as_bytes())?;
    f.sync_all()
}

pub fn copy_to_card(log: &Path, root: &Path) -> std::io::Result<bool> {
    let bytes = match std::fs::read(log) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let dir = crate::root::logs_dir(root);
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(CARD_LOG_NAME);
    if std::fs::read(&dest).is_ok_and(|old| old == bytes) {
        return Ok(false);
    }
    atomic_write(&dest, &bytes)?;
    Ok(true)
}
