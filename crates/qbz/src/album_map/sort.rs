//! `sort_album_items`: local toolbar sort over already-mapped card items.

use crate::AlbumCardItem;

/// Local sort over already-mapped card items, by the toolbar's sort key.
/// `newest`/`oldest` sort on `plain_year`; `title-*`/`artist-*` are
/// case-insensitive; any other key (e.g. `default`) leaves order intact.
pub fn sort_album_items(items: &mut [AlbumCardItem], sort: &str) {
    match sort {
        "oldest" | "year-asc" => {
            items.sort_by(|a, b| a.plain_year.as_str().cmp(b.plain_year.as_str()))
        }
        "newest" | "year-desc" => {
            items.sort_by(|a, b| b.plain_year.as_str().cmp(a.plain_year.as_str()))
        }
        "title-asc" => items.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
        "title-desc" => items.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase())),
        "artist-asc" => {
            items.sort_by(|a, b| a.artist.to_lowercase().cmp(&b.artist.to_lowercase()))
        }
        "artist-desc" => {
            items.sort_by(|a, b| b.artist.to_lowercase().cmp(&a.artist.to_lowercase()))
        }
        _ => {}
    }
}
