//! Surface audio-stream failures as a toast (#508/#534/#500).

use super::super::Runtime;
use crate::AppWindow;

/// The player records a user-readable message when a stream fails to
/// open, but the drain lived in Tauri's polling loop and was never
/// ported — ALSA Direct failures left the bar frozen at 0:00 with
/// no explanation. `take_stream_error_message` drains exactly once
/// per recorded error, so each failed attempt toasts once. Inside a
/// Flatpak/Snap sandbox direct hw: access is blocked by design —
/// when the failure looks ALSA-shaped, say that instead of the raw
/// error.
pub(super) fn surface(runtime: &Runtime, weak: &slint::Weak<AppWindow>) {
    let Some(msg) = runtime.core().player().state.take_stream_error_message() else {
        return;
    };
    let sandboxed = !matches!(
        qbz_audio::health::detect_sandbox(),
        qbz_audio::health::Sandbox::None
    );
    let lower = msg.to_lowercase();
    let text = if sandboxed && (lower.contains("alsa") || lower.contains("hw:")) {
        qbz_i18n::t(
            "Direct ALSA device access is blocked inside Flatpak/Snap — switch the audio backend to PipeWire or System Default",
        )
    } else {
        format!("{}: {}", qbz_i18n::t("Audio output error"), msg)
    };
    crate::toast::error_weak(weak, text);
}
