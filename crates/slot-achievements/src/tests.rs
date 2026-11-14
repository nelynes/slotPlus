use super::*;
use serde_json::{json, Value};
use slot_retro::{AvInfo, ButtonMask, CoreError};
use std::path::Path;
use storage::{Achievement, Auth, Game};

fn achievement(definition: &str) -> Achievement {
    Achievement {
        badge: "12345".into(),
        id: 7,
        title: "Test unlocked".into(),
        description: "A real runtime trigger".into(),
        points: 5,
        flags: 3,
        definition: definition.into(),
    }
}

fn game() -> Game {
    Game {
        presence: String::new(),
        id: 1,
        console: 5,
        title: "Test game".into(),
        achievements: vec![achievement("0xH000000=1")],
    }
}

fn configured() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("Config")).unwrap();
    std::fs::create_dir_all(root.path().join("Games/GBA")).unwrap();
    std::fs::write(
        root.path().join("Config/retroachievements.toml"),
        "enabled = true\nusername = 'Player'\ntoken = 'fixture-token'\n",
    )
    .unwrap();
    root
}

fn wait_for(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "worker did not finish");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn toml_config_accepts_comments_and_literal_credentials_without_leaking_errors() {
    let root = configured();
    let path = root.path().join("Config/retroachievements.toml");
    std::fs::write(
        &path,
        "# Account\nenabled = true\nusername = 'Player'\npassword = 'secret\\with#characters'\n",
    )
    .unwrap();
    let config = Config::read(&path).unwrap();
    assert!(config.enabled);
    assert_eq!(config.password, "secret\\with#characters");
    assert!(config.token.is_empty());
    std::fs::write(&path, "password = 'secret'\nenabeld = true\n").unwrap();
    assert!(matches!(
        Config::read(&path),
        Err("Invalid achievement config")
    ));
    let service = Service::start_with(root.path().into(), Offline);
    wait_for(|| service.sync_status() == SyncStatus::Attention);
    assert!(!service.enabled.load(Ordering::Acquire));
}

struct Offline;
impl network::Transport for Offline {
    fn call(&mut self, _: &[(&str, String)]) -> Result<Value, network::Failure> {
        Err(network::Failure::Network)
    }
}

struct Server {
    calls: mpsc::Sender<String>,
}

#[test]
fn wifi_reconnect_wakes_offline_sync_without_restarting_or_opening_a_game() {
    struct Reconnect {
        online: Arc<AtomicBool>,
        server: Server,
    }
    impl network::Transport for Reconnect {
        fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, network::Failure> {
            if self.online.load(Ordering::Acquire) {
                self.server.call(fields)
            } else {
                Err(network::Failure::Network)
            }
        }
    }
    let root = configured();
    let mut store = Store::open(root.path(), "Player").unwrap();
    store
        .record(Unlock {
            id: 7,
            hash: "abcdef".into(),
            earned_at: network::now(),
            synced: false,
        })
        .unwrap();
    let online = Arc::new(AtomicBool::new(false));
    let (calls, _) = mpsc::channel();
    let service = Service::start_with(
        root.path().into(),
        Reconnect {
            online: online.clone(),
            server: Server { calls },
        },
    );
    wait_for(|| service.sync_status() == SyncStatus::Offline);
    online.store(true, Ordering::Release);
    service.network_available();
    // wait_for's five-second deadline is much shorter than the thirty-second backoff.
    wait_for(|| service.sync_status() == SyncStatus::Ready);
    assert!(Store::open(root.path(), "Player").unwrap().unlocks[&7].synced);
}
impl network::Transport for Server {
    fn badge(&mut self, name: &str) -> Result<Vec<u8>, network::Failure> {
        let _ = self.calls.send(format!("badge:{name}"));
        Ok(badge_png())
    }
    fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, network::Failure> {
        let request = fields.iter().find(|(k, _)| *k == "r").unwrap().1.as_str();
        let _ = self.calls.send(request.into());
        Ok(match request {
            "login2" => json!({"Success":true,"User":"Player","Token":"fixture-token"}),
            "gameid" => json!({"Success":true,"GameID":1}),
            "patch" => json!({"Success":true,"PatchData":game()}),
            "unlocks" => json!({"Success":true,"UserUnlocks":[]}),
            "startsession" => json!({"Success":true,"Unlocks":[],"HardcoreUnlocks":[]}),
            "ping" => {
                assert!(fields
                    .iter()
                    .any(|(k, v)| *k == "m" && v == "Playing Test game"));
                assert!(fields.iter().any(|(k, v)| *k == "x" && v.len() == 32));
                json!({"Success":true})
            }
            "awardachievement" => json!({"Success":true}),
            _ => panic!("unexpected API {request}"),
        })
    }
}

fn badge_png() -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[40, 120, 200, 255].repeat(4))
            .unwrap();
    }
    bytes
}

#[test]
fn badge_cache_survives_restart_and_repairs_corrupt_images() {
    let root = configured();
    let store = Store::open(root.path(), "Player").unwrap();
    let path = badges::path(&store.dir, "12345").unwrap();
    let (calls, requests) = mpsc::channel();
    let mut http = Server { calls };
    let mut queue = badges::Queue::empty(&store.dir);
    queue.enqueue(&game());
    queue.enqueue(&game());
    assert_eq!(queue.fetch_next(&mut http), Some(Ok(())));
    assert_eq!(queue.fetch_next(&mut http), None);
    assert_eq!(requests.try_iter().count(), 1);
    let mut restarted = badges::Queue::empty(&store.dir);
    restarted.enqueue(&game());
    assert!(!restarted.pending());
    // Notifications carry a local path even offline, with no network dependency.
    assert_eq!(load_badge(&path).unwrap().width, 2);
    std::fs::write(&path, b"corrupted PNG").unwrap();
    restarted.enqueue(&game());
    assert_eq!(restarted.fetch_next(&mut http), Some(Ok(())));
    assert!(load_badge(&path).is_some());
    assert!(badges::path(&store.dir, "../../auth").is_none());
    assert!(badges::path(&store.dir, "https://example.org/badge").is_none());
}

