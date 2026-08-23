//! Page fetch (with blacklist filtering) + artwork-job construction.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::home::CardData;

use super::PAGE_SIZE;

/// Fetch one page starting at `offset`, dropping blacklisted albums. Returns
/// the surviving cards, the FETCHED item count (the server offset advances by
/// the fetched — not visible — count; the blacklist drop is log-only) and
/// `has_more`. Genre filtering is server-side: the raw selection (parent or
/// sub-genre id) is in `genre_ids` and Qobuz honors sub-genre ids, so there is
/// no client-side narrowing — 1:1 with Tauri discovery-v2.
pub(super) async fn fetch_pages(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    endpoint: &str,
    genre_ids: Option<Vec<u64>>,
    offset: u32,
) -> Result<(Vec<CardData>, u32, bool), String> {
    let mut data = runtime
        .core()
        .get_discover_albums(endpoint, genre_ids, offset, PAGE_SIZE)
        .await
        .map_err(|e| e.to_string())?;
    let has_more = data.has_more;
    let fetched = data.items.len() as u32;
    // T8: drop blacklisted DiscoverAlbums (ANY of artists[], featured-aware via
    // discover_album_blacklisted). Tauri's discover surfaces log-only — no count
    // adjustment (the endpoints carry no `total`; pagination is has_more-driven).
    let (bl, abl) = if crate::artist_blacklist::is_enabled() {
        (
            crate::artist_blacklist::ids_snapshot(),
            crate::artist_blacklist::album_ids_snapshot(),
        )
    } else {
        Default::default()
    };
    if !bl.is_empty() || !abl.is_empty() {
        data.items
            .retain(|a| !qbz_core::core::discover_album_blacklisted(a, &bl, &abl));
    }
    let cards: Vec<CardData> = data.items.into_iter().map(crate::home::map_album).collect();
    Ok((cards, fetched, has_more))
}

/// Artwork jobs for a page of cards, targeting their absolute indices
/// (`base_index` is the offset of the first card in the model).
pub(super) fn artwork_jobs(cards: &[CardData], base_index: usize) -> Vec<ArtworkJob> {
    cards
        .iter()
        .enumerate()
        .filter(|(_, card)| !card.artwork_url.is_empty())
        .map(|(i, card)| ArtworkJob {
            url: card.artwork_url.clone(),
            target: ArtworkTarget::DiscoverBrowseAlbum {
                index: base_index + i,
            },
        })
        .collect()
}
