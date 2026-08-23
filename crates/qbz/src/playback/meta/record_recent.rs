//! Recording the currently-playing track into the recently-played store
//! (Discover "Recently Played" + Home rails + play-history + reco logging).

use super::super::quality::recent_quality;
use super::super::state::queue_controller;
use super::super::Runtime;

/// Record the currently playing queue track in the recently-played store
/// so the Discover "Recently Played" sections fill.
pub(in super::super) async fn record_recent(runtime: &Runtime) {
    let state = runtime.core().get_queue_state().await;
    let Some(track) = state.current_track else {
        return;
    };
    let artwork = track.artwork_url.clone().unwrap_or_default();
    let album_id = track.album_id.clone().unwrap_or_default();
    // Prefer the album-level metadata captured at album-fetch time (genre,
    // release date, and the album's own max quality) over the single
    // track's values — the `album/get` track summaries are often partial.
    let meta = crate::recently::album_meta(&album_id).unwrap_or_default();
    let (track_tier, track_label) = recent_quality(track.bit_depth, track.sample_rate);
    let quality_tier = if !meta.quality_tier.is_empty() {
        meta.quality_tier
    } else {
        track_tier
    };
    let quality_label = if !meta.quality_label.is_empty() {
        meta.quality_label
    } else {
        track_label
    };
    // Per-album play count — feeds the "Most Played Albums" rail (top-20
    // COUNT(*) GROUP BY album_id). Per-track-start, like the artist store
    // below; no-op when album_id is empty. Same album identity `recently`
    // uses, so the two rails agree.
    {
        let artist_id_str = track.artist_id.map(|id| id.to_string()).unwrap_or_default();
        qbz_app::settings::album_play_history::record_album_play(
            qbz_app::settings::album_play_history::AlbumPlayMeta {
                album_id: &album_id,
                title: &track.album,
                artist: &track.artist,
                artist_id: &artist_id_str,
                artwork_url: &artwork,
                quality_tier: &quality_tier,
                quality_label: &quality_label,
                year: meta.release_date.get(0..4).unwrap_or(""),
                source: track.source.as_deref().unwrap_or("qobuz"),
            },
        );
    }
    crate::recently::record(crate::recently::RecentTrack {
        id: track.id.to_string(),
        title: track.title.clone(),
        subtitle: track.artist.clone(),
        artwork_url: artwork.clone(),
        album_id,
        album_title: track.album.clone(),
        album_artist: track.artist.clone(),
        album_artwork_url: artwork,
        quality_tier,
        quality_label,
        genre: meta.genre,
        release_date: meta.release_date,
        artist_id: track.artist_id,
        source: track.source.clone().unwrap_or_else(|| "qobuz".to_string()),
    });
    // Per-artist play count — feeds the discovery filter "skip
    // artists I already know" (HavingCount > threshold). artist_id
    // is optional on QueueTrack; skip when absent.
    if let Some(artist_id) = track.artist_id {
        crate::play_history::record_play(artist_id, &track.artist);
    }
    // reco: log this play for taste scoring. The helper gates to Qobuz-catalog
    // sources only (local/ephemeral ids don't resolve against the Qobuz
    // catalog and would poison the home seeds). SQLite is blocking, so it runs
    // on the blocking pool, off the async record_recent path.
    let (rid, ralb, rart, rsrc) = (
        track.id,
        track.album_id.clone(),
        track.artist_id,
        track.source.clone(),
    );
    tokio::task::spawn_blocking(move || {
        crate::reco::log_play_gated(rid, ralb, rart, rsrc.as_deref());
    });
    // Home rails auto-refresh: the recently-played store just changed, so
    // notify the UI layer — it re-reads the LOCAL store into the Home rails
    // NOW if the Home view is showing (small JSON read, cached artwork), or
    // leaves them dirty for the next Home mount. Reaches the window through
    // the global queue controller, like apply_plex_quality_to_queue
    // (record_recent does not carry a weak).
    if let Some(controller) = queue_controller() {
        crate::note_recent_store_changed(controller.weak().clone(), controller.handle().clone());
    }
}
