//! Album page: the Last.fm "similar albums" row.

use std::sync::{Arc, Mutex};

use qbz_app::shell::AppRuntime;
use qbz_external_reco::{
    build_similar_albums_seeded, AlbumReco, LastFmHandle, LocalHistory, RecoCache, RecoInputs,
};
use qbz_integrations::{LastFmClient, MusicBrainzClient};

use crate::adapter::SlintAdapter;

use super::{rotation_seed, CoreRecoCatalog, CACHE_DIR};

/// 30-day TTL for the album-page Last.fm row's resolved result. Similar-artist
/// data is stable; a long window keeps Last.fm/Qobuz traffic near-zero on repeat
/// opens, while still refreshing often enough that an emerging artist's growing
/// similar set is picked up within a month.
const LASTFM_SIMILAR_TTL_SECS: i64 = 30 * 86_400;

/// Album page: build the Last.fm "similar albums" row for `seed_artist`
/// (the open album's primary artist), excluding albums already shown by the
/// Qobuz `/album/suggest` row (`exclude_pairs`/`exclude_ids`). Returns empty
/// when Last.fm is not connected. Reuses the same catalog, resolution cache,
/// and rotation as the Discover Recommendations tab.
///
/// The resolved result is cached per `album_id` for 30 days (the same
/// `RecoCache` results store the Discover tab uses) so re-opening an album
/// makes ZERO Last.fm/Qobuz calls. Only non-empty results are cached, so a
/// transient Last.fm failure (empty result) re-fetches on the next open instead
/// of hiding the row for the whole window.
pub async fn load_similar_albums_seeded(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    album_id: &str,
    seed_artist: &str,
    exclude_pairs: &[(String, String)],
    exclude_ids: &std::collections::HashSet<String>,
) -> Vec<AlbumReco> {
    let cfg = crate::scrobbler_settings::get();
    if !cfg.lastfm_is_authed() || cfg.lastfm_username.is_empty() {
        return Vec::new();
    }
    let cache_dir = CACHE_DIR.lock().ok().and_then(|g| g.clone());
    let cache = match &cache_dir {
        Some(dir) => RecoCache::open_at(dir).ok().map(Mutex::new),
        None => None,
    };
    let cache_key = format!("album_lastfm:{album_id}");

    // Cache hit: return the resolved row without any Last.fm/Qobuz traffic.
    if let Some(c) = &cache {
        if let Some(json) = c
            .lock()
            .ok()
            .and_then(|g| g.get_results(&cache_key, LASTFM_SIMILAR_TTL_SECS))
        {
            if let Ok(cached) = serde_json::from_str::<Vec<AlbumReco>>(&json) {
                return cached;
            }
        }
    }

    let lastfm_client = LastFmClient::new();
    let mb_client = MusicBrainzClient::new();
    let catalog = CoreRecoCatalog {
        runtime: runtime.clone(),
    };
    let inputs = RecoInputs {
        lastfm: Some(LastFmHandle {
            username: cfg.lastfm_username.clone(),
            client: &lastfm_client,
        }),
        listenbrainz: None,
        musicbrainz: &mb_client,
        catalog: &catalog,
        cache: cache.as_ref(),
        local: LocalHistory::default(),
        rotation_seed: rotation_seed(),
    };
    let mut recos = build_similar_albums_seeded(&inputs, seed_artist, exclude_pairs).await;
    // Drop any that resolved to a Qobuz id already shown by the Qobuz row
    // (the pre-resolution artist|title dedup can miss these).
    recos.retain(|r| !exclude_ids.contains(&r.qobuz_album_id));

    // Cache only a non-empty result (an empty one is likely transient).
    if !recos.is_empty() {
        if let Some(c) = &cache {
            if let (Ok(g), Ok(json)) = (c.lock(), serde_json::to_string(&recos)) {
                g.put_results(&cache_key, &json);
            }
        }
    }
    recos
}
