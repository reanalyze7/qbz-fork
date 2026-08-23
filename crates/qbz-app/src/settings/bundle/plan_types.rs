use serde_json::Value;

/// One line of the three-bucket summary (§5.4). `old` is set only for *adapted*
/// lines (which render `old -> new`); *skipped* lines carry the reason in `why`
/// and leave `new` empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanLine {
    pub key: String,
    pub old: Option<String>,
    pub new: String,
    pub why: String,
}

/// A machine device that needs an interactive re-pick (§5.3 step 4, TTY path).
#[derive(Debug, Clone)]
pub struct DevicePick {
    pub wanted: String,
    /// The backend the options were enumerated for — the §5.4 prompt names it
    /// ("Available on Alsa:").
    pub backend: String,
    pub options: Vec<(String, String)>,
}

/// The operator's answer to a [`DevicePick`] (from the CLI/TUI). Fed back into
/// [`super::replan_with_device`].
#[derive(Debug, Clone)]
pub enum DeviceChoice {
    SystemDefault,
    Device { id: String, label: String },
}

/// The classified plan (`plan` output): the three display buckets, plus the
/// execution list (`writes`), an optional device re-pick, and the decrypted
/// auth token to validate before any write.
#[derive(Debug, Clone, Default)]
pub struct ImportPlan {
    pub applied: Vec<PlanLine>,
    pub adapted: Vec<PlanLine>,
    pub skipped: Vec<PlanLine>,
    pub device_pick: Option<DevicePick>,
    /// Present only when `--include-auth` AND the bundle carries `auth`. The CLI
    /// validates it via `validate_token` BEFORE calling `apply` (§5.3 step 5).
    pub auth_token: Option<String>,
    /// Cross-check uid from the bundle (`auth.user_id`); the authoritative uid
    /// is the validated login's (§5.7).
    pub bundle_user_id: Option<u64>,
    /// The typed write actions backing `applied` + `adapted` (display strings
    /// are lossy — e.g. `(auto)` — so execution rides raw JSON values here).
    pub writes: Vec<(String, Value)>,
    /// True when a routing-critical field (backend/device/exclusive) changed —
    /// the CLI's reload line owns the honesty note (§5.3 step 7).
    pub routing_critical_changed: bool,
}

/// The outcome of `apply` (§5.3 step 8): bucket counts + per-domain results so
/// a mid-apply I/O failure is reported honestly.
#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub applied: usize,
    pub adapted: usize,
    pub skipped: usize,
    pub per_domain: Vec<(String, Result<(), String>)>,
}
