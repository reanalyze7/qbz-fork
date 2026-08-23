use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::{MoreRows, SearchCategory, PAGE_SIZE};
use crate::search::mappers::{map_album, map_artist, map_playlist, map_track};

/// Fetch the next page for one category, starting at `offset`.
pub async fn load_more<A>(
    runtime: &Arc<AppRuntime<A>>,
    query: &str,
    category: SearchCategory,
    search_type: Option<String>,
    offset: u32,
) -> Result<MoreRows, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let core = runtime.core();
    let search_type = search_type.as_deref();
    // Page-2+ was unfiltered before (the core pass-throughs carry no &bl).
    // Post-filter here, closing both the album and the artist leak. Shared
    // enabled gate.
    let (bl, abl) = if crate::artist_blacklist::is_enabled() {
        (
            crate::artist_blacklist::ids_snapshot(),
            crate::artist_blacklist::album_ids_snapshot(),
        )
    } else {
        Default::default()
    };
    match category {
        SearchCategory::Albums => {
            let page = core
                .search_albums(query, PAGE_SIZE, offset, search_type)
                .await
                .map_err(|e| e.to_string())?;
            Ok(MoreRows::Albums(
                page.items
                    .into_iter()
                    .filter(|a| !qbz_core::core::album_blacklisted(a, &bl, &abl))
                    .map(map_album)
                    .collect(),
            ))
        }
        SearchCategory::Tracks => {
            let page = core
                .search_tracks(query, PAGE_SIZE, offset, search_type)
                .await
                .map_err(|e| e.to_string())?;
            Ok(MoreRows::Tracks(
                page.items
                    .into_iter()
                    .filter(|t| !qbz_core::core::track_blacklisted(t, &bl, &abl))
                    .map(map_track)
                    .collect(),
            ))
        }
        SearchCategory::Artists => {
            let (page, favs) = tokio::join!(
                core.search_artists(query, PAGE_SIZE, offset, search_type),
                core.favorite_artist_ids(),
            );
            let page = page.map_err(|e| e.to_string())?;
            let favs = favs.unwrap_or_default();
            Ok(MoreRows::Artists(
                page.items
                    .iter()
                    .filter(|a| !bl.contains(&a.id))
                    .map(|a| map_artist(a, favs.contains(&a.id)))
                    .collect(),
            ))
        }
        SearchCategory::Playlists => {
            let page = core
                .search_playlists(query, PAGE_SIZE, offset)
                .await
                .map_err(|e| e.to_string())?;
            Ok(MoreRows::Playlists(
                page.items.into_iter().map(map_playlist).collect(),
            ))
        }
    }
}
