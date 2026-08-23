//! Catalog Track -> DB row mapper.

use qbz_offline_cache::TrackCacheInfo;

/// Build the DB row metadata from a catalog track. Offline copies are always
/// fetched at the top quality tier.
pub(super) fn track_cache_info(track: &qbz_models::Track) -> TrackCacheInfo {
    TrackCacheInfo {
        track_id: track.id,
        title: track.title.clone(),
        artist: track
            .performer
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default(),
        album: track.album.as_ref().map(|a| a.title.clone()),
        album_id: track.album.as_ref().map(|a| a.id.clone()),
        duration_secs: track.duration as u64,
        quality: "UltraHiRes".to_string(),
        bit_depth: track.maximum_bit_depth,
        sample_rate: track.maximum_sampling_rate,
    }
}