#[test]
fn a_badge_failure_does_not_hold_up_awards_or_cached_game_data() {
    struct NoBadges(Server);
    impl network::Transport for NoBadges {
        fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, network::Failure> {
            self.0.call(fields)
        }
    }
    let root = configured();
    std::fs::write(root.path().join("Games/GBA/Test.gba"), b"fixture").unwrap();
    let mut store = Store::open(root.path(), "Player").unwrap();
    store
        .record(Unlock {
            id: 7,
            hash: "abcdef".into(),
            earned_at: network::now(),
            synced: false,
        })
        .unwrap();
    let (calls, _) = mpsc::channel();
    let service = Service::start_with(root.path().into(), NoBadges(Server { calls }));
    wait_for(|| service.enabled.load(Ordering::Acquire));
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Done);
    let report = service.scan_report();
    assert_eq!((report.new, report.badges), (1, 0));
    wait_for(|| Store::open(root.path(), "Player").unwrap().unlocks[&7].synced);
    let store = Store::open(root.path(), "Player").unwrap();
    let hash = format!("{:x}", md5::compute(b"fixture"));
    assert!(store.dir.join(format!("{hash}.json")).exists());
    assert!(load_badge(&badges::path(&store.dir, "12345").unwrap()).is_none());
}

#[derive(Default)]
struct TestCore {
    value: u8,
    console: u32,
}
impl RetroCore for TestCore {
    fn load(&mut self, _: &Path) -> Result<(), CoreError> {
        Ok(())
    }
    fn run_frame(&mut self, input: ButtonMask) {
        self.value = u8::from(input.0 != 0);
    }
    fn video_xrgb8888(&self) -> &[u8] {
        &[]
    }
    fn take_audio(&mut self) -> Vec<i16> {
        vec![]
    }
    fn serialize(&mut self) -> Result<Vec<u8>, CoreError> {
        Ok(vec![self.value])
    }
    fn unserialize(&mut self, _: &[u8]) -> Result<(), CoreError> {
        Ok(())
    }
    fn save_ram(&self) -> Option<Vec<u8>> {
        None
    }
    fn load_save_ram(&mut self, _: &[u8]) -> Result<(), CoreError> {
        Ok(())
    }
    fn av_info(&self) -> AvInfo {
        AvInfo {
            fps: 60.0,
            sample_rate: 48000.0,
        }
    }
    fn achievement_memory(&self, _console: u32, ram: &mut [u8]) -> AchievementMemory {
        if matches!(self.console, 4 | 6) {
            ram[0xc000] = self.value;
            AchievementMemory::new([0xc000..0xe000, 0xff80..0xffff])
        } else {
            ram[0] = self.value;
            AchievementMemory::new([0..0x8000, 0x8000..0x48000])
        }
    }
}

#[test]
fn runtime_evaluates_real_definitions_and_never_awards_missing_memory() {
    let mut runtime = runtime::Runtime::new().unwrap();
    assert!(runtime.activate(7, "0xH000000=1"));
    assert!(runtime.activate(8, "0xH048000=1"));
    assert!(!runtime.activate(9, "not a definition"));
    let mut ram = vec![0; RAM_SIZE];
    assert!(runtime
        .frame(&ram, &AchievementMemory::new([0..0x8000, 0x8000..0x48000]))
        .is_empty());
    ram[0] = 1;
    ram[0x48000] = 1;
    assert_eq!(
        runtime.frame(&ram, &AchievementMemory::new([0..0x8000, 0x8000..0x48000])),
        &[7]
    );
    assert!(runtime
        .frame(&ram, &AchievementMemory::new([0..0x8000, 0x8000..0x48000]))
        .is_empty());
}

#[test]
fn offline_unlock_is_durable_then_syncs_without_reopening_the_game() {
    for (console, shelf, ext) in [(5, "GBA", "gba"), (4, "GB", "gb"), (6, "GBC", "gbc")] {
        offline_unlock_for(console, shelf, ext);
    }
}

fn offline_unlock_for(console: u32, shelf: &str, ext: &str) {
    let root = configured();
    let dir = root.path().join("Games").join(shelf);
    std::fs::create_dir_all(&dir).unwrap();
    let rom = dir.join(format!("Test.{ext}"));
    std::fs::write(&rom, b"fixture ROM").unwrap();
    let store = Store::open(root.path(), "Player").unwrap();
    let hash = library::hash(&store.dir, &rom).unwrap();
    storage::write(
        &store.dir.join("auth.json"),
        &Auth {
            username: "Player".into(),
            token: "fixture-token".into(),
        },
    )
    .unwrap();
    let mut data = game();
    data.console = console;
    if console != 5 {
        data.achievements[0].definition = "0xH00c000=1".into();
    }
    storage::write(
        &store.dir.join(format!("{hash}.json")),
        &(data, std::collections::BTreeSet::<u32>::new()),
    )
    .unwrap();
    let service = Service::start_with(root.path().into(), Offline);
    let mut core = service.wrap(Box::new(TestCore {
        console,
        ..Default::default()
    }));
    core.load(&rom).unwrap();
    wait_for(|| {
        service
            .take_notice()
            .is_some_and(|n| n.title.starts_with("Achievements ready"))
    });
    core.run_frame(ButtonMask(0));
    core.run_frame(ButtonMask(1));
    // Eject immediately: queued final frames must be evaluated before Unload retires them.
    drop(core);
    wait_for(|| {
        service
            .take_notice()
            .is_some_and(|n| n.kind == NoticeKind::Earned)
    });
    let saved = Store::open(root.path(), "Player").unwrap();
    assert!(!saved.unlocks[&7].synced);
    assert_eq!(saved.unlocks[&7].hash, hash);
    wait_for(|| service.sync_status() == SyncStatus::Offline);
    drop(service);

    let restarted = Service::start_with(root.path().into(), Offline);
    let mut core = restarted.wrap(Box::new(TestCore {
        console,
        ..Default::default()
    }));
    core.load(&rom).unwrap();
    wait_for(|| {
        restarted
            .take_notice()
            .is_some_and(|n| n.title.starts_with("Achievements ready"))
    });
    core.run_frame(ButtonMask(0));
    core.run_frame(ButtonMask(1));
    drop(core);
    // A reconnect uses the same persistent account ledger without needing a running core.
    drop(restarted);
    let (calls, requests) = mpsc::channel();
    let online = Service::start_with(root.path().into(), Server { calls });
    wait_for(|| {
        online
            .take_notice()
            .is_some_and(|n| n.title == "Achievements synced")
    });
    assert!(Store::open(root.path(), "Player").unwrap().unlocks[&7].synced);
    assert_eq!(
        requests
            .try_iter()
            .filter(|r| r == "awardachievement")
            .count(),
        1
    );
    assert!(Store::open(root.path(), "OtherPlayer")
        .unwrap()
        .unlocks
        .is_empty());
}

