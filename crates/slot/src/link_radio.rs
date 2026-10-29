#[cfg(feature = "device")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "device")]
use std::sync::mpsc::{channel, Sender};
#[cfg(feature = "device")]
use std::sync::OnceLock;
#[cfg(any(feature = "device", test))]
use std::{path::Path, process::Command};

use crate::link_net::Cancel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkRole {
    Host,
    Join,
}

impl LinkRole {
    pub fn arg(self) -> &'static str {
        match self {
            LinkRole::Host => "host",
            LinkRole::Join => "join",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadioFail {
    NoHost,
    Cancelled,
    Radio(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioJob {
    /// slotPlus: whether Home Wi-Fi is wanted. It is paused while a link has the radio.
    Home(bool),
    Warm,
    Cool,
    Down,
}

pub trait RadioJobs: Send {
    fn ask(&mut self, job: RadioJob);

    fn warmed(&self) -> bool;

    /// The home network is joined and has an address: not just wanted.
    fn home_connected(&self) -> bool {
        false
    }
}

pub struct RadioQueue;

pub fn radio_jobs() -> Box<dyn RadioJobs> {
    Box::new(RadioQueue)
}

#[cfg(feature = "device")]
impl RadioJobs for RadioQueue {
    fn ask(&mut self, job: RadioJob) {
        if job == RadioJob::Home(false) {
            HOME_CONNECTED.store(false, Ordering::SeqCst);
        }
        let _ = queue().send(job);
    }

    fn warmed(&self) -> bool {
        WARM.load(Ordering::SeqCst)
    }

    fn home_connected(&self) -> bool {
        HOME_CONNECTED.load(Ordering::SeqCst)
    }
}

#[cfg(feature = "device")]
static WARM: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "device")]
static HOME_CONNECTED: AtomicBool = AtomicBool::new(false);
/// Home Wi-Fi is wanted, and whether a link has it paused.
#[cfg(feature = "device")]
static HOME_WANTED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "device")]
static HOME_PAUSED: AtomicBool = AtomicBool::new(false);

/// Hand the radio to a link: the link's own script runs wlan0 and wlan1, so the home
/// connection steps aside until the link is down.
#[cfg(feature = "device")]
fn pause_home() {
    if HOME_WANTED.load(Ordering::SeqCst) && !HOME_PAUSED.swap(true, Ordering::SeqCst) {
        HOME_CONNECTED.store(false, Ordering::SeqCst);
        let _ = services().args(["home", "off"]).status();
    }
}

#[cfg(feature = "device")]
fn resume_home() {
    if HOME_PAUSED.swap(false, Ordering::SeqCst) && HOME_WANTED.load(Ordering::SeqCst) {
        let _ = services().args(["home", "on"]).status();
    }
}

/// The card's `slot-services` (Home Wi-Fi and the clock), through the loader: exFAT
/// carries no exec bit. Adapted from Prashant Vaibhav's fork of Slot.
#[cfg(feature = "device")]
fn services() -> Command {
    let root = std::env::var_os("SLOT_ROOT").unwrap_or_else(|| "/mnt/sdcard".into());
    let service = Path::new(&root).join("System/slot-services");
    let loader = Path::new("/lib/ld-linux-aarch64.so.1");
    let mut c = if loader.exists() {
        let mut c = Command::new(loader);
        c.arg(service);
        c
    } else {
        Command::new(service)
    };
    c.env("SLOT_ROOT", root)
        .env("SLOT_OWNER_PID", std::process::id().to_string())
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    c
}

#[cfg(feature = "device")]
fn home_status() -> bool {
    services()
        .args(["service", "status"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .any(|v| v == "home_connected=true")
        })
}

