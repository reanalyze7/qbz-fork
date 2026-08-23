// crates/qbzd/src/cli/copy/ — normative CLI copy for the auth verbs.
//
// Strings are reproduced verbatim from 02-cli-and-api.md §2.2, "modulo
// interpolated values" (§1.4): the ephemeral listener port is substituted into
// the `ssh -L` forward hint so it is actionable on a headless box, and the
// success line interpolates the validated session's email / plan / user id.
mod auth;
mod bundle;
mod daemon_errors;

pub use auth::{login_browser_open_failed, login_ssh_detected, login_success, login_timeout, logout_success};
pub use bundle::{
    bundle_export_success, bundle_no_desktop_profile, bundle_secret_warning,
    bundle_token_decrypt_failed, bundle_token_rejected, bundle_version_too_new,
};
pub use daemon_errors::{
    api_version_skew, daemon_down, daemon_up_needs_auth, foreign_qbzd, lan_posture_note,
    linger_off, port_in_use, seek_unsupported_dsd, version_skew, volume_fixed_dsd,
};
