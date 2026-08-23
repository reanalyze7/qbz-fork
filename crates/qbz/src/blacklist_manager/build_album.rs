//! Album-axis filtered view-model build.

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::BlacklistedAlbumItem;

use super::state::{current_query, format_added};

/// Build the visible (filtered) blocked-album items from the full title-sorted
/// snapshot, applying the current query (matches album title OR artist name).
/// Also returns the cover-load jobs for rows that carry a cover URL (resolved
/// async; rows render the blind-eye fallback until the image lands).
pub(super) fn build_album_items() -> (Vec<BlacklistedAlbumItem>, i32, Vec<ArtworkJob>) {
    let all = crate::artist_blacklist::get_all_albums();
    let count = all.len() as i32;
    let query = current_query();
    let needle = query.trim().to_lowercase();

    let mut jobs: Vec<ArtworkJob> = Vec::new();
    let items: Vec<BlacklistedAlbumItem> = all
        .into_iter()
        .filter(|a| {
            needle.is_empty()
                || a.album_title.to_lowercase().contains(&needle)
                || a.artist_name.to_lowercase().contains(&needle)
        })
        .enumerate()
        .map(|(idx, a)| {
            let notes = a.notes.clone().unwrap_or_default();
            if !a.cover_url.is_empty() {
                jobs.push(ArtworkJob {
                    url: a.cover_url.clone(),
                    target: ArtworkTarget::BlacklistAlbum { idx },
                });
            }
            BlacklistedAlbumItem {
                album_id: a.album_id.into(),
                album_title: a.album_title.into(),
                artist_name: a.artist_name.into(),
                cover_url: a.cover_url.into(),
                cover: slint::Image::default(),
                added_at: a.added_at as i32,
                added_display: format_added(a.added_at).into(),
                has_notes: !notes.is_empty(),
                notes: notes.into(),
            }
        })
        .collect();

    (items, count, jobs)
}