#[cfg(feature = "device")]
fn queue() -> &'static Sender<RadioJob> {
    static Q: OnceLock<Sender<RadioJob>> = OnceLock::new();
    Q.get_or_init(|| {
        let (tx, rx) = channel::<RadioJob>();
        std::thread::spawn(move || loop {
            // Only asks after the home connection while it is wanted, so Wi-Fi left Off
            // never starts the helper.
            let job = match rx.recv_timeout(std::time::Duration::from_secs(3)) {
                Ok(job) => job,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if HOME_WANTED.load(Ordering::SeqCst) && !HOME_PAUSED.load(Ordering::SeqCst) {
                        HOME_CONNECTED.store(home_status(), Ordering::SeqCst);
                    }
                    continue;
                }
            };
            match job {
                RadioJob::Home(on) => {
                    let was = HOME_WANTED.swap(on, Ordering::SeqCst);
                    if !HOME_PAUSED.load(Ordering::SeqCst) && (on || was) {
                        let _ = services()
                            .args(["home", if on { "on" } else { "off" }])
                            .status();
                    }
                    if !on {
                        HOME_CONNECTED.store(false, Ordering::SeqCst);
                    }
                }
                RadioJob::Warm => {
                    pause_home();
                    WARM.store(run("warm"), Ordering::SeqCst)
                }
                RadioJob::Cool => {
                    WARM.store(false, Ordering::SeqCst);
                    run("cool");
                    resume_home();
                }
                RadioJob::Down => {
                    WARM.store(false, Ordering::SeqCst);
                    down();
                    resume_home();
                }
            }
        });
        tx
    })
}

#[cfg(any(feature = "device", test))]
fn link_command(root: &Path, verb: &str) -> Command {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg(root.join("System/slotlink.sh"))
        .arg("link")
        .arg(verb);
    cmd
}

#[cfg(feature = "device")]
fn link(verb: &str) -> Command {
    let root = std::env::var_os("SLOT_ROOT").unwrap_or_else(|| "/mnt/sdcard".into());
    link_command(Path::new(&root), verb)
}

#[cfg(feature = "device")]
fn run(sub: &str) -> bool {
    link(sub).status().is_ok_and(|status| status.success())
}

#[cfg(feature = "device")]
pub fn up(role: LinkRole, cancel: &Cancel) -> Result<(), RadioFail> {
    pause_home();
    let mut child = link(role.arg())
        .spawn()
        .map_err(|e| RadioFail::Radio(format!("link {} would not start: {e}", role.arg())))?;
    loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RadioFail::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) if status.code() == Some(3) => return Err(RadioFail::NoHost),
            Ok(Some(status)) => {
                return Err(RadioFail::Radio(format!(
                    "link {} failed: {status}",
                    role.arg()
                )))
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(e) => return Err(RadioFail::Radio(format!("link {}: {e}", role.arg()))),
        }
    }
}

#[cfg(feature = "device")]
pub fn down() {
    let _ = link("down").status();
}

#[cfg(not(feature = "device"))]
pub fn up(_role: LinkRole, _cancel: &Cancel) -> Result<(), RadioFail> {
    Ok(())
}

#[cfg(not(feature = "device"))]
pub fn down() {}

#[cfg(not(feature = "device"))]
impl RadioJobs for RadioQueue {
    fn ask(&mut self, _job: RadioJob) {}

    fn warmed(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_role_asks_for_its_own_subcommand() {
        assert_eq!(LinkRole::Host.arg(), "host");
        assert_eq!(LinkRole::Join.arg(), "join");
    }

    #[test]
    fn asking_for_a_job_is_never_an_error() {
        let mut jobs = radio_jobs();
        jobs.ask(RadioJob::Warm);
        jobs.ask(RadioJob::Cool);
        jobs.ask(RadioJob::Down);
    }

    fn argv(cmd: &Command) -> Vec<String> {
        std::iter::once(cmd.get_program())
            .chain(cmd.get_args())
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn a_link_runs_the_cards_script_through_sh() {
        let d = tempfile::tempdir().unwrap();
        let script = d.path().join("System/slotlink.sh");
        assert_eq!(
            argv(&link_command(d.path(), "host")),
            ["/bin/sh", script.to_str().unwrap(), "link", "host"]
        );
    }

    #[cfg(not(feature = "device"))]
    #[test]
    fn a_host_build_has_no_driver_left_to_load() {
        assert!(radio_jobs().warmed());
    }
}
