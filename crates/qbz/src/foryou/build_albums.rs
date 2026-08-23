//! Pure builders (no network) for the album-based sections, plus the async
//! wrapper Home (#566) shares with For You, and the local play-count rail.

use std::collections::HashSet;
use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::Album;

use super::fetch::fetch_fav_albums;
use super::models::map_album;
use super::AlbumCard;

/// Rediscover — favorite albums the user hasn't returned to lately. Prefers the
/// reco store's "forgotten favorites" (favorited, not played in 30d, from the
/// Tauri-shared events.db) when warm; falls back to "not in the local recents
/// cache" when reco is cold so the row never empties.
pub(super) fn build_rediscover(
    fav_albums: &[Album],
    recent_ids: &HashSet<String>,
    forgotten: Option<&HashSet<String>>,
) -> Vec<AlbumCard> {
    fav_albums
        .iter()
        .filter(|a| match forgotten {
            Some(set) => set.contains(&a.id),
            None => !recent_ids.contains(&a.id),
        })
        .take(18)
        .cloned()
        .map(map_album)
        .collect()
}

/// Favorite Albums — the user's favorited albums, capped at 18, in favorite
/// order (matches Tauri's home-resolved favoriteAlbums sliced 18; unfiltered,
/// unlike Rediscover).
/// Reorder resolved albums so the reco-scored favorites lead (taste order),
/// keeping unscored albums in their original relative order. `scored` is the
/// reco favorite-album id order; `None`/empty leaves the input untouched, so a
/// cold reco store never reorders (no regression).
pub(super) fn order_by_score(mut albums: Vec<Album>, scored: Option<&[String]>) -> Vec<Album> {
    if let Some(order) = scored {
        if !order.is_empty() {
            albums.sort_by_key(|a| order.iter().position(|id| id == &a.id).unwrap_or(usize::MAX));
        }
    }
    albums
}

pub(super) fn build_favorite_albums(fav_albums: &[Album]) -> Vec<AlbumCard> {
    fav_albums.iter().take(18).cloned().map(map_album).collect()
}

/// The full "Library Albums" data pipeline (fetch -> taste-order -> cap 18)
/// as one call. Shared with Home (#566): `home::load_home` populates
/// `HomeState.favorite-albums` from this SAME pipeline, so the Home rail and
/// the For You rail render identical data. For You's own `albums_branch`
/// keeps calling the pieces directly because it also needs the un-capped
/// `Vec<Album>` for Rediscover / the genre backfill.
pub(crate) async fn favorite_album_cards<A>(runtime: &Arc<AppRuntime<A>>) -> Vec<AlbumCard>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let fav_albums = fetch_fav_albums(runtime).await;
    let scored = crate::reco::scored_favorite_album_ids(80);
    let fav_albums = order_by_score(fav_albums, scored.as_deref());
    build_favorite_albums(&fav_albums)
}

/// Cards for the "Most Played Albums" rail — top 20 albums by local play
/// count (`album_play_history`). Local (no network); the SAME set feeds Home
/// and For You. Ranked rows map 1:1 onto `AlbumCard`.
pub(crate) fn most_played_album_cards() -> Vec<AlbumCard> {
    qbz_app::settings::album_play_history::top_albums(20)
        .into_iter()
        .map(|r| AlbumCard {
            id: r.album_id,
            title: r.title,
            artist: r.artist,
            artist_id: r.artist_id,
            year: r.year,
            quality_tier: r.quality_tier,
            quality_label: r.quality_label,
            artwork_url: r.artwork_url,
        })
        .collect()
}
