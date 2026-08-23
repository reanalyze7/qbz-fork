//! Qobuz Playlists row + category-tag extraction from the discover index.

use qbz_models::{DiscoverContainers, DiscoverPlaylist};

use crate::home::map::map_playlist;
use crate::home::PlaylistCardData;

/// Qobuz Playlists row — both the Home and Editor's Picks tabs draw from the
/// SAME `containers.playlists` (one fetch). Capped at 40 (raised from
/// Tauri's 18) so the client-side category filter has material to work with
/// without holding 100 cards' covers in memory; the carousel still pages,
/// and the un-filtered view shows the same first cards. Each card carries
/// its tag slugs for the filter. Returns (home, editor) — same data, two
/// independent cache slots.
pub(super) fn cards(containers: &mut DiscoverContainers) -> (Vec<PlaylistCardData>, Vec<PlaylistCardData>) {
    let items: Vec<DiscoverPlaylist> =
        containers.playlists.take().map(|c| c.data.items).unwrap_or_default();
    let editor: Vec<PlaylistCardData> =
        items.iter().cloned().take(40).map(map_playlist).collect();
    let home: Vec<PlaylistCardData> = items.into_iter().take(40).map(map_playlist).collect();
    (home, editor)
}

/// Category tags for the multi-select filter (slug + localized name).
pub(super) fn tags(containers: &mut DiscoverContainers) -> Vec<(String, String)> {
    containers
        .playlists_tags
        .take()
        .map(|c| {
            c.data
                .items
                .into_iter()
                .map(|tag| (tag.slug, tag.name))
                .collect()
        })
        .unwrap_or_default()
}
