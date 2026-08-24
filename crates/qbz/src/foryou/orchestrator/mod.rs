//! Orchestrator: `reset_loading` + `spawn_for_you`, the big concurrent-branch
//! loader for the For You tab.
use slint::ComponentHandle;

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use futures_util::future::join_all;
use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::artwork::ImageCache;
use crate::{AppWindow, ForYouState};

use super::apply_sections::apply_recent;
use super::build::{recent_album_cards, recent_track_slims};

mod branch_albums_suggest;
mod branch_release_artists;

/// Set the loading flag so the skeleton shows until the first sections paint.
pub fn reset_loading(window: &AppWindow) {
    window.global::<ForYouState>().set_loading(true);
}

/// Load every For You section progressively and in parallel, then latch
/// `loaded`. Spawned once by `ensure_for_you_loaded` on first tab open.
///
/// Dependency layers:
///   - Layer 0 (instant, no network): Recently Played albums + tracks.
///   - Layer 0 (concurrent network): release-watch, favorite-artists,
///     favorite-albums, and album-suggest (common case, seeded from the most
///     recent local album).
///   - Layer 1 (after favorite-artists): Your Top Artists (immediate) then
///     Artists to Follow ∥ Spotlight.
///   - Layer 1 (after favorite-albums): Rediscover (and the album-suggest
///     fallback when there is no recent play-history seed).
///   - Latch: `loading = false` + `loaded = true` once ALL branches resolve.
pub fn spawn_for_you<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
) where
    A: FrontendAdapter + Send + Sync + 'static,
{
    handle.spawn(async move {
        // ---- Layer 0: instant local/static sections (no await) ----
        let recent_album_list = crate::recently::load_albums();
        let recent_ids: HashSet<String> =
            recent_album_list.iter().map(|a| a.id.clone()).collect();
        let recents_seed: Option<String> = recent_album_list
            .first()
            .map(|a| a.id.clone())
            .filter(|s| !s.is_empty());
        // Seed title for the "Similar to {seed}" header — the most-recent
        // album's title (its suggestions seed the common-case row).
        let recents_seed_title: Option<String> =
            recent_album_list.first().map(|a| a.title.clone());
        let has_recents_seed = recents_seed.is_some();

        apply_recent(
            &weak,
            &image_cache,
            recent_album_cards(&recent_album_list),
            recent_track_slims(),
        );

        let release_branch: Pin<Box<dyn Future<Output = ()> + Send>> =
            branch_release_artists::release_branch(runtime.clone(), weak.clone(), image_cache.clone());
        let artists_branch: Pin<Box<dyn Future<Output = ()> + Send>> =
            branch_release_artists::artists_branch(runtime.clone(), weak.clone(), image_cache.clone());
        let albums_branch: Pin<Box<dyn Future<Output = ()> + Send>> =
            branch_albums_suggest::albums_branch(
                runtime.clone(),
                weak.clone(),
                image_cache.clone(),
                recent_ids.clone(),
                has_recents_seed,
            );
        let suggest_branch: Pin<Box<dyn Future<Output = ()> + Send>> =
            branch_albums_suggest::suggest_branch(
                runtime.clone(),
                weak.clone(),
                image_cache.clone(),
                recents_seed,
                recents_seed_title,
            );

        join_all(vec![release_branch, artists_branch, albums_branch, suggest_branch]).await;

        // ---- All branches resolved: latch loaded so re-entry is a no-op ----
        let _ = weak.upgrade_in_event_loop(|w| {
            let state = w.global::<ForYouState>();
            state.set_loading(false);
            state.set_loaded(true);
        });
    });
}
