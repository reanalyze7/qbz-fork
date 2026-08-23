// ============================ T11: settings reload ============================
// `POST /api/settings/reload` (02-cli-and-api.md §3.3.17; `crate::api::settings
// ::reload` is the thin HTTP wrapper) re-reads every engine store and applies
// what changed: audio (routing-critical -> `Player::reinit_device`, the rest ->
// `Player::reload_settings`), the daemon's own streaming-quality cell (the
// driver's background auto-advance), the QConnect KV (device-name cache +
// connect/disconnect reconciliation), and finally the credential file (absent
// -> NeedsAuth; new -> session restore). Never re-reads `qbzd.toml` (§3.1.2 —
// process config is boot-only). Response = the post-reload `/api/status` body,
// composed by the caller — zero new shapes (03-setup-tui.md §4.3: the
// reinit/reload narrative is composed CLIENT-side from the CLI's own copy of
// the Apply-ladder classification, never carried on the wire).
mod audio;
mod credential;

pub(crate) use audio::{audio_routing_changed, reload_audio, reload_quality};
pub(crate) use credential::{decide_credential_action, reload_credentials, CredentialAction};

/// The single entry point the HTTP route calls. Order matters only at the
/// margin (independent domains): audio/quality/qconnect-KV first, credentials
/// last, so a login/logout settles the auth state before QConnect decides
/// whether to (re)connect against it.
pub(crate) async fn reload(state: &crate::api::ApiState) {
    reload_audio(state);
    reload_quality(state);
    reload_credentials(&state.runtime, &state.shared, &state.roots).await;
}
