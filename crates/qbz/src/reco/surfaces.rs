//! Surfaces (W7/W8): home seeds + the reco-scored/forgotten album helpers.

use std::collections::HashSet;

use qbz_app::settings::reco_store::{HomeSeedLimits, HomeSeeds};

use super::RECO;

/// Read the home/Discover ID seeds. `None` when reco is disabled (no session)
/// or the read fails — callers fall back to their existing local source so a
/// cold reco store never empties a surface.
pub fn home_seeds(limits: HomeSeedLimits) -> Option<HomeSeeds> {
    let guard = RECO.lock().ok()?;
    let store = guard.as_ref()?;
    store.get_home_seeds(limits).ok()
}

/// Forgotten-favorite album ids (favorited, not played within `recency_days`).
/// `None` when reco is disabled or the read fails — the caller falls back to
/// its local heuristic so the Rediscover row never empties on a cold store.
pub fn forgotten_favorite_album_ids(limit: u32, recency_days: u32) -> Option<Vec<String>> {
    let guard = RECO.lock().ok()?;
    let store = guard.as_ref()?;
    store
        .get_forgotten_favorite_album_ids(limit, recency_days)
        .ok()
}

/// The reco-scored favorite album ids in taste order (highest first) when the
/// store is warm (trained); `None` when reco is cold/disabled so the caller
/// keeps its original ordering. Bounded by `limit`.
pub fn scored_favorite_album_ids(limit: u32) -> Option<Vec<String>> {
    let limits = HomeSeedLimits {
        recent_albums: 0,
        continue_tracks: 0,
        top_artists: 0,
        favorites: limit,
    };
    let seeds = home_seeds(limits)?;
    if seeds.favorite_album_ids.is_empty() {
        None
    } else {
        Some(seeds.favorite_album_ids)
    }
}

/// Backfill genres `(album_id, genre_id, genre_name)` onto reco events +
/// album-meta once albums are resolved. Best-effort, blocking SQLite — call
/// from `spawn_blocking`. Plays carry no genre, so this is what feeds
/// `get_top_genres`; idempotent (only fills still-NULL event genres).
pub fn backfill_album_genres(entries: Vec<(String, u64, String)>) {
    if entries.is_empty() {
        return;
    }
    if let Ok(guard) = RECO.lock() {
        if let Some(store) = guard.as_ref() {
            for (album_id, genre_id, genre_name) in entries {
                let _ = store.update_genre_for_album(&album_id, genre_id);
                let _ = store.set_album_genre_name(&album_id, &genre_name);
            }
        }
    }
}

/// Artist ids the user "knows" — played more than `play_threshold` times OR
/// favorited — to augment the discovery "skip artists I already know" filter
/// with the reco signal (plays + favorites, not plays alone). `None` when reco
/// is cold/disabled, so discovery falls back to the play_history set only.
pub fn known_artist_ids(play_threshold: u32) -> Option<HashSet<u64>> {
    let guard = RECO.lock().ok()?;
    let store = guard.as_ref()?;
    store.get_known_artist_ids(play_threshold).ok()
}

/// Most-recently-played distinct Qobuz track ids (the local "already heard
/// in-app" set). Kept for the external-reco filters; currently the deep-cut row
/// filters on album ids, so this is unused for now.
#[allow(dead_code)]
// KEPT on the author's stated intent: the doc above records it is held for
// the external-reco filters, and that the deep-cut row currently filters on
// album ids instead. That is a deliberate hold, not an oversight.
#[allow(dead_code)]
pub fn recent_track_ids(limit: u32) -> Option<Vec<u64>> {
    let guard = RECO.lock().ok()?;
    let store = guard.as_ref()?;
    store.get_recent_track_ids(limit).ok()
}
