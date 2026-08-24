//! Pure Discover -> Card/Slim/Playlist mappers. No I/O, no Slint types.

mod album;
mod playlist;
mod quality;

pub(crate) use album::map_album;
pub(crate) use playlist::map_playlist;

use qbz_app::settings::discover_prefs::DiscoverySectionId;
use qbz_models::{DiscoverAlbum, DiscoverContainer};

use super::{SectionData, SlimData};

pub(super) fn push_section(
    out: &mut Vec<SectionData>,
    id: DiscoverySectionId,
    title: &str,
    endpoint: &str,
    container: Option<DiscoverContainer<DiscoverAlbum>>,
) {
    let Some(container) = container else {
        return;
    };
    if container.data.items.is_empty() {
        return;
    }
    out.push(SectionData {
        id,
        title: title.to_string(),
        endpoint: endpoint.to_string(),
        albums: container.data.items.into_iter().map(map_album).collect(),
    });
}

/// Like `push_section` but borrows the container (clones the items)
/// so the same data can feed more than one tab's section set.
pub(super) fn push_section_ref(
    out: &mut Vec<SectionData>,
    id: DiscoverySectionId,
    title: &str,
    endpoint: &str,
    container: &Option<DiscoverContainer<DiscoverAlbum>>,
) {
    let Some(container) = container else {
        return;
    };
    if container.data.items.is_empty() {
        return;
    }
    out.push(SectionData {
        id,
        title: title.to_string(),
        endpoint: endpoint.to_string(),
        albums: container.data.items.iter().cloned().map(map_album).collect(),
    });
}

pub(super) fn map_slim(index: usize, album: DiscoverAlbum) -> SlimData {
    let subtitle = album
        .artists
        .first()
        .map(|a| a.name.clone())
        .unwrap_or_default();
    let artwork_url = album
        .image
        .thumbnail
        .or(album.image.small)
        .or(album.image.large)
        .unwrap_or_default();
    SlimData {
        id: album.id,
        title: album.title,
        subtitle,
        rank: (index + 1).to_string(),
        artwork_url,
    }
}
