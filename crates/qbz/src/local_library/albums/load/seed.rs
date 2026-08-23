//! Seed the four tab-badge counts up front (mirrors Favorites' seeded counts).

use slint::ComponentHandle;

use crate::{AppWindow, LocalLibraryState};

use crate::local_library::shared::exclude_network_folders_now;

use super::state::{current_group_mode, ALBUMS_FULL_LOAD_LIMIT};

/// Seed all four tab-badge counts up front (mirrors Favorites' seeded
/// counts) so the nav shows numbers without visiting each tab. Cheap:
/// bounded album/folder/artist reads + a `COUNT(*)` for the (potentially
/// huge) tracks table. Album/artist counts match each tab's own loader
/// exactly (same `get_albums_metadata_page` set; same
/// `normalize_artist` grouping the rail uses), so badges never jump when a
/// tab is opened.
pub fn seed_counts(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let group_mode = weak
        .upgrade()
        .map(|w| current_group_mode(&w))
        .unwrap_or(qbz_library::album_grouping::AlbumGroupMode::Folder);
    handle.spawn(async move {
        let counts: Option<(usize, usize, usize, usize)> = tokio::task::spawn_blocking(move || {
            // Same include-qobuz / network flags as the Albums tab loader, so
            // the badge always matches the grid.
            let exclude_network = exclude_network_folders_now();
            crate::library_db::with_db(|db| {
                // Total under the same filter the Albums tab uses, so the
                // badge matches the grid.
                let albums = db
                    .get_albums_metadata_page(
                        0,
                        ALBUMS_FULL_LOAD_LIMIT,
                        None,
                        "artist",
                        "asc",
                        true,
                        exclude_network,
                        group_mode,
                    )
                    .map(|p| p.total as usize)
                    .unwrap_or(0);
                let folders = db
                    .get_folders_with_metadata()
                    .map(|v| v.len())
                    .unwrap_or(0);
                let tracks = db.count_all_local_tracks().unwrap_or(0) as usize;
                // Exact rail count = distinct non-empty normalized names
                // (mirrors merge_artists' grouping key).
                let artists_raw = db.get_artists().unwrap_or_default();
                let mut seen = std::collections::HashSet::new();
                for a in &artists_raw {
                    let n = crate::local_library::artists::normalize_artist(&a.name);
                    if !n.is_empty() {
                        seen.insert(n);
                    }
                }
                Ok((albums, seen.len(), folders, tracks))
            })
        })
        .await
        .ok()
        .flatten();

        if let Some((albums, artists, folders, tracks)) = counts {
            let _ = weak.upgrade_in_event_loop(move |w| {
                let s = w.global::<LocalLibraryState>();
                s.set_album_count(albums as i32);
                s.set_artist_count(artists as i32);
                s.set_folder_count(folders as i32);
                s.set_track_count(tracks as i32);
            });
        }
    });
}
