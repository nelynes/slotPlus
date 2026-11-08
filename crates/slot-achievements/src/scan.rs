//! slotPlus: Download Achievement Sets. Run from Settings → RetroAchievements while the
//! player watches, not in the background: every GB, GBC and GBA ROM on the card is hashed,
//! looked up and its set and badges saved, so games can earn with Wi-Fi off. A ROM already
//! looked up is counted from the card without asking the server again.
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::Ordering;

use crate::network::{self, Failure, Transport};
use crate::storage::{self, Auth, Config, Game};
use crate::{badges, library, ScanReport, ScanState, Status};

/// A set slot can use: a real game on a tracked system with at least one official
/// achievement that isn't one of RetroAchievements' notices.
pub(crate) fn has_set(game: &Game) -> bool {
    game.supported_console() && game.earnable().next().is_some()
}

fn update(status: &Status, f: impl FnOnce(&mut ScanReport)) {
    f(&mut status.scan.lock().unwrap_or_else(|e| e.into_inner()));
}

fn finish(status: &Status, dir: &Path, state: ScanState) {
    library::flush();
    status.publish_sets(library::with_sets(dir));
    update(status, |r| {
        r.state = state;
        r.game.clear();
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    root: &Path,
    dir: &Path,
    config: &Config,
    auth: &mut Option<Auth>,
    verified: &mut bool,
    status: &Status,
    http: &mut impl Transport,
) {
    let stopped = || status.scan_stop.load(Ordering::Acquire);
    let systems = library::roms(root);
    update(status, |r| {
        *r = ScanReport {
            state: ScanState::SigningIn,
            systems: systems.len(),
            ..ScanReport::default()
        }
    });
    if !*verified {
        match network::login(http, config, auth.as_ref()) {
            Ok(signed_in) => {
                if storage::write(&dir.join("auth.json"), &signed_in).is_err() {
                    return finish(status, dir, ScanState::CardError);
                }
                *auth = Some(signed_in);
                *verified = true;
            }
            Err(Failure::Network) => return finish(status, dir, ScanState::Offline),
            Err(_) => return finish(status, dir, ScanState::SignInFailed),
        }
    }
    let Some(auth) = auth.clone() else {
        return finish(status, dir, ScanState::SignInFailed);
    };
    update(status, |r| r.state = ScanState::Running);
    let mut badges = badges::Queue::empty(dir);
    for (number, (_, paths)) in systems.iter().enumerate() {
        update(status, |r| r.system = number + 1);
        for path in paths {
            if stopped() {
                return finish(status, dir, ScanState::Stopped);
            }
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            update(status, |r| r.game = name);
            let Ok(hash) = library::hash(dir, path) else {
                update(status, |r| r.failed += 1);
                continue;
            };
            let cached = storage::read::<(Game, BTreeSet<u32>)>(&dir.join(format!("{hash}.json")))
                .ok()
                .map(|(game, _)| game)
                .filter(|game| game.id == 0 || game.supported_console());
            let (game, fresh) = match cached {
                Some(game) => (game, false),
                None => match network::prefetch(http, &auth, dir, &hash) {
                    Ok(game) => (game, true),
                    Err(Failure::Network) => return finish(status, dir, ScanState::Offline),
                    Err(_) => {
                        update(status, |r| r.failed += 1);
                        continue;
                    }
                },
            };
            let found = has_set(&game);
            library::checked(dir, path, network::now(), found);
            if !found {
                update(status, |r| r.none += 1);
                continue;
            }
            update(status, |r| match fresh {
                true => r.new += 1,
                false => r.already += 1,
            });
            status.add_set(path);
            // The game's badges come with it, so a stop part way leaves whole games.
            badges.enqueue(&game);
            while let Some(result) = badges.fetch_next(http) {
                if stopped() {
                    return finish(status, dir, ScanState::Stopped);
                }
                match result {
                    Ok(()) => update(status, |r| r.badges += 1),
                    Err(Failure::Network) => return finish(status, dir, ScanState::Offline),
                    // A badge that won't come shows the trophy instead; not worth stopping.
                    Err(_) => {}
                }
            }
            library::flush_due();
        }
    }
    finish(status, dir, ScanState::Done);
}