#[test]
fn download_achievement_sets_prepares_the_library_without_starting_sessions() {
    let root = configured();
    let rom = root.path().join("Games/GBA/Unopened.gba");
    std::fs::write(&rom, b"unopened ROM").unwrap();
    let (calls, requests) = mpsc::channel();
    let service = Service::start_with(root.path().into(), Server { calls });
    let store = Store::open(root.path(), "Player").unwrap();
    let hash = format!("{:x}", md5::compute(b"unopened ROM"));
    wait_for(|| service.sync_status() == SyncStatus::Ready);
    // Nothing is fetched until the player asks.
    std::thread::sleep(Duration::from_millis(300));
    assert!(!store.dir.join(format!("{hash}.json")).exists());
    assert!(service.achievement_sets().1.is_empty());
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Done);
    let report = service.scan_report();
    assert_eq!(
        (report.systems, report.system, report.new, report.already, report.none, report.badges),
        (1, 1, 1, 0, 0, 1)
    );
    assert!(service.achievement_sets().1.contains(&rom));
    let badge = badges::path(&store.dir, "12345").unwrap();
    assert_eq!(
        load_badge(&badge).unwrap().rgba,
        [40, 120, 200, 255].repeat(4)
    );
    let (cached, _): (Game, std::collections::BTreeSet<u32>) =
        storage::read(&store.dir.join(format!("{hash}.json"))).unwrap();
    assert_eq!(cached.achievements[0].id, 7);
    let calls: Vec<_> = requests.try_iter().collect();
    assert!(calls.iter().any(|c| c == "unlocks"));
    assert!(!calls.iter().any(|c| c == "startsession"));
    // A second run counts it from the card and asks the server nothing about it.
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Done);
    let report = service.scan_report();
    assert_eq!((report.new, report.already, report.badges), (0, 1, 0));
    assert!(!requests.try_iter().any(|c| c == "gameid" || c.starts_with("badge")));
    // Refresh All Sets asks again for the set on the card, and its earned list, but leaves
    // the badges already there.
    service.start_refresh();
    wait_for(|| service.scan_report().state == ScanState::Done);
    let report = service.scan_report();
    assert!(report.refresh);
    assert_eq!((report.new, report.refreshed, report.already, report.badges), (0, 1, 0, 0));
    let calls: Vec<_> = requests.try_iter().collect();
    assert!(calls.iter().any(|c| c == "patch") && calls.iter().any(|c| c == "unlocks"));
    assert!(!calls.iter().any(|c| c.starts_with("badge")));
    // And a plain run afterwards is a plain run again.
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Done);
    assert_eq!(service.scan_report().already, 1);
    // A changed ROM is a different dump: no trophy until it's scanned again.
    std::fs::write(&rom, b"changed content, different size").unwrap();
    assert!(library::with_sets(&store.dir).is_empty());
    assert_ne!(library::hash(&store.dir, &rom).unwrap(), hash);
}

#[test]
fn a_scan_counts_roms_without_achievements_and_stops_when_asked() {
    struct Mixed(Server);
    impl network::Transport for Mixed {
        fn badge(&mut self, name: &str) -> Result<Vec<u8>, network::Failure> {
            self.0.badge(name)
        }
        fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, network::Failure> {
            let none = fields.iter().any(|(k, v)| *k == "m" && v == &format!("{:x}", md5::compute(b"homebrew")));
            match none {
                true => Ok(json!({"Success":true,"GameID":0})),
                false => self.0.call(fields),
            }
        }
    }
    let root = configured();
    std::fs::create_dir_all(root.path().join("Games/GB")).unwrap();
    std::fs::write(root.path().join("Games/GB/Homebrew.gb"), b"homebrew").unwrap();
    std::fs::write(root.path().join("Games/GBA/Game.gba"), b"a game").unwrap();
    let (calls, _) = mpsc::channel();
    let service = Service::start_with(root.path().into(), Mixed(Server { calls }));
    wait_for(|| service.enabled.load(Ordering::Acquire));
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Done);
    let report = service.scan_report();
    assert_eq!((report.systems, report.new, report.none), (2, 1, 1));
    // Asked to stop before the worker picks it up: it never starts.
    service.start_scan();
    service.stop_scan();
    assert_eq!(service.scan_report().state, ScanState::Stopped);
}

#[test]
fn a_scan_without_an_account_says_so() {
    let root = tempfile::tempdir().unwrap();
    let service = Service::start_with(root.path().into(), Offline);
    service.start_scan();
    assert_eq!(service.scan_report().state, ScanState::NoAccount);
    let root = configured();
    std::fs::write(root.path().join("Games/GBA/Game.gba"), b"a game").unwrap();
    let service = Service::start_with(root.path().into(), Offline);
    wait_for(|| service.enabled.load(Ordering::Acquire));
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Offline);
}

#[test]
fn library_preparation_includes_game_boy_and_color_shelves() {
    let root = configured();
    for (shelf, name) in [
        ("GBA", "Advance.GBA"),
        ("GB", "Original.GB"),
        ("GB", "Color.gbc"),
        ("GBC", "Color.GBC"),
        ("GBC", "Original.gb"),
    ] {
        let dir = root.path().join("Games").join(shelf);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), name).unwrap();
    }
    std::fs::write(root.path().join("Games/GB/.hidden.gb"), b"hidden").unwrap();
    std::fs::write(root.path().join("Games/GB/Notes.txt"), b"notes").unwrap();
    let roms = library::roms(root.path());
    assert_eq!(roms.len(), 3);
    assert_eq!(roms.iter().map(|(_, paths)| paths.len()).sum::<usize>(), 5);
}

