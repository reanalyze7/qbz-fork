use crate::state::{AuthState, LatchedErrors};

use super::{AudioStatus, AuthStatus, NetworkStatus, PlaybackStatus, StatusDoc};

/// A NeedsAuth [`StatusDoc`] built directly (no live runtime), matching the
/// 02 §3.3.3 NeedsAuth fragment — the serde-shape contract these tests pin.
fn needs_auth_doc() -> StatusDoc {
    StatusDoc {
        version: "2.1.0".into(),
        api_version: crate::API_VERSION,
        uptime_secs: 261_360,
        data_root: "/home/pi/.local/share/qbzd".into(),
        driver_tick_age_ms: Some(210),
        auth: AuthStatus {
            state: AuthState::NeedsAuth,
            user_id: None,
            subscription: None,
        },
        audio: AudioStatus {
            backend: Some("alsa".into()),
            configured_device: None,
            device_present: true,
            device_open: false,
            bit_perfect: None,
            sample_rate: None,
            bit_depth: None,
        },
        playback: PlaybackStatus {
            state: "stopped".into(),
            track_id: None,
            title: None,
            artist: None,
            position: None,
            duration: None,
            volume: 0.0,
            muted: false,
            queue_len: 0,
        },
        network: NetworkStatus { online: true },
        last_errors: LatchedErrors {
            stream: None,
            auth: Some("token rejected by Qobuz (401) — cleared".into()),
            transport: None,
        },
    }
}

#[test]
fn status_doc_mirrors_the_top_level_contract_keys() {
    // 02-cli-and-api.md §3.3.3 top-level keys, exactly.
    let json = serde_json::to_value(needs_auth_doc()).unwrap();
    let obj = json.as_object().unwrap();
    for key in [
        "version",
        "api_version",
        "uptime_secs",
        "data_root",
        "driver_tick_age_ms",
        "auth",
        "audio",
        "playback",
        "network",
        "last_errors",
    ] {
        assert!(obj.contains_key(key), "missing top-level key: {key}");
    }
}

#[test]
fn needs_auth_fragment_matches_spec_example() {
    // 02 §3.3.3 NeedsAuth example fragment + auth.state serde string.
    let doc = needs_auth_doc();
    assert_eq!(doc.auth.state, AuthState::NeedsAuth);
    assert!(doc.auth.user_id.is_none());
    assert!(doc.auth.subscription.is_none());
    assert_eq!(
        doc.last_errors.auth.as_deref(),
        Some("token rejected by Qobuz (401) — cleared")
    );
    let json = serde_json::to_value(&doc.auth).unwrap();
    assert_eq!(json["state"], "needs_auth");
}

#[test]
fn status_doc_playback_volume_serializes_canonically() {
    // Pins the `status()` pointer-overwrite: `to_value(&doc)` widens the
    // f32 `playback.volume` via `Number::from_f32`; the fix must land
    // `0.8` on the wire, never `0.800000011920929`.
    let mut doc = needs_auth_doc();
    doc.playback.volume = 0.8f32;
    let mut value = serde_json::to_value(&doc).unwrap();
    if let Some(vol) = value.pointer_mut("/playback/volume") {
        *vol = crate::api::canon_volume(doc.playback.volume);
    }
    let rendered = serde_json::to_string(&value).unwrap();
    assert!(rendered.contains("\"volume\":0.8"), "got: {rendered}");
    assert!(!rendered.contains("0.80000"), "got: {rendered}");
}

#[test]
fn audio_block_serializes_the_documented_keys() {
    // 02 §3.3.3 audio object — the shape the live assembler fills.
    let json = serde_json::to_value(needs_auth_doc()).unwrap();
    let audio = json["audio"].as_object().unwrap();
    for key in [
        "backend",
        "configured_device",
        "device_present",
        "device_open",
        "bit_perfect",
        "sample_rate",
        "bit_depth",
    ] {
        assert!(audio.contains_key(key), "missing audio key: {key}");
    }
}
