use serde_json::Value;

use crate::api::canon_volume;

use super::queue_modes::repeat_str;

#[test]
fn repeat_str_matches_contract_lowercase() {
    assert_eq!(repeat_str(qbz_models::RepeatMode::Off), "off");
    assert_eq!(repeat_str(qbz_models::RepeatMode::All), "all");
    assert_eq!(repeat_str(qbz_models::RepeatMode::One), "one");
}

/// A minimal `PlaybackEvent` for the volume-serialization tests below —
/// same shape `player.get_playback_event()` returns, just hand-built so
/// the test does not need a live `ApiState`/runtime.
fn sample_event(volume: f32) -> qbz_player::player::PlaybackEvent {
    qbz_player::player::PlaybackEvent {
        is_playing: true,
        position: 10,
        duration: 200,
        track_id: 42,
        volume,
        sample_rate: None,
        bit_depth: None,
        shuffle: Some(false),
        repeat: Some("off".into()),
        normalization_gain: None,
        gapless_ready: false,
        gapless_next_track_id: 0,
        bit_perfect_mode: None,
        buffer_progress: None,
    }
}

#[test]
fn now_playing_map_path_serializes_canonical_volume() {
    // Pins the exact `now_playing` sequence: `to_value(&ev)` (which
    // widens f32 volume via `Number::from_f32`), then the map overwrite
    // that replaces it with `canon_volume`. 0.8f32 must land as `0.8`,
    // never `0.800000011920929`.
    let ev = sample_event(0.8f32);
    let mut playback = serde_json::to_value(&ev).unwrap();
    if let Value::Object(map) = &mut playback {
        map.insert("muted".into(), serde_json::json!(false));
        map.insert("queue_len".into(), serde_json::json!(3));
        map.insert("volume".into(), canon_volume(ev.volume));
    }
    let rendered = serde_json::to_string(&playback).unwrap();
    assert!(rendered.contains("\"volume\":0.8"), "got: {rendered}");
    assert!(!rendered.contains("0.80000"), "got: {rendered}");
}

#[test]
fn volume_post_response_serializes_canonical_volume() {
    // Pins the `/api/playback/volume` and `mute` response builders'
    // `json!({"volume": canon_volume(_), ...})` shape.
    let target = 0.8f32;
    let body = serde_json::json!({"volume": canon_volume(target), "muted": false});
    let rendered = serde_json::to_string(&body).unwrap();
    assert!(rendered.contains("\"volume\":0.8"), "got: {rendered}");
    assert!(!rendered.contains("0.80000"), "got: {rendered}");
}