#[test]
fn game_boy_consoles_prepare_offline_caches_and_play_without_the_network() {
    struct ConsoleServer {
        server: Server,
        console: u32,
    }
    impl network::Transport for ConsoleServer {
        fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, network::Failure> {
            let mut response = self.server.call(fields)?;
            if response.get("PatchData").is_some() {
                response["PatchData"]["ConsoleID"] = json!(self.console);
            }
            Ok(response)
        }
        fn badge(&mut self, name: &str) -> Result<Vec<u8>, network::Failure> {
            self.server.badge(name)
        }
    }
    for (console, shelf, ext) in [(4, "GB", "gb"), (6, "GBC", "gbc")] {
        let root = configured();
        let dir = root.path().join("Games").join(shelf);
        std::fs::create_dir_all(&dir).unwrap();
        let rom = dir.join(format!("Test.{ext}"));
        std::fs::write(&rom, b"fixture ROM").unwrap();
        let (calls, requests) = mpsc::channel();
        let service = Service::start_with(
            root.path().into(),
            ConsoleServer {
                server: Server { calls },
                console,
            },
        );
        wait_for(|| service.enabled.load(Ordering::Acquire));
        service.start_scan();
        wait_for(|| service.scan_report().state == ScanState::Done);
        let store = Store::open(root.path(), "Player").unwrap();
        let hash = library::content_hash(&rom).unwrap();
        let cache = store.dir.join(format!("{hash}.json"));
        let (data, _): (Game, std::collections::BTreeSet<u32>) = storage::read(&cache).unwrap();
        assert_eq!(data.console, console);
        assert_eq!(
            artwork_title(root.path(), &rom).as_deref(),
            Some("Test game")
        );
        assert!(!requests.try_iter().any(|r| r == "startsession"));
        let mut core = service.wrap(Box::<TestCore>::default());
        core.load(&rom).unwrap();
        wait_for(|| tracked(&service));
        wait_for(|| service.take_set_ready());
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(requests.try_iter().count(), 0, "a game in play never calls out");
    }
}

#[test]
fn game_boy_runtime_reads_banked_ram_and_rejects_memory_holes() {
    let mut runtime = runtime::Runtime::new().unwrap();
    for (id, definition) in [
        (1, "0xH00c000=1"),
        (2, "0xH00ff80=1"),
        (3, "0xH010000=1"),
        (4, "0xH015fff=1"),
        (5, "0xH016000=1"),
        (6, "0xH033fff=1"),
        (7, "0xH00fea0=0_0xH00c000=1"),
    ] {
        assert!(runtime.activate(id, definition));
    }
    let memory = AchievementMemory::new([0xc000..0xfea0, 0xff80..0xffff, 0x10000..0x34000]);
    let mut ram = vec![0; RAM_SIZE];
    assert!(runtime.frame(&ram, &memory).is_empty());
    for address in [0xc000, 0xff80, 0x10000, 0x15fff, 0x16000, 0x33fff] {
        ram[address] = 1;
    }
    let mut earned = runtime.frame(&ram, &memory).to_vec();
    earned.sort_unstable();
    assert_eq!(earned, [1, 2, 3, 4, 5, 6]);

    // Indirect reads into a hole must not treat the missing byte as zero.
    let mut indirect = runtime::Runtime::new().unwrap();
    assert!(indirect.activate(8, "I:0xX00c010_0xH000000=0_0xH00c000=1"));
    ram[0xc010..0xc013].copy_from_slice(&[0xa0, 0xfe, 0x00]);
    ram[0xc000] = 0;
    assert!(indirect.frame(&ram, &memory).is_empty());
    ram[0xc000] = 1;
    assert!(indirect.frame(&ram, &memory).is_empty());
}

#[test]
fn a_full_worker_queue_cannot_block_the_emulator() {
    let (controls, _controls_rx) = mpsc::channel();
    let (frames, _frames_rx) = mpsc::sync_channel(QUEUE_SIZE);
    let (_, notices) = mpsc::channel();
    let service = Service {
        controls,
        frames,
        notices,
        enabled: Arc::new(AtomicBool::new(true)),
        current: Arc::new(AtomicU64::new(1)),
        tracking: Arc::new(AtomicU64::new(0)),
        flushing: AtomicBool::new(false),
        flushed: Arc::new(AtomicBool::new(false)),
        status: Arc::new(Status::default()),
    };
    let mut core = service.wrap(Box::<TestCore>::default());
    let (done, completed) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        for _ in 0..1000 {
            core.run_frame(ButtonMask(1));
        }
        done.send(()).unwrap();
    });
    let result = completed.recv_timeout(Duration::from_secs(2));
    // Release the receiver even on failure, so a regressed blocking send can unwind.
    drop(_frames_rx);
    worker.join().unwrap();
    result.expect("emulation waited for the achievement worker");
}

#[test]
fn damaged_ledger_is_preserved_and_failed_writes_are_not_acknowledged() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path(), "Player").unwrap();
    let unlock = Unlock {
        id: 7,
        hash: "hash".into(),
        earned_at: 1000,
        synced: false,
    };
    store.record(unlock.clone()).unwrap();
    assert!(!store.record(unlock).unwrap());
    let path = store.dir.join("unlocks.json");
    std::fs::write(&path, b"broken").unwrap();
    assert!(Store::open(root.path(), "Player").is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"broken");
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert!(store.ack(7).is_err());
    assert!(!store.unlocks[&7].synced);
}

fn frame(sequence: u64, timeline: u64, value: u8) -> Frame {
    let mut ram = vec![0; RAM_SIZE];
    ram[0] = value;
    let (recycle, _) = mpsc::sync_channel(1);
    Frame {
        generation: 1,
        sequence,
        timeline,
        earned_at: 1000,
        ram,
        valid: AchievementMemory::new([0..0x8000, 0x8000..0x48000]),
        recycle,
    }
}

#[test]
fn dropped_frames_and_rewinds_reset_partial_hit_counts() {
    for timeline in [0, 1] {
        let root = tempfile::tempdir().unwrap();
        let store = Arc::new(Mutex::new(Store::open(root.path(), "Player").unwrap()));
        let mut runtime = runtime::Runtime::new().unwrap();
        let a = achievement("0xH000000=1.2.");
        assert!(runtime.activate(a.id, &a.definition));
        let mut playing = Some(Playing {
            generation: 1,
            hash: "hash".into(),
            runtime,
            achievements: [(7, a)].into(),
            encore: Default::default(),
            previous: None,
            warned_gap: false,
        });
        let (notices, _) = mpsc::channel();
        let mut unsaved = BTreeMap::new();
        evaluate(frame(1, 0, 0), &mut playing, &store, &notices, &mut unsaved);
        evaluate(frame(2, 0, 1), &mut playing, &store, &notices, &mut unsaved);
        evaluate(
            frame(if timeline == 0 { 4 } else { 3 }, timeline, 1),
            &mut playing,
            &store,
            &notices,
            &mut unsaved,
        );
        assert!(store.lock().unwrap().unlocks.is_empty());
        evaluate(
            frame(5, timeline, 0),
            &mut playing,
            &store,
            &notices,
            &mut unsaved,
        );
        evaluate(
            frame(6, timeline, 1),
            &mut playing,
            &store,
            &notices,
            &mut unsaved,
        );
        evaluate(
            frame(7, timeline, 1),
            &mut playing,
            &store,
            &notices,
            &mut unsaved,
        );
        assert!(store.lock().unwrap().unlocks.contains_key(&7));
    }
}

