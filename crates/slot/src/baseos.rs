use std::path::Path;

const RELEASE: &str = "/etc/baseos-release";

pub fn var(key: &str) -> Option<String> {
    var_in(key, std::env::var(key).ok(), Path::new(RELEASE))
}

pub fn var_in(key: &str, env: Option<String>, release: &Path) -> Option<String> {
    env.or_else(|| lookup(&std::fs::read_to_string(release).ok()?, key))
}

pub fn lookup(release: &str, key: &str) -> Option<String> {
    release.lines().find_map(|line| {
        let (k, v) = line.trim().split_once('=')?;
        (k == key).then(|| v.trim_matches('"').to_string())
    })
}
