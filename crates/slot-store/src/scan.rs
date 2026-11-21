use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::gba::header_title_code;
use crate::platform::Platform;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cart {
    pub platform: Platform,
    pub stem: String,
    pub rom: PathBuf,
    pub label: Option<PathBuf>,
    pub title: String,
    pub code: String,
    pub shell: Option<crate::ShellChoice>,
    pub tags: Vec<String>,
}

impl Cart {
    pub fn read_header(&mut self) {
        match self.platform.hardware() {
            Platform::Gba => {
                if let Some((title, code)) = header_title_code(&self.rom) {
                    self.title = title;
                    self.code = code;
                }
            }
            // Lynx and Game Gear files have no Game Boy header to read a title
            // from, so the file name stays the title.
            Platform::Lynx | Platform::Gg | Platform::Ngp | Platform::Tg16 | Platform::Nds => {}
            _ => {
                if let Some(title) = crate::gb::title(&self.rom) {
                    self.title = title;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

pub fn scan(root: &Path) -> Result<Vec<Cart>, StoreError> {
    scan_of(root, &Platform::ALL)
}

pub fn scan_platform(root: &Path, platform: Platform) -> Result<Vec<Cart>, StoreError> {
    scan_of(root, &[platform])
}

/// slotPlus: every cart on a platform's shelf, in the shelf's order. The same as
/// `scan_platform` except on the PokeDaisy shelf, which gathers its three systems' carts.
pub fn scan_shelf(root: &Path, platform: Platform) -> Result<Vec<Cart>, StoreError> {
    let members: Vec<Platform> = Platform::ALL
        .into_iter()
        .filter(|p| p.shelf() == platform.shelf())
        .collect();
    scan_of(root, &members)
}

fn scan_of(root: &Path, platforms: &[Platform]) -> Result<Vec<Cart>, StoreError> {
    let read = |file: &str| std::fs::read_to_string(root.join(file)).unwrap_or_default();
    let shells = crate::cart_shell::layered(
        &read(crate::CART_SHELL_FILE),
        &read(crate::LABELS_SHELL_FILE),
    );
    let mut carts = Vec::new();
    for platform in platforms.iter().copied() {
        let dir = root.join("Games").join(platform.dir_name());
        let entries = match std::fs::read_dir(&dir) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                eprintln!("slot: scan: {}: {e}", dir.display());
                continue;
            }
        };
        let labels_dir = root.join("Labels").join(platform.dir_name());
        let labels = listing(&labels_dir);
        // slotPlus: a PokeDaisy game takes its own system's label when the PokeDaisy folder has
        // none, so art already on the card shows.
        let system_dir = root.join("Labels").join(platform.hardware().dir_name());
        let system_labels = match platform.is_pokedaisy() {
            true => listing(&system_dir),
            false => HashMap::new(),
        };
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let rom = entry.path();
            if is_hidden(&rom) || !platform.accepts(&rom) || !is_file(&entry) {
                continue;
            }
            let Some(stem) = rom.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let png = format!("{stem}.png").to_lowercase();
            let label = labels
                .get(&png)
                .map(|name| labels_dir.join(name))
                .or_else(|| system_labels.get(&png).map(|name| system_dir.join(name)));
            carts.push(Cart {
                platform,
                stem: stem.to_string(),
                title: String::new(),
                code: String::new(),
                shell: shells.get(&crate::cart_shell::key(stem)).copied(),
                tags: Vec::new(),
                label,
                rom,
            });
        }
    }
    // slotPlus: the music cart, on the Game Boy Advance shelf whenever there's music.
    if platforms.contains(&Platform::Gba) && crate::music::has_music(root) {
        carts.push(crate::music::music_cart(root));
    }
    // By shelf, then by name: the PokeDaisy shelf's three systems' carts sort together.
    carts.sort_by_key(|c| {
        (
            c.platform.shelf() as u8,
            sort_key(&c.stem),
            c.platform as u8,
        )
    });
    tell_twins_apart(&mut carts);
    Ok(carts)
}

fn tell_twins_apart(carts: &mut [Cart]) {
    let mut twins: HashMap<(Platform, String), Vec<usize>> = HashMap::new();
    for (i, c) in carts.iter().enumerate() {
        twins
            .entry((c.platform, clean_label(&c.stem).to_lowercase()))
            .or_default()
            .push(i);
    }
    for group in twins.values().filter(|g| g.len() > 1) {
        let all: Vec<Vec<String>> = group.iter().map(|&i| label_tags(&carts[i].stem)).collect();
        for (&i, tags) in group.iter().zip(&all) {
            carts[i].tags = tags
                .iter()
                .filter(|t| !all.iter().all(|other| other.contains(t)))
                .cloned()
                .collect();
        }
    }
}

pub fn listing(dir: &Path) -> HashMap<String, String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return HashMap::new();
    };
    entries
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .map(|name| (name.to_lowercase(), name))
        .collect()
}

fn is_file(entry: &std::fs::DirEntry) -> bool {
    match entry.file_type() {
        Ok(t) if t.is_symlink() => entry.path().is_file(),
        Ok(t) => t.is_file(),
        Err(_) => false,
    }
}

pub fn sort_key(stem: &str) -> (u8, String) {
    (group_of(stem), stem.to_uppercase())
}

fn group_of(stem: &str) -> u8 {
    match stem.chars().find(|c| !c.is_whitespace()) {
        Some(c) if c.is_ascii_digit() => 0,
        Some(c) if c.is_alphabetic() => 1,
        _ => 2,
    }
}

pub fn initial(stem: &str) -> char {
    match group_of(stem) {
        1 => stem
            .chars()
            .find(|c| !c.is_whitespace())
            .and_then(|c| c.to_uppercase().next())
            .unwrap_or('#'),
        _ => '#',
    }
}

pub fn is_hidden(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.'))
}

pub fn clean_label(stem: &str) -> String {
    let mut bare = String::with_capacity(stem.len());
    let mut depth = 0u32;
    for ch in stem.chars() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => bare.push(ch),
            _ => {}
        }
    }

    let mut out = String::with_capacity(bare.len());
    for word in bare.split_whitespace().filter(|w| *w != "-") {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    if out.is_empty() {
        stem.to_string()
    } else {
        out
    }
}

pub fn label_tags(stem: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0u32;
    let mut cur = String::new();
    for ch in stem.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
                if depth == 1 {
                    cur.clear();
                    continue;
                }
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let t = cur.trim();
                    if !t.is_empty() {
                        out.push(t.to_string());
                    }
                    continue;
                }
            }
            _ => {}
        }
        if depth >= 1 {
            cur.push(ch);
        }
    }
    out
}