#[test]
fn a_cached_game_can_earn_and_flush_while_https_is_stalled() {
    struct Stalled {
        entered: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
    }
    impl network::Transport for Stalled {
        fn call(&mut self, _: &[(&str, String)]) -> Result<Value, network::Failure> {
            let _ = self.entered.send(());
            let _ = self.release.recv();
            Err(network::Failure::Network)
        }
    }
    let root = configured();
    let rom = root.path().join("Games/GBA/Test.gba");
    std::fs::write(&rom, b"fixture ROM").unwrap();
    let store = Store::open(root.path(), "Player").unwrap();
    let hash = library::hash(&store.dir, &rom).unwrap();
    storage::write(
        &store.dir.join("auth.json"),
        &Auth {
            username: "Player".into(),
            token: "fixture-token".into(),
        },
    )
    .unwrap();
    storage::write(
        &store.dir.join(format!("{hash}.json")),
        &(game(), std::collections::BTreeSet::<u32>::new()),
    )
    .unwrap();
    let (entered, blocked) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let service = Service::start_with(
        root.path().into(),
        Stalled {
            entered,
            release: wait,
        },
    );
    blocked.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(service.sync_status(), SyncStatus::Syncing);
    let mut core = service.wrap(Box::<TestCore>::default());
    core.load(&rom).unwrap();
    wait_for(|| {
        service
            .take_notice()
            .is_some_and(|n| n.title.starts_with("Achievements ready"))
    });
    core.run_frame(ButtonMask(0));
    core.run_frame(ButtonMask(1));
    wait_for(|| service.flush_ready());
    assert!(Store::open(root.path(), "Player")
        .unwrap()
        .unlocks
        .contains_key(&7));
    // No HTTPS response was needed for preparation, evaluation, notification, or flushing.
    drop(release);
}

#[test]
fn disabling_achievements_cannot_hold_up_poweroff() {
    let root = tempfile::tempdir().unwrap();
    let service = Service::start_with(root.path().into(), Offline);
    wait_for(|| service.flush_ready());
    assert_eq!(service.sync_status(), SyncStatus::Disabled);
}

#[test]
fn unsaved_unlocks_retry_with_the_original_time_after_storage_recovers() {
    let root = tempfile::tempdir().unwrap();
    let store = Arc::new(Mutex::new(Store::open(root.path(), "Player").unwrap()));
    let path = store.lock().unwrap().dir.join("unlocks.json");
    std::fs::create_dir(&path).unwrap();
    let mut runtime = runtime::Runtime::new().unwrap();
    let a = achievement("0xH000000=1");
    runtime.activate(a.id, &a.definition);
    let mut playing = Some(Playing {
        generation: 1,
        hash: "hash".into(),
        runtime,
        achievements: [(7, a)].into(),
        encore: Default::default(),
        previous: None,
        warned_gap: false,
    });
    let (notices, received) = mpsc::channel();
    let mut unsaved = BTreeMap::new();
    evaluate(frame(1, 0, 0), &mut playing, &store, &notices, &mut unsaved);
    evaluate(frame(2, 0, 1), &mut playing, &store, &notices, &mut unsaved);
    assert_eq!(unsaved.len(), 1);
    assert!(store.lock().unwrap().unlocks.is_empty());
    assert!(!received.try_iter().any(|n| n.kind == NoticeKind::Earned));
    std::fs::remove_dir(path).unwrap();
    retry_unsaved(&mut unsaved, &store, &notices);
    assert!(unsaved.is_empty());
    assert_eq!(
        Store::open(root.path(), "Player").unwrap().unlocks[&7].earned_at,
        1000
    );
    assert!(received.try_iter().any(|n| n.kind == NoticeKind::Earned));
}


#[test]
fn rich_presence_tracks_memory_and_clears_invalid_replacements() {
    let mut runtime = runtime::Runtime::new().unwrap();
    assert!(runtime.activate_presence("Display:\n?0xH000000=1?In the castle\nOn the map"));
    let mut ram = vec![0; RAM_SIZE];
    runtime.frame(
        &ram,
        &AchievementMemory::new([0..0x8000, 0x8000..0x48000, 0x48000..RAM_SIZE]),
    );
    assert_eq!(runtime.presence(), "On the map");
    ram[0] = 1;
    runtime.frame(
        &ram,
        &AchievementMemory::new([0..0x8000, 0x8000..0x48000, 0x48000..RAM_SIZE]),
    );
    assert_eq!(runtime.presence(), "In the castle");
    assert!(!runtime.activate_presence("not a valid script"));
    runtime.frame(
        &ram,
        &AchievementMemory::new([0..0x8000, 0x8000..0x48000, 0x48000..RAM_SIZE]),
    );
    assert!(runtime.presence().is_empty());
}

#[test]
fn rich_presence_cache_accepts_old_data_and_retains_new_scripts() {
    let mut value = serde_json::to_value(game()).unwrap();
    value.as_object_mut().unwrap().remove("RichPresencePatch");
    assert!(serde_json::from_value::<Game>(value)
        .unwrap()
        .presence
        .is_empty());
    let mut data = game();
    data.presence = "Display:\nOn the map".into();
    let root = configured();
    let path = root.path().join("game.json");
    storage::write(&path, &data).unwrap();
    assert_eq!(
        storage::read::<Game>(&path).unwrap().presence,
        data.presence
    );
}

#[test]
fn null_rich_presence_from_the_service_is_an_empty_script() {
    let mut value = serde_json::to_value(game()).unwrap();
    value["RichPresencePatch"] = Value::Null;
    let parsed = serde_json::from_value::<Game>(value.clone()).unwrap();
    assert!(parsed.presence.is_empty());
    assert_eq!(parsed.achievements.len(), 1);
    value["RichPresencePatch"] = json!(42);
    assert!(serde_json::from_value::<Game>(value).is_err());
}

