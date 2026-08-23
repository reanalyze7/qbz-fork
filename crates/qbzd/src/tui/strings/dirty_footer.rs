// ============================ dirty-save / quit ============================

pub const DIRTY_TITLE: &str = "Unsaved changes";
pub const DIRTY_BODY: &str = "This screen has unsaved edits.";
pub const DIRTY_HINT: &str = "s save · d discard · Esc stay";

// ============================ footer (daemon state) ============================

pub const FOOTER_UNREACHABLE: &str = "daemon: not reachable";
pub const FOOTER_RUNNING: &str = "daemon: running";
pub const FOOTER_NEEDS_AUTH: &str = "not signed in";
/// Appended to a save result when the daemon is down (03 §2.3, error-voice).
pub const APPLIES_ON_START: &str =
    "changes apply when the daemon starts — systemctl --user status qbzd";
