// crates/qbzd/src/tui/app/nav.rs — the pure navigation-intent layer: mapping
// keystrokes/credential-presence to intents, independent of the `App` state
// machine that executes them (`state_nav.rs`, `keys.rs`).

use std::path::PathBuf;

use super::messages::Screen;
use crate::tui::strings as s;

/// Construct the path to the OAuth token file in the config root.
pub(super) fn cred_file_path(config_root: &PathBuf) -> PathBuf {
    config_root.join(".qbz-oauth-token")
}

/// Determine the startup focus (03 §2.2, re-shelled for FB3). The landing
/// SECTION is always Account (there is no menu to land on any more); only the
/// focus differs:
/// - No credential file → focus the CONTENT (Account is ready to log in).
/// - Credential file present → focus the NAV (the operator picks where to go).
///
/// The decision is based on credential-file presence, not live daemon auth state.
pub(super) fn initial_focus(cred_file_present: bool) -> Focus {
    if cred_file_present {
        Focus::Nav
    } else {
        Focus::Content
    }
}

/// Which pane holds the keyboard focus. The frames shell has two: the persistent
/// left navigation sidebar and the right content frame (FB3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Nav,
    Content,
}

/// The 0-based index of a section in `SCREENS` (sidebar row / number key).
pub(super) fn section_index(screen: Screen) -> usize {
    super::messages::SCREENS.iter().position(|s| *s == screen).unwrap_or(0)
}

/// The full section title for the breadcrumb (the sidebar uses short labels).
pub(super) fn section_title(screen: Screen) -> &'static str {
    match screen {
        Screen::Account => s::ACCOUNT_TITLE,
        Screen::Audio => s::AUDIO_TITLE,
        Screen::Playback => s::PLAYBACK_TITLE,
        Screen::Network => s::NETWORK_TITLE,
        Screen::Bundle => s::BUNDLE_TITLE,
        Screen::Wizard => s::WIZARD_TITLE,
        Screen::Scrobbler => s::SCROBBLER_TITLE,
    }
}

/// Breadcrumb node composition (max 2 levels, FB3). Pure so it can be pinned:
/// - not editing → (`Setup`, section)   — dim prefix, accent current.
/// - editing a field → (section, field) — the field label is the current node.
/// Modals/pickers are the third level (overlays); they do NOT change the crumb,
/// so the caller passes `None` for them (see each screen's `editing_label`).
pub(super) fn breadcrumb_nodes<'a>(section: &'a str, editing_field: Option<&'a str>) -> (&'a str, &'a str) {
    match editing_field {
        Some(field) => (section, field),
        None => (s::BREADCRUMB_ROOT, section),
    }
}

/// Whether a sidebar row shows the dirty `*`. Only the ACTIVE section can be
/// dirty — leaving a section is gated by the Save/Discard/Stay modal, so every
/// other section is clean by construction. Pure for the mapping test.
pub(super) fn sidebar_dirty_marker(row: Screen, active: Screen, active_dirty: bool) -> bool {
    row == active && active_dirty
}

// NavIntent + classify_key (the keystroke → intent table) live in
// `nav_classify.rs`.
