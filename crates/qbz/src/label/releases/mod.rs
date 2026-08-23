//! `LabelReleasesView` — loads the label header (name + image from
//! /label/page) and the paginated album catalog (from /label/getAlbums),
//! pushing them into `LabelState`.

mod derive;
mod view;

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::{bl_snapshots, extract_label_image};
use crate::album_map::{map_album, AlbumCard};

pub use derive::derive_releases;
pub use view::{apply_image, apply_label, append_albums, artwork_jobs, reset_label};

/// Page size for the album catalog. Tauri pulls 500 at a time; keep
/// the same so a typical label loads in one shot.
pub const PAGE_SIZE: u32 = 500;

pub struct LabelData {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub albums: Vec<AlbumCard>,
    pub total: usize,
    pub has_more: bool,
}

/// Fetch the label page (name + image) and the first album page.
pub async fn load_label<A>(
    runtime: &Arc<AppRuntime<A>>,
    label_id: u64,
    fallback_name: &str,
) -> Result<LabelData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let page = runtime
        .core()
        .get_label_page(label_id)
        .await
        .map_err(|e| e.to_string())?;

    let albums_page = runtime
        .core()
        .get_label_albums(label_id, PAGE_SIZE, 0, None, None, None, None, None)
        .await
        .map_err(|e| e.to_string())?;

    let name = if page.name.is_empty() {
        fallback_name.to_string()
    } else {
        page.name
    };
    let image_url = extract_label_image(page.image.as_ref());
    let item_count = albums_page.items.len();
    let total = albums_page
        .total
        .map(|t| t as usize)
        .unwrap_or(item_count);
    // /label/getAlbums caps each page below the full catalog; trust the
    // `has_more` flag, falling back to a total comparison when it's absent.
    let has_more = albums_page.has_more.unwrap_or(total > item_count);
    let (bl, abl) = bl_snapshots();
    let albums = albums_page
        .items
        .into_iter()
        .filter(|a| !qbz_core::core::album_blacklisted(a, &bl, &abl))
        .map(map_album)
        .collect();

    Ok(LabelData {
        id: label_id.to_string(),
        name,
        image_url,
        albums,
        total,
        has_more,
    })
}

/// Fetch one more album page for the load-more affordance. Returns the
/// new cards, the (best-known) total, and whether more pages remain.
pub async fn load_more_albums<A>(
    runtime: &Arc<AppRuntime<A>>,
    label_id: u64,
    offset: u32,
) -> Result<(Vec<AlbumCard>, usize, bool), String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let page = runtime
        .core()
        .get_label_albums(label_id, PAGE_SIZE, offset, None, None, None, None, None)
        .await
        .map_err(|e| e.to_string())?;
    let item_count = page.items.len();
    let loaded = offset as usize + item_count;
    let total = page.total.map(|t| t as usize).unwrap_or(loaded);
    // More pages remain when the API says so, or when this page came back
    // full (a short page means the catalog is exhausted).
    let has_more = page
        .has_more
        .unwrap_or(item_count >= PAGE_SIZE as usize || total > loaded);
    let (bl, abl) = bl_snapshots();
    let albums = page
        .items
        .into_iter()
        .filter(|a| !qbz_core::core::album_blacklisted(a, &bl, &abl))
        .map(map_album)
        .collect();
    Ok((albums, total, has_more))
}
