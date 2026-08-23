//! Favorite-albums branch (Library Albums, Most Played, Rediscover, +
//! no-recents-seed suggest fallback) and the common-case suggest branch.

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::super::apply_misc::{
    apply_favorite_albums, apply_more_from_library, apply_most_played_albums, apply_rediscover,
};
use super::super::build_albums::{
    build_favorite_albums, build_rediscover, most_played_album_cards, order_by_score,
};
use super::super::fetch::{fetch_fav_albums, fetch_suggest};

/// ---- Branch: favorite albums -> Rediscover (+ suggest fallback) ----
pub(super) fn albums_branch<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    cache: ImageCache,
    recent_ids: HashSet<String>,
    has_recents_seed: bool,
) -> Pin<Box<dyn Future<Output = ()> + Send>>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    Box::pin(async move {
        let fav_albums = fetch_fav_albums(&runtime).await;
        // reco: lead with the highest-scored favorites (trained taste
        // order) when the store is warm; cold -> original Qobuz order.
        let scored_fav = crate::reco::scored_favorite_album_ids(80);
        let fav_albums = order_by_score(fav_albums, scored_fav.as_deref());
        apply_favorite_albums(&weak, &cache, build_favorite_albums(&fav_albums));
        // Most Played Albums — local play-count rail (no fetch); rides
        // this branch since it shares the cache + weak already cloned.
        apply_most_played_albums(&weak, &cache, most_played_album_cards());
        // reco: backfill genres for the resolved favorite albums so the
        // engine's top-genres has data (plays alone carry no genre).
        let genre_entries: Vec<(String, u64, String)> = fav_albums
            .iter()
            .filter_map(|a| {
                a.genre
                    .as_ref()
                    .filter(|g| g.id > 0)
                    .map(|g| (a.id.clone(), g.id, g.name.clone()))
            })
            .collect();
        if !genre_entries.is_empty() {
            tokio::task::spawn_blocking(move || crate::reco::backfill_album_genres(genre_entries));
        }
        // reco: prefer the reco "forgotten favorites" set when the store
        // is warm (shared events.db); fall back to the local recents
        // heuristic when cold so the Rediscover row never empties.
        let forgotten: Option<HashSet<String>> = crate::reco::forgotten_favorite_album_ids(60, 30)
            .filter(|ids| !ids.is_empty())
            .map(|ids| ids.into_iter().collect());
        apply_rediscover(
            &weak,
            &cache,
            build_rediscover(&fav_albums, &recent_ids, forgotten.as_ref()),
        );

        // Only the no-recent-history case needs the favorite-album seed;
        // the common case is handled concurrently in `suggest_branch`.
        if !has_recents_seed {
            if let Some(id) = fav_albums
                .first()
                .map(|a| a.id.clone())
                .filter(|s| !s.is_empty())
            {
                let seed_title = fav_albums.first().map(|a| a.title.clone()).unwrap_or_default();
                let cards = fetch_suggest(&runtime, &id).await;
                apply_more_from_library(&weak, &cache, cards, seed_title);
            }
        }
    })
}

/// ---- Branch: More From Your Library (common case — recent-album seed) ----
pub(super) fn suggest_branch<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    cache: ImageCache,
    recents_seed: Option<String>,
    recents_seed_title: Option<String>,
) -> Pin<Box<dyn Future<Output = ()> + Send>>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    Box::pin(async move {
        if let Some(id) = recents_seed {
            let cards = fetch_suggest(&runtime, &id).await;
            apply_more_from_library(&weak, &cache, cards, recents_seed_title.unwrap_or_default());
        }
    })
}
