//! Artists to Follow fetch helper.

use std::collections::HashSet;
use std::sync::Arc;

use futures_util::future::join_all;
use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::Artist;

use super::models::map_artist;
use super::{ArtistSlim, ARTIST_SEEDS, FOLLOW_MAX, SIMILAR_PER_SEED};

/// Artists to Follow — similar artists seeded from up to `ARTIST_SEEDS`
/// favorites, excluding ones already followed.
///
/// The ≤4 seed calls are issued CONCURRENTLY (was a sequential await loop),
/// but the dedup + `FOLLOW_MAX` cap are then re-applied SEQUENTIALLY over the
/// joined results IN SEED ORDER — this preserves the exact membership the old
/// sequential loop produced (same `seen` set seeded with the favorite ids,
/// same first-wins dedup, same early cap), only faster.
pub(super) async fn fetch_to_follow<A>(
    runtime: &Arc<AppRuntime<A>>,
    fav_artists: &[Artist],
    favorite_ids: &HashSet<u64>,
) -> Vec<ArtistSlim>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let seeds: Vec<u64> = fav_artists.iter().take(ARTIST_SEEDS).map(|a| a.id).collect();
    let futures = seeds.into_iter().map(|id| {
        let runtime = runtime.clone();
        async move { runtime.core().get_similar_artists(id, SIMILAR_PER_SEED, 0).await }
    });
    let results = join_all(futures).await; // Vec<Result<..>> in seed order

    let mut seen: HashSet<u64> = favorite_ids.clone();
    let mut to_follow: Vec<ArtistSlim> = Vec::new();
    'outer: for res in results {
        if let Ok(page) = res {
            for artist in page.items {
                if to_follow.len() >= FOLLOW_MAX {
                    break 'outer;
                }
                // T8: similar-artists surface — drop blacklisted artist ids
                // (is_blacklisted auto-gates on the enabled flag). This is
                // the v2_get_similar_artists equivalent; the carousel has no
                // surfaced total to decrement, so a drop just yields fewer
                // rows. NOT to be confused with the artist-detail page's own
                // similar list (a parity-negative left untouched).
                if crate::artist_blacklist::is_blacklisted(artist.id) {
                    continue;
                }
                if seen.insert(artist.id) {
                    to_follow.push(map_artist(artist, false));
                }
            }
        }
    }
    to_follow
}
