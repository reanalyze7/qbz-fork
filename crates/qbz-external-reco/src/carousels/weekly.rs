//! Weekly playlists (ListenBrainz curated: exploration / jams).
//!
//! These have their OWN ListenBrainz cadence: a brand-new playlist (new mbid +
//! date) every Monday. They are cached per-week, keyed by the playlist mbid, and
//! are DELIBERATELY decoupled from the shared 48h results blob — bundling them in
//! it is what made them vanish (a transient empty build got cached for 48h, and
//! the 7d per-track negative cache compounded it across rebuilds). See cache/mod.rs.

use crate::types::{RecoSource, TrackCandidate, TrackReco};
use crate::RecoInputs;

use super::validate_pools::validate_track_pool;
use super::weekly_discover::{cached_weekly_fallback, find_current_playlist};
use super::PLAYLIST_CAP;

pub async fn build_weekly(inputs: &RecoInputs<'_>, source_patch: &str) -> Vec<TrackReco> {
    let Some(lb) = &inputs.listenbrainz else {
        log::info!("[reco] weekly '{source_patch}': ListenBrainz not connected — skipping");
        return Vec::new();
    };

    // Discover the current week's playlist for this patch (one cheap call).
    let (matching, chosen) = find_current_playlist(lb.client, &lb.username, source_patch).await;
    log::info!(
        "[reco] weekly '{source_patch}': {matching} created-for playlists from ListenBrainz match the patch"
    );
    let Some(meta) = chosen else {
        log::warn!(
            "[reco] weekly '{source_patch}': ListenBrainz returned no matching playlist \
             (rate-limit / not generated yet) — serving last cached week"
        );
        return cached_weekly_fallback(inputs.cache, source_patch);
    };
    let week = meta.created_at.as_deref().unwrap_or("?");

    // Week-keyed cache: the mbid changes every Monday, so a new week is a natural
    // miss and the current week is served instantly (no Qobuz/MusicBrainz round-trips).
    let cache_key = format!("{}:{}", source_patch, meta.playlist_mbid);
    if let Some(c) = inputs.cache {
        if let Some(json) = c.lock().ok().and_then(|g| g.get_weekly(&cache_key)) {
            if let Ok(tracks) = serde_json::from_str::<Vec<TrackReco>>(&json) {
                if !tracks.is_empty() {
                    log::info!(
                        "[reco] weekly '{source_patch}': cache hit — {} tracks (week {week})",
                        tracks.len()
                    );
                    return tracks;
                }
            }
        }
    }

    // Cache miss for this week: fetch + resolve to Qobuz.
    let raw = lb
        .client
        .get_playlist_tracks(&meta.playlist_mbid)
        .await
        .unwrap_or_default();
    let candidates: Vec<TrackCandidate> = raw
        .into_iter()
        .filter(|t| !t.title.is_empty() && !t.artist_name.is_empty())
        .map(|t| TrackCandidate {
            artist: t.artist_name,
            title: t.title,
            album: t.release_name,
            duration_ms: None,
            isrc: None,
            recording_mbid: t.recording_mbid,
            source: RecoSource::ListenBrainz,
            score: 0.0,
        })
        .collect();
    let cand_count = candidates.len();
    log::info!(
        "[reco] weekly '{source_patch}': fetched {cand_count} tracks (week {week}, mbid {}); resolving to Qobuz…",
        meta.playlist_mbid
    );

    // skip_negative=true: a transient throttle on these tracks must NOT stick as
    // a 7-day negative (that locked the rows empty across rebuilds).
    // skip_mb=true: bypass the serial 1.1s/req MusicBrainz ISRC lookup (~110s
    // for 100 tracks) and resolve via fuzzy Qobuz search — fast and reliable
    // for these mainstream playlists. This is the fix for "the row never paints".
    let pool = validate_track_pool(
        inputs.catalog,
        inputs.musicbrainz,
        inputs.cache,
        candidates,
        true,
        true,
    )
    .await;
    let resolved: Vec<TrackReco> = pool.into_iter().take(PLAYLIST_CAP).collect();
    log::info!(
        "[reco] weekly '{source_patch}': resolved {} / {cand_count} tracks to Qobuz (week {week}, mbid {})",
        resolved.len(),
        meta.playlist_mbid
    );

    if !resolved.is_empty() {
        // Persist the resolved set for this week (only when non-empty).
        if let Some(c) = inputs.cache {
            if let (Ok(g), Ok(json)) = (c.lock(), serde_json::to_string(&resolved)) {
                g.put_weekly(&cache_key, source_patch, &json);
            }
        }
        return resolved;
    }

    // Resolved empty this build (transient) — show last cached week, not nothing.
    log::warn!(
        "[reco] weekly '{source_patch}': resolved 0 tracks this build — serving last cached week"
    );
    cached_weekly_fallback(inputs.cache, source_patch)
}
