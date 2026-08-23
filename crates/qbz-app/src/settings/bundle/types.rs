use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The bundle schema version this importer implements and this exporter writes
/// (04 §1). Hard-gated on import (`plan` step 2, §5.6). v1 is the floor.
pub const SCHEMA_VERSION: i64 = 1;

/// One versioned JSON document (04 §1). The header fields are typed; every
/// settings domain rides in `domains` as raw JSON so the importer can classify
/// whatever is present (§1 corollary). Serializes FLAT — the domains sit at the
/// top level alongside the header, exactly like the §2.9 example.
#[derive(Debug, Clone, Serialize)]
pub struct Bundle {
    pub schema_version: i64,
    /// RFC 3339 UTC timestamp of export.
    pub created_at: String,
    pub source: BundleSource,
    #[serde(flatten)]
    pub domains: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleSource {
    pub app_version: String,
    /// `"desktop"` | `"daemon"` — which profile was read.
    pub profile: String,
    pub hostname: String,
}

/// A profile's config + data roots. For the daemon these are the daemon roots
/// (`~/.config/qbzd`, `~/.local/share/qbzd`); for the desktop the global roots
/// (`~/.config/qbz`, `~/.local/share/qbz`).
#[derive(Debug, Clone)]
pub struct ProfilePaths {
    pub config_root: PathBuf,
    pub data_root: PathBuf,
}

/// Where an export reads its settings from (04 §4.1).
pub enum ExportSource {
    /// The GLOBAL desktop stores at `~/.local/share/qbz` (read-only; the ONLY
    /// place desktop paths are legal — the per-user `users/<uid>/` copies are
    /// Tauri-era ghosts, never read).
    Desktop,
    /// The daemon's own profile roots.
    Daemon(ProfilePaths),
}

#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    pub include_auth: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    pub include_auth: bool,
    pub trust_dsd: bool,
    /// Repeatable `--remap OLD=NEW` prefix rewrites for `library_folders`.
    pub remap: Vec<(String, String)>,
    /// True when there is no interactive terminal — machine fields that would
    /// need a prompt fall to safe defaults instead of hanging (§5.3 step 4).
    pub non_tty: bool,
}

/// Injected snapshot of the local audio system so classification is testable
/// without hardware (`BackendManager::available_backends()` + the chosen
/// backend's device enumeration).
#[derive(Debug, Clone, Default)]
pub struct LiveSystem {
    pub backends: Vec<String>,
    /// `(id, label)` pairs for the chosen backend.
    pub devices: Vec<(String, String)>,
}