#[test]
fn artwork_identity_follows_rom_bytes_instead_of_its_filename() {
    let root = configured();
    let rom = root.path().join("Games/GBA/Completely renamed.gba");
    std::fs::write(&rom, b"same rom bytes").unwrap();
    let hash = library::content_hash(&rom).unwrap();
    let store = Store::open(root.path(), "Player").unwrap();
    let mut data = game();
    data.title = "Tomb Raider: Legend".into();
    let ids = std::collections::BTreeSet::<u32>::new();
    storage::write(&store.dir.join(format!("{hash}.json")), &(&data, ids)).unwrap();
    assert_eq!(
        artwork_title(root.path(), &rom).as_deref(),
        Some("Tomb Raider: Legend")
    );
    let moved = rom.with_file_name("Yet another name.gba");
    std::fs::rename(&rom, &moved).unwrap();
    assert_eq!(
        artwork_title(root.path(), &moved).as_deref(),
        Some("Tomb Raider: Legend")
    );
    std::fs::write(&moved, b"different rom bytes").unwrap();
    assert!(artwork_title(root.path(), &moved).is_none());
}

#[test]
fn a_rom_is_ready_offline_once_a_scan_caches_it() {
    let root = configured();
    let rom = root.path().join("Games/GBA/Prepared.gba");
    std::fs::write(&rom, b"prepared ROM").unwrap();
    let mut readiness = Readiness::default();
    assert!(readiness.scan(root.path()).is_empty());
    let (calls, _) = mpsc::channel();
    let service = Service::start_with(root.path().into(), Server { calls });
    wait_for(|| service.enabled.load(Ordering::Acquire));
    service.start_scan();
    wait_for(|| readiness.scan(root.path()).contains(&rom));
}

#[test]
fn readiness_needs_a_matching_dump_with_core_achievements() {
    let root = configured();
    let store = Store::open(root.path(), "Player").unwrap();
    let cache = |rom: &Path, game: Game| {
        let hash = library::hash(&store.dir, rom).unwrap();
        storage::write(
            &store.dir.join(format!("{hash}.json")),
            &(game, std::collections::BTreeSet::<u32>::new()),
        )
        .unwrap();
    };
    let rom = |name: &str| {
        let path = root.path().join("Games/GBA").join(name);
        std::fs::write(&path, name).unwrap();
        path
    };
    let known = rom("Known.gba");
    cache(&known, game());
    let unknown = rom("Unknown.gba");
    cache(
        &unknown,
        Game {
            id: 0,
            achievements: vec![],
            ..game()
        },
    );
    let unofficial = rom("Unofficial.gba");
    let mut draft = game();
    draft.achievements[0].flags = 5;
    cache(&unofficial, draft);
    rom("Unchecked.gba");
    let mut readiness = Readiness::default();
    assert_eq!(
        readiness.scan(root.path()),
        std::collections::HashSet::from([known.clone()])
    );
    // Another dump under the same name has a different hash, and nothing cached for it yet.
    std::fs::write(&known, b"a different dump of the same game").unwrap();
    assert!(readiness.scan(root.path()).is_empty());
    std::fs::write(
        root.path().join("Config/retroachievements.toml"),
        "enabled = false\nusername = 'Player'\n",
    )
    .unwrap();
    cache(&known, game());
    assert!(readiness.scan(root.path()).is_empty());
}

#[test]
fn a_renamed_rom_is_ready_again_once_it_is_loaded_offline() {
    let root = configured();
    let old = root.path().join("Games/GBA/Old name.gba");
    std::fs::write(&old, b"renamed ROM").unwrap();
    let store = Store::open(root.path(), "Player").unwrap();
    let hash = library::hash(&store.dir, &old).unwrap();
    storage::write(
        &store.dir.join(format!("{hash}.json")),
        &(game(), std::collections::BTreeSet::<u32>::new()),
    )
    .unwrap();
    let new = root.path().join("Games/GBA/New name.gba");
    std::fs::rename(&old, &new).unwrap();
    let mut readiness = Readiness::default();
    assert!(readiness.scan(root.path()).is_empty());
    let service = Service::start_with(root.path().into(), Offline);
    let before = service.cache_generation();
    let mut core = service.wrap(Box::<TestCore>::default());
    core.load(&new).unwrap();
    wait_for(|| service.cache_generation() != before);
    assert!(readiness.scan(root.path()).contains(&new));
}

#[test]
fn a_verdict_is_not_carried_over_to_another_account() {
    let root = configured();
    let rom = root.path().join("Games/GBA/Shared.gba");
    std::fs::write(&rom, b"shared ROM").unwrap();
    let cache = |user: &str, game: Game| {
        let store = Store::open(root.path(), user).unwrap();
        let hash = library::hash(&store.dir, &rom).unwrap();
        let path = store.dir.join(format!("{hash}.json"));
        storage::write(&path, &(game, std::collections::BTreeSet::<u32>::new())).unwrap();
        path
    };
    let first = cache("Player", game());
    let second = cache(
        "Other",
        Game {
            id: 0,
            achievements: vec![],
            ..game()
        },
    );
    // Same timestamp on both caches, as a restored backup might have.
    let when = std::fs::metadata(&first).unwrap().modified().unwrap();
    std::fs::File::options()
        .write(true)
        .open(&second)
        .unwrap()
        .set_modified(when)
        .unwrap();
    let mut readiness = Readiness::default();
    assert!(readiness.scan(root.path()).contains(&rom));
    std::fs::write(
        root.path().join("Config/retroachievements.toml"),
        "enabled = true\nusername = 'Other'\ntoken = 'fixture-token'\n",
    )
    .unwrap();
    assert!(readiness.scan(root.path()).is_empty());
}

