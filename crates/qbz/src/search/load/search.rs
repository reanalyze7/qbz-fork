use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::search::mappers::map_search_all;
use crate::search::rows::SearchData;

/// Run a combined search and map it to plain `Send` data. The search and
/// the user's followed-artist set are fetched concurrently.
pub async fn load_search<A>(
    runtime: &Arc<AppRuntime<A>>,
    query: &str,
) -> Result<SearchData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    // Blacklist filtering (featured-aware via qbz-core helpers); skipped when the feature is disabled.
    let blacklist = if crate::artist_blacklist::is_enabled() {
        crate::artist_blacklist::ids_snapshot()
    } else {
        std::collections::HashSet::new()
    };
    // Album axis shares the same enabled gate.
    let album_blacklist = if crate::artist_blacklist::is_enabled() {
        crate::artist_blacklist::album_ids_snapshot()
    } else {
        std::collections::HashSet::new()
    };
    let core = runtime.core();
    let (results, favs) = tokio::join!(
        core.search_all(query, &blacklist, &album_blacklist),
        core.favorite_artist_ids(),
    );
    let results = results.map_err(|e| e.to_string())?;
    // Replace the cache only on a SUCCESSFUL fetch: set_all_artists syncs to
    // the per-user disk store, so wiping it on a failed fetch (empty default)
    // corrupted the follow set across restarts — the Home/ForYou Pinned
    // carousel seeds from it at build time and showed every artist as
    // not-followed. A failed fetch still maps this page with an empty set
    // (transient), but must not touch the persisted cache.
    let favs_ok = favs.is_ok();
    let favs = favs.unwrap_or_default();
    if favs_ok {
        // Seed the in-memory artist fav cache so the follow toggle has current state.
        crate::fav_cache::set_all_artists(favs.clone());
    }
    Ok(map_search_all(query, results, &favs))
}
