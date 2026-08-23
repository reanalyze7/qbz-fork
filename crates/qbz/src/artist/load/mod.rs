//! Async fetch functions that build [`crate::artist::data::ArtistData`].

mod map;

pub(crate) use map::map_artist;

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::artist::data::ArtistData;
use crate::home::CardData;

/// Items fetched per `get_releases_grid` load-more page.
pub const RELEASE_PAGE_SIZE: u32 = 20;

/// Fetch and map an artist page by id.
pub async fn load_artist<A>(
    runtime: &Arc<AppRuntime<A>>,
    artist_id: &str,
) -> Result<ArtistData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let id: u64 = artist_id
        .parse()
        .map_err(|_| format!("invalid artist id: {artist_id}"))?;
    let page = runtime
        .core()
        .get_artist_page(id, None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(map_artist(page))
}

/// Fetch one more page of an artist's releases for a given bucket via
/// `get_releases_grid` (the reused, already-wired endpoint). Returns the
/// mapped cards + the server `has_more` flag.
pub async fn load_release_page<A>(
    runtime: &Arc<AppRuntime<A>>,
    artist_id: &str,
    release_type: &str,
    offset: u32,
) -> Result<(Vec<CardData>, bool), String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let id: u64 = artist_id
        .parse()
        .map_err(|_| format!("invalid artist id: {artist_id}"))?;
    let resp = runtime
        .core()
        .get_releases_grid(id, release_type, RELEASE_PAGE_SIZE, offset, Some("release_date"))
        .await
        .map_err(|e| e.to_string())?;
    let has_more = resp.has_more;
    let cards = resp
        .items
        .into_iter()
        .map(crate::artist::track_map::map_release)
        .filter(|c| !crate::artist_blacklist::card_blacklisted(&c.id, &c.artist_id))
        .collect();
    Ok((cards, has_more))
}
