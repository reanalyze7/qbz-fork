// crates/qbzd/src/cli/settings/verbs.rs — the `settings show`/`set` CLI
// entry points (the `config path`/`config show` verbs live in
// `verbs_config.rs`).

use crate::paths::ProfileRoots;

use super::keys::ApplyClass;
use super::nudge::nudge;
use super::store::read_all;
use super::write::{write_one, SetError};

/// `qbzd settings show [--json]` (⬇). `--json`: `{"audio.backend": "alsa", ...}`
/// — every value the plain string `settings set` would accept back (module
/// doc). Exit: 0 · 1 (a store failed to open/read).
pub fn show(json: bool, roots: &ProfileRoots) -> i32 {
    let values = match read_all(roots) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    if json {
        let mut map = serde_json::Map::with_capacity(values.len());
        for (k, v) in &values {
            map.insert((*k).to_string(), serde_json::Value::String(v.clone()));
        }
        println!("{}", serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_default());
    } else {
        let width = values.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        for (k, v) in &values {
            println!("{k:width$} = {v}");
        }
    }
    0
}

/// `qbzd settings set <KEY> <VALUE>` (⬇). Unknown key → exit 2 listing valid
/// keys; invalid value for a known key → exit 2 naming the valid values;
/// the key/value classified and parsed fine but the backing store failed to
/// open or write (disk/permissions) → exit 1 (02 §1.3: 2 is USAGE-only —
/// see [`SetError`]). Writes always succeed locally before any daemon
/// contact is attempted (§2.4 daemon-down capable); daemon-down prints
/// `changes apply when the daemon starts` (this task's brief, verbatim)
/// instead of failing.
pub fn set(roots: &ProfileRoots, key: &str, value: &str) -> i32 {
    let class = match write_one(roots, key, value) {
        Ok(c) => c,
        Err(SetError::Usage(e)) => {
            eprintln!("error: {}", e.trim_end());
            return 2;
        }
        Err(SetError::Io(e)) => {
            eprintln!("error: {}", e.trim_end());
            return 1;
        }
    };
    if nudge(roots) {
        let hint = match class {
            ApplyClass::Reinit => " (daemon reinitialized the output device)",
            ApplyClass::Reload | ApplyClass::None => "",
        };
        println!("{key} = {value}{hint}");
    } else {
        println!("{key} = {value}");
        println!("changes apply when the daemon starts");
    }
    0
}
