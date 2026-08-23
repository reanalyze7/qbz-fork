//! Discover playlist -> `PlaylistCardData` mapper.

use qbz_models::DiscoverPlaylist;

use crate::home::PlaylistCardData;

/// Map a Discover playlist into a single-cover card. Preferred cover is the
/// landscape `rectangle`, falling back to the first square `cover`. Owner,
/// duration and tracks_count are intentionally dropped (1:1 with Tauri's
/// PlaylistCardLite, which shows the name only).
pub(crate) fn map_playlist(p: DiscoverPlaylist) -> PlaylistCardData {
    let artwork_url = p
        .image
        .rectangle
        .or_else(|| p.image.covers.and_then(|c| c.into_iter().next()))
        .unwrap_or_default();
    // First tag → the UPPERCASE accent subtag; all tag slugs → the filter
    // material. DiscoverPlaylist.tags is Option<Vec<PlaylistTag{id,slug,name}>>.
    // Uppercased here (Slint has no text-transform; same convention as the
    // MIXTAPE/COLLECTION eyebrow tags). The name is already localized by the
    // API response.
    let category = p
        .tags
        .as_ref()
        .and_then(|t| t.first())
        .map(|t| t.name.to_uppercase())
        .unwrap_or_default();
    let tags = p
        .tags
        .as_ref()
        .map(|t| t.iter().map(|tag| tag.slug.clone()).collect())
        .unwrap_or_default();
    PlaylistCardData {
        id: p.id.to_string(),
        title: p.name,
        artwork_url,
        category,
        tags,
    }
}
