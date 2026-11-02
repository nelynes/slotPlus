//! slotPlus: the Settings menu's view of `Config/retroachievements.toml`, and the edits it
//! makes. The file stays the one place the account lives, so editing it by hand still works;
//! the menu rewrites only the lines it changes and keeps everything else, comments too.
use std::path::{Path, PathBuf};

use crate::storage::Config;

/// What the menu shows. The password itself never leaves the file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub enabled: bool,
    pub username: String,
    pub has_password: bool,
    /// An emulator login token is set in the file.
    pub has_token: bool,
    pub encore: bool,
}

impl Settings {
    /// The card's settings, or the defaults when the file is missing or can't be read.
    pub fn read(root: &Path) -> Settings {
        let config = Config::read(&path(root)).unwrap_or_default();
        Settings {
            enabled: config.enabled,
            username: config.username.trim().to_string(),
            has_password: !config.password.is_empty(),
            has_token: !config.token.is_empty(),
            encore: config.encore,
        }
    }

    /// An account is set up, so achievements are on.
    pub fn signed_in(&self) -> bool {
        self.enabled && !self.username.is_empty() && (self.has_password || self.has_token)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Username(String),
    Password(String),
    Encore(bool),
}

fn path(root: &Path) -> PathBuf {
    root.join("Config/retroachievements.toml")
}

/// Write one change into the file, creating it if needed. A new username or password from
/// the menu also turns achievements on and drops a token from the file, which belongs to
/// whoever set it by hand and would otherwise win over the password just typed.
pub fn save(root: &Path, change: Change) -> std::io::Result<()> {
    let path = path(root);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    match change {
        Change::Username(name) => {
            set(&mut lines, "username", &quoted(name.trim()));
            set(&mut lines, "enabled", "true");
            remove(&mut lines, "token");
        }
        Change::Password(password) => {
            set(&mut lines, "password", &quoted(&password));
            set(&mut lines, "enabled", "true");
            remove(&mut lines, "token");
        }
        Change::Encore(on) => set(&mut lines, "encore", if on { "true" } else { "false" }),
    }
    let mut out = lines.join("\n");
    out.push('\n');
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    slot_store::atomic_write(&path, out.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// A TOML basic string.
fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The key a line sets, if it sets one (comments and blank lines set none).
fn key_of(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if line.starts_with('#') {
        return None;
    }
    let (key, _) = line.split_once('=')?;
    Some(key.trim())
}

fn set(lines: &mut Vec<String>, key: &str, value: &str) {
    let line = format!("{key} = {value}");
    match lines.iter().position(|l| key_of(l) == Some(key)) {
        Some(i) => lines[i] = line,
        None => {
            // Top-level keys go before any table, of which there are none today.
            let at = lines
                .iter()
                .position(|l| l.trim_start().starts_with('['))
                .unwrap_or(lines.len());
            lines.insert(at, line);
        }
    }
}

fn remove(lines: &mut Vec<String>, key: &str) {
    lines.retain(|l| key_of(l) != Some(key));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("Config")).unwrap();
        d
    }

    #[test]
    fn the_menu_makes_a_file_that_signs_in() {
        let d = card();
        assert!(!Settings::read(d.path()).signed_in());
        save(d.path(), Change::Username("  Player ".into())).unwrap();
        save(d.path(), Change::Password("a \"tricky\" \\ pass".into())).unwrap();
        let s = Settings::read(d.path());
        assert!(s.signed_in(), "{s:?}");
        assert_eq!(s.username, "Player");
        let config = Config::read(&path(d.path())).unwrap();
        assert_eq!(config.password, "a \"tricky\" \\ pass");
    }

    #[test]
    fn edits_keep_the_rest_of_the_file() {
        let d = card();
        std::fs::write(
            path(d.path()),
            "# my account\nenabled = false\nusername = \"old\"\ntoken = \"t\"\n# encore = true\n",
        )
        .unwrap();
        save(d.path(), Change::Encore(true)).unwrap();
        let text = std::fs::read_to_string(path(d.path())).unwrap();
        assert!(text.contains("# my account") && text.contains("token = \"t\""), "{text}");
        assert!(Settings::read(d.path()).encore);
        assert!(!Settings::read(d.path()).enabled, "Encore alone doesn't turn it on");
        save(d.path(), Change::Password("pw".into())).unwrap();
        let text = std::fs::read_to_string(path(d.path())).unwrap();
        assert!(!text.contains("token ="), "a typed password replaces the token: {text}");
        assert!(text.contains("# encore = true"), "comments stay: {text}");
        let s = Settings::read(d.path());
        assert!(s.enabled && s.has_password && !s.has_token);
        assert_eq!(s.username, "old");
    }
}
