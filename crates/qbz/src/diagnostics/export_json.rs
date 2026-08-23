//! Export JSON (camelCase, matching the Tauri export shape) — semantically
//! distinct from the human-readable markdown report.

use serde_json::{json, Value};

pub(super) fn build_playback_json(
    pb: &qbz_player::PlaybackState,
    track: Option<&qbz_models::QueueTrack>,
) -> Value {
    json!({
        "isPlaying": pb.is_playing,
        "volumePercent": (pb.volume * 100.0).round() as i64,
        "positionSecs": pb.position,
        "durationSecs": pb.duration,
        "hasTrack": track.is_some(),
        "trackTitle": track.map(|t| t.title.clone()),
        "trackArtist": track.map(|t| t.artist.clone()),
        "trackAlbum": track.map(|t| t.album.clone()),
        "trackQuality": Value::Null,
        "trackFormat": Value::Null,
        "trackBitDepth": track.and_then(|t| t.bit_depth),
        "trackSamplingRate": track.and_then(|t| t.sample_rate),
        "trackIsLocal": track.map(|t| t.is_local),
        "trackSource": track.and_then(|t| t.source.clone()),
    })
}
