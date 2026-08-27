// ==================== settings bundle (04-settings-portability.md) ====================
// Verbatim §3 / §4.1 / §5.3 / §5.6 copy — "modulo interpolated values" (§1.4).

fn basename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Secret-bearing export warning (04 §3, verbatim). The last two lines teach the
/// import step. `path` is the full written path; the basename is interpolated
/// into the import command.
pub fn bundle_secret_warning(path: &str) -> String {
    let base = basename(path);
    format!(
        "WARNING: this bundle contains your Qobuz session token and integration tokens.
Anyone who can read this file can use your accounts.
  - the file was written with mode 0600 — keep it that way
  - move it directly (scp/USB) and delete it after importing
  - it is NOT encrypted: do not mail it, cloud-sync it, or commit it anywhere
wrote {path} (0600, includes secrets)
next: copy it to the daemon box and run:  qbzd settings import {base} --include-auth"
    )
}

/// Non-secret export success (04 §4.1): the path + the same "next:" hint.
pub fn bundle_export_success(path: &str) -> String {
    let base = basename(path);
    format!(
        "wrote {path} (0600)
next: copy it to the daemon box and run:  qbzd settings import {base}"
    )
}

/// Missing desktop profile on `--from desktop` (04 §4.1, verbatim).
pub fn bundle_no_desktop_profile() -> String {
    "error: no desktop profile found at ~/.local/share/qbz
  is desktop Qoqobuz installed on this box? To export this daemon's settings instead:
  qbzd settings export            (daemon profile is the default)"
        .to_string()
}

/// IV1 desktop-token decryption failure on `--from desktop --include-auth`
/// (04 §4.1, verbatim) — the portal secret is bound to the desktop session.
pub fn bundle_token_decrypt_failed() -> String {
    "error: could not decrypt the desktop Qobuz token
  the desktop token is protected with a session key only available inside your
  desktop session
  → run this command from a terminal inside your desktop session, or
  → skip --include-auth and log the daemon in directly:  qbzd login"
        .to_string()
}

/// Version-skew rejection, bundle newer than importer (04 §5.6, verbatim).
pub fn bundle_version_too_new(bundle: i64, supported: i64) -> String {
    format!(
        "error: this bundle is schema v{bundle}; this qbzd understands up to v{supported}
  the bundle came from a newer Qoqobuz — update qbzd on this box, then retry
  (self-built? rebuild from the current tag)"
    )
}

/// Bundle auth token rejected by Qobuz on import (04 §5.3 step 5, verbatim).
pub fn bundle_token_rejected() -> String {
    "error: the Qobuz token in this bundle was rejected — re-export with a fresh desktop login, or run: qbzd login"
        .to_string()
}