#[test]
fn indexing_a_renamed_rom_is_announced_while_https_is_stalled() {
    struct Stalled {
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl network::Transport for Stalled {
        fn call(&mut self, _: &[(&str, String)]) -> Result<Value, network::Failure> {
            let _ = self.entered.send(());
            let _ = self.release.lock().unwrap().recv();
            Err(network::Failure::Network)
        }
    }
    let root = configured();
    let old = root.path().join("Games/GBA/Old name.gba");
    std::fs::write(&old, b"renamed while stalled").unwrap();
    let store = Store::open(root.path(), "Player").unwrap();
    let hash = library::hash(&store.dir, &old).unwrap();
    storage::write(
        &store.dir.join("auth.json"),
        &Auth {
            username: "Player".into(),
            token: "fixture-token".into(),
        },
    )
    .unwrap();
    storage::write(
        &store.dir.join(format!("{hash}.json")),
        &(game(), std::collections::BTreeSet::<u32>::new()),
    )
    .unwrap();
    let new = root.path().join("Games/GBA/New name.gba");
    std::fs::rename(&old, &new).unwrap();
    let (release, stalled) = mpsc::channel();
    let (entered, stuck) = mpsc::channel();
    let service = Service::start_with(
        root.path().into(),
        Stalled {
            entered,
            release: Mutex::new(stalled),
        },
    );
    // HTTPS is now stuck logging in, so it cannot see the load below.
    stuck.recv_timeout(Duration::from_secs(5)).unwrap();
    let before = service.cache_generation();
    let mut core = service.wrap(Box::<TestCore>::default());
    core.load(&new).unwrap();
    // Ejected before HTTPS wakes up: only the evaluator ever sees this load.
    drop(core);
    wait_for(|| service.cache_generation() != before);
    assert!(Readiness::default().scan(root.path()).contains(&new));
    drop(release);
}

// slotPlus: the costs kept off boot and game loading.


#[test]
fn no_ram_is_copied_until_the_game_is_being_tracked() {
    let (controls, _controls_rx) = mpsc::channel();
    let (frames, frames_rx) = mpsc::sync_channel(QUEUE_SIZE);
    let (_, notices) = mpsc::channel();
    let tracking = Arc::new(AtomicU64::new(u64::MAX));
    let service = Service {
        controls,
        frames,
        notices,
        enabled: Arc::new(AtomicBool::new(true)),
        current: Arc::new(AtomicU64::new(0)),
        tracking: tracking.clone(),
        flushing: AtomicBool::new(false),
        flushed: Arc::new(AtomicBool::new(false)),
        status: Arc::new(Status::default()),
    };
    let mut core = service.wrap(Box::<TestCore>::default());
    core.run_frame(ButtonMask(0));
    assert!(
        frames_rx.try_recv().is_err(),
        "copied RAM for an untracked game"
    );
    tracking.store(0, Ordering::Release);
    core.run_frame(ButtonMask(0));
    assert!(
        frames_rx.try_recv().is_ok(),
        "did not copy RAM for the tracked game"
    );
}

#[test]
fn the_library_index_reaches_the_card_on_flush() {
    let root = tempfile::tempdir().unwrap();
    let rom = root.path().join("game.gb");
    std::fs::write(&rom, [1u8; 4096]).unwrap();
    let hash = library::hash(root.path(), &rom).unwrap();
    assert_eq!(hash, library::content_hash(&rom).unwrap());
    library::flush();
    let written = std::fs::read_to_string(root.path().join("library.json")).unwrap();
    assert!(written.contains(&hash));
}


#[test]
fn an_encore_unlock_is_shown_but_never_recorded_again() {
    let root = tempfile::tempdir().unwrap();
    let store = Arc::new(Mutex::new(Store::open(root.path(), "Player").unwrap()));
    let mut runtime = runtime::Runtime::new().unwrap();
    let a = achievement("0xH000000=1");
    runtime.activate(a.id, &a.definition);
    let mut playing = Some(Playing {
        generation: 1,
        hash: "hash".into(),
        runtime,
        achievements: [(7, a)].into(),
        encore: [7].into(),
        previous: None,
        warned_gap: false,
    });
    let (notices, received) = mpsc::channel();
    let mut unsaved = BTreeMap::new();
    evaluate(frame(1, 0, 0), &mut playing, &store, &notices, &mut unsaved);
    evaluate(frame(2, 0, 1), &mut playing, &store, &notices, &mut unsaved);
    assert!(received.try_iter().any(|n| n.kind == NoticeKind::Earned));
    assert!(unsaved.is_empty());
    assert!(
        store.lock().unwrap().unlocks.is_empty(),
        "an encore unlock was recorded"
    );
}


fn tracked(service: &Service) -> bool {
    service.tracking.load(Ordering::Acquire) != 0
}

#[test]
fn a_game_says_ready_once_its_cached_set_is_active() {
    // Nothing cached: nothing to say, and nothing fetched while it plays.
    let root = configured();
    let rom = root.path().join("Games/GBA/Test.gba");
    std::fs::write(&rom, b"fixture ROM").unwrap();
    let (calls, requests) = mpsc::channel();
    let service = Service::start_with(root.path().into(), Server { calls });
    wait_for(|| service.sync_status() == SyncStatus::Ready);
    let _ = requests.try_iter().count();
    let mut core = service.wrap(Box::<TestCore>::default());
    core.load(&rom).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert!(!tracked(&service));
    assert!(!service.take_set_ready());
    assert_eq!(requests.try_iter().count(), 0, "a game in play never calls out");
    drop(core);

    // Cached by a scan: ready, said once.
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Done);
    let mut core = service.wrap(Box::<TestCore>::default());
    core.load(&rom).unwrap();
    wait_for(|| tracked(&service));
    wait_for(|| service.take_set_ready());
    assert!(!service.take_set_ready());
    drop(core);
}

fn notice(id: u32, title: &str) -> Achievement {
    Achievement {
        id,
        title: title.into(),
        badge: "00000".into(),
        description: "A notice from RetroAchievements".into(),
        points: 0,
        flags: 3,
        definition: "0=1".into(),
    }
}

#[test]
fn retroachievements_notices_and_stand_in_games_are_not_sets() {
    // A real set with the "Unknown Emulator" notice still counts...
    let mut real = game();
    real.achievements.push(notice(101000001, "Warning: Unknown Emulator"));
    assert!(scan::has_set(&real));
    assert_eq!(real.earnable().map(|a| a.id).collect::<Vec<_>>(), [7]);
    // ...one with only unofficial achievements beside the notice doesn't (Tetris today)...
    let mut unofficial = real.clone();
    unofficial.achievements[0].flags = 5;
    assert!(!scan::has_set(&unofficial));
    // ...nor does the stand-in RetroAchievements sends for a dump it doesn't support.
    let stand_in = Game {
        id: 1100002506,
        title: "Unsupported Game Version (TwinBee Da!!)".into(),
        achievements: vec![notice(101000001, "Unsupported Game Version")],
        ..game()
    };
    assert!(stand_in.stand_in() && !scan::has_set(&stand_in));
}

