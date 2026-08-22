//! Async Qobuz resolver free fns: album / track / playlist -> CoreQueueTrack.

use qbz_models::QueueTrack as CoreQueueTrack;

use super::mapping::track_to_queue_track_from_api;

// ── Qobuz album ──

pub async fn resolve_qobuz_album(
    client: &qbz_qobuz::QobuzClient,
    album_id: &str,
) -> Result<Vec<CoreQueueTrack>, String> {
    let album = client
        .get_album(album_id)
        .await
        .map_err(|e| format!("Qobuz get_album({}) failed: {}", album_id, e))?;

    let tracks = match album.tracks {
        Some(container) => container.items,
        None => return Err(format!("album {} returned no tracks container", album_id)),
    };

    if tracks.is_empty() {
        return Err(format!("album {} has 0 tracks", album_id));
    }

    // Build QueueTrack from each track. We have the parent Album in scope so
    // we can fill artwork / album title / album artist even when the track's
    // own `album` field is absent (shallow responses inside albums/get).
    let album_artwork = album.image.large.clone()
        .or_else(|| album.image.extralarge.clone())
        .or_else(|| album.image.thumbnail.clone());
    let album_title = album.title.clone();
    let album_artist_name = album.artist.name.clone();
    let album_id_str = album.id.clone();

    let result = tracks
        .into_iter()
        .map(|track| {
            // Prefer the track's own performer; fall back to album artist.
            let artist = track
                .performer
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| album_artist_name.clone());
            let artist_id = track.performer.as_ref().map(|p| p.id);
            // Prefer the track's nested album image when present.
            let artwork = track
                .album
                .as_ref()
                .and_then(|a| a.image.large.clone().or_else(|| a.image.thumbnail.clone()))
                .or_else(|| album_artwork.clone());

            CoreQueueTrack {
                id: track.id,
                title: track.title.clone(),
                version: track.version.clone(),
                artist,
                album: album_title.clone(),
                album_version: None,
                duration_secs: track.duration as u64,
                artwork_url: artwork,
                hires: track.hires,
                bit_depth: track.maximum_bit_depth,
                sample_rate: track.maximum_sampling_rate,
                is_local: false,
                album_id: Some(album_id_str.clone()),
                artist_id,
                streamable: track.streamable,
                source: Some("qobuz".to_string()),
                parental_warning: track.parental_warning,
                // Stamped centrally by resolve_collection_tracks; left None here.
                source_item_id_hint: None,
                context_kind: None,
                context_id: None,
            }
        })
        .collect();

    Ok(result)
}

// ── Qobuz track ──

pub async fn resolve_qobuz_track(
    client: &qbz_qobuz::QobuzClient,
    track_id: u64,
) -> Result<Vec<CoreQueueTrack>, String> {
    let track = client
        .get_track(track_id)
        .await
        .map_err(|e| format!("Qobuz get_track({}) failed: {}", track_id, e))?;

    Ok(vec![track_to_queue_track_from_api(&track)])
}

// ── Qobuz playlist ──

pub async fn resolve_qobuz_playlist(
    client: &qbz_qobuz::QobuzClient,
    playlist_id: u64,
) -> Result<Vec<CoreQueueTrack>, String> {
    let playlist = client
        .get_playlist(playlist_id)
        .await
        .map_err(|e| format!("Qobuz get_playlist({}) failed: {}", playlist_id, e))?;

    let tracks = match playlist.tracks {
        Some(container) => container.items,
        None => return Err(format!("playlist {} returned no tracks", playlist_id)),
    };

    if tracks.is_empty() {
        return Err(format!("playlist {} has 0 tracks", playlist_id));
    }

    Ok(tracks.iter().map(track_to_queue_track_from_api).collect())
}
