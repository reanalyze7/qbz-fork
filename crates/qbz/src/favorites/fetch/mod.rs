//! Network + parsing "IO" layer: fetch and shape one favorites tab.

mod album_ids;
mod generic;
mod playlists;

pub use album_ids::favorite_album_ids;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use serde::Deserialize;
use std::sync::Arc;

use crate::favorites::mapping::{ArtistCard, LabelCard, TrackCard};
use crate::favorites::FavTab;
use crate::search::PlaylistRow;
use qbz_models::Track;

/// Favorites-labels response item — the qbz-models `Label` is just
/// {id, name}, but the favorites payload carries an image + count,
/// so parse into this richer local shape. `image` is a bare string on
/// the wire (LegacyLabelDto), but typed as Value to also tolerate the
/// `{mega|extralarge|large|thumbnail|small}` object form other label
/// surfaces return (resolved via `label::extract_label_image`).
#[derive(Deserialize)]
pub(crate) struct FavLabel {
    #[serde(default)]
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) albums_count: Option<u32>,
    #[serde(default)]
    pub(crate) image: Option<serde_json::Value>,
}

pub enum FavData {
    Tracks { items: Vec<TrackCard>, play: Vec<Track>, total: usize },
    Albums { items: Vec<crate::album_map::AlbumCard>, total: usize },
    Artists { items: Vec<ArtistCard>, total: usize },
    Playlists { favorites: Vec<PlaylistRow>, following: Vec<PlaylistRow> },
    Labels { items: Vec<LabelCard>, total: usize },
}

/// Fetch + parse one favorites tab.
pub async fn load_favorites<A>(
    runtime: &Arc<AppRuntime<A>>,
    tab: FavTab,
) -> Result<FavData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    if tab == FavTab::Playlists {
        return playlists::load_playlists(runtime).await;
    }
    generic::load_generic(runtime, tab).await
}