#[test]
fn a_trophy_judged_by_older_rules_is_judged_again_from_the_cached_set() {
    let root = configured();
    let rom = root.path().join("Games/GBA/TwinBee.gba");
    std::fs::write(&rom, b"twinbee").unwrap();
    let store = Store::open(root.path(), "Player").unwrap();
    let hash = library::hash(&store.dir, &rom).unwrap();
    let stand_in = Game {
        id: 1100002506,
        title: "Unsupported Game Version (TwinBee Da!!)".into(),
        achievements: vec![notice(101000001, "Unsupported Game Version")],
        ..game()
    };
    storage::write(
        &store.dir.join(format!("{hash}.json")),
        &(&stand_in, std::collections::BTreeSet::<u32>::new()),
    )
    .unwrap();
    // As the 2026-11-07 scan left it: a trophy, under the old rules (2).
    library::set_verdict(&store.dir, &rom, true, 2);
    assert!(library::with_sets(&store.dir).is_empty());
    assert_eq!(library::verdict(&store.dir, &rom), Some((false, 3)));
}

#[test]
fn a_played_game_is_reported_from_the_shelf_afterwards() {
    let root = configured();
    let rom = root.path().join("Games/GBA/Test.gba");
    std::fs::write(&rom, b"fixture ROM").unwrap();
    let (calls, requests) = mpsc::channel();
    let service = Service::start_with(root.path().into(), Server { calls });
    wait_for(|| service.enabled.load(Ordering::Acquire));
    service.start_scan();
    wait_for(|| service.scan_report().state == ScanState::Done);
    wait_for(|| service.sync_status() == SyncStatus::Ready);
    let _ = requests.try_iter().count();
    let mut core = service.wrap(Box::<TestCore>::default());
    core.load(&rom).unwrap();
    wait_for(|| tracked(&service));
    let dir = Store::open(root.path(), "Player").unwrap().dir;
    assert_eq!(played::queued(&dir).len(), 1);
    std::thread::sleep(Duration::from_millis(300));
    assert!(!requests.try_iter().any(|r| r == "startsession"), "not while it plays");
    drop(core);
    wait_for(|| played::queued(&dir).is_empty());
    assert!(requests.try_iter().any(|r| r == "startsession"));
}

#[test]
fn a_lynx_game_is_hashed_without_its_header_as_retroachievements_does() {
    let d = tempfile::tempdir().unwrap();
    let body = b"the game itself".repeat(100);
    let mut lnx = b"LYNX\0".to_vec();
    lnx.resize(64, 7);
    lnx.extend_from_slice(&body);
    std::fs::write(d.path().join("Game.lnx"), &lnx).unwrap();
    std::fs::write(d.path().join("Game.lyx"), &body).unwrap();
    let want = format!("{:x}", md5::compute(&body));
    assert_eq!(library::content_hash(&d.path().join("Game.lnx")).unwrap(), want);
    assert_eq!(library::content_hash(&d.path().join("Game.lyx")).unwrap(), want);
    // Anything else is hashed whole, a 64-byte one included.
    std::fs::write(d.path().join("Tiny.ws"), [9u8; 64]).unwrap();
    assert_eq!(
        library::content_hash(&d.path().join("Tiny.ws")).unwrap(),
        format!("{:x}", md5::compute([9u8; 64]))
    );
}

#[test]
fn a_headered_hucard_is_hashed_without_its_header_as_retroachievements_does() {
    let d = tempfile::tempdir().unwrap();
    let body = vec![5u8; 0x2000 * 4];
    let mut headered = vec![1u8; 512];
    headered.extend_from_slice(&body);
    std::fs::write(d.path().join("Card.pce"), &headered).unwrap();
    std::fs::write(d.path().join("Plain.pce"), &body).unwrap();
    let want = format!("{:x}", md5::compute(&body));
    assert_eq!(library::content_hash(&d.path().join("Card.pce")).unwrap(), want);
    assert_eq!(library::content_hash(&d.path().join("Plain.pce")).unwrap(), want);
    // The size rule is only the PC Engine's: a Game Gear game that size is hashed whole.
    std::fs::write(d.path().join("Same.gg"), &headered).unwrap();
    assert_eq!(
        library::content_hash(&d.path().join("Same.gg")).unwrap(),
        format!("{:x}", md5::compute(&headered))
    );
}

#[test]
fn ultras_systems_are_scanned_after_the_lynx_and_known_by_their_consoles() {
    let root = configured();
    for (shelf, name) in [("TG16", "Hu.pce"), ("NGP", "Pocket.ngc"), ("GG", "Gear.gg"), ("Lynx", "Cat.lyx")] {
        let dir = root.path().join("Games").join(shelf);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), name).unwrap();
    }
    use slot_store::Platform;
    let systems: Vec<_> = library::roms(root.path()).into_iter().map(|(p, _)| p).collect();
    assert_eq!(systems, [Platform::Lynx, Platform::Gg, Platform::Ngp, Platform::Tg16]);
    let consoles: Vec<_> = systems.into_iter().map(library::console_of).collect();
    assert_eq!(consoles, [13, 15, 14, 8]);
    for console in [8, 14, 15] {
        assert!(Game { console, ..game() }.supported_console(), "{console}");
    }
    // The DS isn't tracked yet.
    assert!(!Game { console: 18, ..game() }.supported_console());
}

#[test]
fn wonderswan_and_lynx_shelves_are_scanned_after_the_game_boys() {
    let root = configured();
    for (shelf, name) in [("WS", "Swan.wsc"), ("Lynx", "Cat.lnx"), ("GB", "Boy.gb")] {
        let dir = root.path().join("Games").join(shelf);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), name).unwrap();
    }
    let systems: Vec<_> = library::roms(root.path()).into_iter().map(|(p, _)| p).collect();
    use slot_store::Platform;
    assert_eq!(systems, [Platform::Gb, Platform::Ws, Platform::Lynx]);
    let lynx = Game { console: 13, ..game() };
    let swan = Game { console: 53, ..game() };
    assert!(scan::has_set(&lynx) && scan::has_set(&swan));
}
