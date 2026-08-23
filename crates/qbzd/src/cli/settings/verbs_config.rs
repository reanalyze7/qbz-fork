// crates/qbzd/src/cli/settings/verbs_config.rs — the `config path`/`config
// show` CLI entry points (process concerns only; engine settings live in
// `verbs.rs`'s `show`/`set`).

use std::path::{Path, PathBuf};

use crate::paths::ProfileRoots;

/// `qbzd config path` (⬇). Process roots + the credential file + the
/// (conventional, not necessarily yet installed — T14) systemd user unit
/// path. No `--json` (02 §2.2 gives none for this subverb).
pub fn config_path(roots: &ProfileRoots) -> i32 {
    println!("config : {}", roots.config.display());
    println!("data   : {}", roots.data.display());
    println!("cache  : {}", roots.cache.display());
    println!("cred   : {}", roots.config.join(".qbz-oauth-token").display());
    println!("unit   : {}", unit_path().display());
    0
}

fn unit_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("systemd/user/qbzd.service")
}

/// `qbzd config show [--json]` (⬇) — effective `qbzd.toml`, process concerns
/// only (01-architecture.md §10.1; engine settings live in `settings show`).
/// Keys the file doesn't set are annotated `(default)` in human mode.
pub fn config_show(json: bool, roots: &ProfileRoots) -> i32 {
    let path = roots.config.join("qbzd.toml");
    let (cfg, _warns) = match crate::config::QbzdConfig::load(&path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string(&cfg).unwrap_or_default());
        return 0;
    }
    let present = present_keys(&path);
    let line = |label: &str, dotted: &str, value: String| {
        let marker = if present.contains(dotted) { "" } else { " (default)" };
        println!("{label:<24}= {value}{marker}");
    };
    line("config_version", "config_version", cfg.config_version.to_string());
    line(
        "data_root",
        "data_root",
        cfg.data_root.clone().unwrap_or_else(|| "(auto)".to_string()),
    );
    line("server.bind", "server.bind", cfg.server.bind.clone());
    line("server.port", "server.port", cfg.server.port.to_string());
    line(
        "server.token",
        "server.token",
        match &cfg.server.token {
            Some(t) if !t.trim().is_empty() => "(set)".to_string(),
            _ => "(empty = open)".to_string(),
        },
    );
    line("log.level", "log.level", cfg.log.level.clone());
    line("mpris.enabled", "mpris.enabled", cfg.mpris.enabled.to_string());
    0
}

/// Which dotted config keys the on-disk `qbzd.toml` actually sets (vs.
/// defaulted) — a missing/unreadable file means every key is `(default)`.
pub(super) fn present_keys(path: &Path) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let Ok(text) = std::fs::read_to_string(path) else {
        return out;
    };
    let Ok(value) = toml::from_str::<toml::Value>(&text) else {
        return out;
    };
    if let toml::Value::Table(top) = &value {
        for (k, v) in top {
            if let toml::Value::Table(inner) = v {
                for ik in inner.keys() {
                    out.insert(format!("{k}.{ik}"));
                }
            } else {
                out.insert(k.clone());
            }
        }
    }
    out
}
