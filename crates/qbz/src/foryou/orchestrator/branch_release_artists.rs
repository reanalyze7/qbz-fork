//! Release Watch + favorite-artists branches (Top Artists, To-Follow,
//! Spotlight).

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use futures_util::future::join_all;
use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::super::apply_misc::apply_spotlight;
use super::super::apply_sections::{apply_release_watch, apply_to_follow, apply_top_artists};
use super::super::build::top_artist_slims;
use super::super::fetch::{fetch_fav_artists, fetch_release_watch};
use super::super::follow::fetch_to_follow;
use super::super::spotlight::load_spotlight;

/// ---- Branch: Release Watch (independent) ----
pub(super) fn release_branch<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    cache: ImageCache,
) -> Pin<Box<dyn Future<Output = ()> + Send>>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    Box::pin(async move {
        let cards = fetch_release_watch(&runtime).await;
        apply_release_watch(&weak, &cache, cards);
    })
}

/// ---- Branch: favorite artists -> Top Artists, then To-Follow ∥ Spotlight ----
pub(super) fn artists_branch<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    cache: ImageCache,
) -> Pin<Box<dyn Future<Output = ()> + Send>>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    Box::pin(async move {
        let fav_artists = fetch_fav_artists(&runtime).await;
        apply_top_artists(&weak, &cache, top_artist_slims(&fav_artists));

        let favorite_ids: HashSet<u64> = fav_artists.iter().map(|a| a.id).collect();

        let follow_branch: Pin<Box<dyn Future<Output = ()> + Send>> = {
            let runtime = runtime.clone();
            let weak = weak.clone();
            let cache = cache.clone();
            let fav_artists = fav_artists.clone();
            let favorite_ids = favorite_ids.clone();
            Box::pin(async move {
                let to_follow = fetch_to_follow(&runtime, &fav_artists, &favorite_ids).await;
                apply_to_follow(&weak, &cache, to_follow);
            })
        };
        let spotlight_branch: Pin<Box<dyn Future<Output = ()> + Send>> = {
            let runtime = runtime.clone();
            let weak = weak.clone();
            let cache = cache.clone();
            let fav_artists = fav_artists.clone();
            Box::pin(async move {
                let sp = load_spotlight(&runtime, &fav_artists).await;
                apply_spotlight(&weak, &cache, sp);
            })
        };
        join_all(vec![follow_branch, spotlight_branch]).await;
    })
}
