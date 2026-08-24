//! Per-track "playing from" container stamping + the catalog-`Track` ->
//! `QueueTrack` builder shared by every Qobuz play/enqueue path.
use slint::ComponentHandle;

use qbz_models::QueueTrack;

/// Record the playback CONTEXT — the source the queue was launched from — on
/// `NowPlayingState`, so the song-card layers button can navigate back to it.
/// Stamp every queued track with the container it was launched FROM, so the
/// now-playing song-card "playing from" button — re-derived per track in
/// `refresh_now_playing_meta` — always points at the right source for whatever
/// is actually playing. Pass the same `kind`/`id` the old
/// `set_now_playing_context` call used at that play path ("album"/album_id,
/// "artist"/artist_id, "playlist"/playlist_id, "label"/label_id). This replaces
/// the single-global approach that went stale across track changes: the origin
/// now travels WITH each track and is republished on every advance (gapless
/// included), never cached.
pub(crate) fn stamp_queue_context(tracks: &mut [QueueTrack], kind: &str, id: &str) {
    for t in tracks.iter_mut() {
        t.context_kind = Some(kind.to_string());
        t.context_id = Some(id.to_string());
    }
}

/// LEGACY single-global context setter. Superseded by per-track
/// `stamp_queue_context` + the per-track republish in
/// `refresh_now_playing_meta` (which is now authoritative). Retained for the
/// miniplayer mirror path / potential reuse; no live caller remains.
#[allow(dead_code)]
pub fn set_now_playing_context(weak: &slint::Weak<crate::AppWindow>, kind: &str, id: &str) {
    let kind = kind.to_string();
    let id = id.to_string();
    let _ = weak.upgrade_in_event_loop(move |w| {
        let np = w.global::<crate::NowPlayingState>();
        np.set_context_kind(kind.into());
        np.set_context_id(id.into());
    });
}

/// Build a `QueueTrack` for the queue from the catalog `Track`, filling
/// the album metadata from `album_meta` (the track's own album summary is
/// often partial in album responses).
pub(crate) fn make_queue_track(
    track: &qbz_models::Track,
    album_id: &str,
    album_title: &str,
    album_artist: &str,
    album_artwork: &str,
    album_version: Option<&str>,
) -> QueueTrack {
    QueueTrack {
        id: track.id,
        title: track.title.clone(),
        version: track.version.clone(),
        artist: track
            .performer
            .as_ref()
            .map(|p| p.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| album_artist.to_string()),
        album: album_title.to_string(),
        album_version: album_version
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        duration_secs: track.duration as u64,
        artwork_url: if album_artwork.is_empty() {
            None
        } else {
            Some(album_artwork.to_string())
        },
        hires: track.hires,
        bit_depth: track.maximum_bit_depth,
        sample_rate: track.maximum_sampling_rate,
        is_local: false,
        album_id: Some(album_id.to_string()),
        artist_id: track.performer.as_ref().map(|p| p.id),
        streamable: track.streamable,
        source: Some("qobuz".to_string()),
        parental_warning: track.parental_warning,
        source_item_id_hint: Some(album_id.to_string()),
        // Container origin is stamped by the play path (stamp_queue_context);
        // the generic builder leaves it unset so single-track / search plays
        // fall back to the track's own album.
        context_kind: None,
        context_id: None,
    }
}
