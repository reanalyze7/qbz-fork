//! Shared Track/LocalTrack -> CoreQueueTrack mapping helpers, used by both
//! the Qobuz and local resolvers.

use qbz_models::QueueTrack as CoreQueueTrack;
// The real shared Qobuz model types (re-exported by qbz-qobuz from qbz-models).
use qbz_models::Track as ApiTrack;

/// Map a Qobuz API `Track` to a `CoreQueueTrack`.
pub fn track_to_queue_track_from_api(track: &ApiTrack) -> CoreQueueTrack {
    let artwork_url = track
        .album
        .as_ref()
        .and_then(|a| a.image.large.clone())
        .or_else(|| track.album.as_ref().and_then(|a| a.image.thumbnail.clone()))
        .or_else(|| track.album.as_ref().and_then(|a| a.image.extralarge.clone()));
    let artist = track
        .performer
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Unknown Artist".to_string());
    let album = track
        .album
        .as_ref()
        .map(|a| a.title.clone())
        .unwrap_or_else(|| "Unknown Album".to_string());
    let album_id = track.album.as_ref().map(|a| a.id.clone());
    let artist_id = track.performer.as_ref().map(|p| p.id);

    CoreQueueTrack {
        id: track.id,
        title: track.title.clone(),
        version: track.version.clone(),
        artist,
        album,
        album_version: None,
        duration_secs: track.duration as u64,
        artwork_url,
        hires: track.hires,
        bit_depth: track.maximum_bit_depth,
        sample_rate: track.maximum_sampling_rate,
        is_local: false,
        album_id,
        artist_id,
        streamable: track.streamable,
        source: Some("qobuz".to_string()),
        parental_warning: track.parental_warning,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}

/// Map a `LocalTrack` to a `CoreQueueTrack`.
/// `is_local = true`, `source = "local"`, `sample_rate` is converted from Hz
/// to kHz to match the Qobuz convention used elsewhere in the queue display.
pub fn local_track_to_queue_track(track: &qbz_library::LocalTrack) -> CoreQueueTrack {
    // Artwork: local tracks store a file path; expose it as a `file://` URL
    // so the frontend's <img> can load it. Falls back to None when absent.
    let artwork_url = track.artwork_path.as_ref().map(|p| {
        if p.starts_with("file://") {
            p.clone()
        } else {
            format!("file://{}", p)
        }
    });

    // sample_rate in LocalTrack is stored in Hz (e.g. 44100.0 / 192000.0).
    // CoreQueueTrack.sample_rate is in kHz (e.g. 44.1 / 192.0) matching the
    // Qobuz API field `maximum_sampling_rate`. Divide by 1000.
    let sample_rate_khz = track.sample_rate / 1000.0;

    CoreQueueTrack {
        // Local track ids are i64; CoreQueueTrack.id is u64.
        // Local ids start from 1 and are never negative in practice.
        id: track.id as u64,
        title: track.title.clone(),
        version: None,
        artist: track.artist.clone(),
        album: track.album_group_title.clone(),
        album_version: None,
        duration_secs: track.duration_secs,
        artwork_url,
        hires: track.bit_depth.map(|d| d > 16).unwrap_or(false),
        bit_depth: track.bit_depth,
        sample_rate: Some(sample_rate_khz),
        is_local: true,
        album_id: Some(track.album_group_key.clone()),
        artist_id: None,
        streamable: true,
        source: Some("local".to_string()),
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}
