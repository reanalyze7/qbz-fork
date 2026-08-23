//! Multi-select removal for the open LOCAL playlist.

use qbz_library::local_playlists as repo;
use slint::{ComponentHandle, Model};

use super::state::ROW_POSITIONS;
use super::{is_local_id, Runtime};
use crate::artwork::ImageCache;
use crate::{AppWindow, PlaylistState};

/// Remove the selected rows from the open local playlist, then reload the
/// detail. UI thread entry; DB work off-thread.
pub fn remove_selected(
    window: &AppWindow,
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
) {
    let model = window.global::<PlaylistState>().get_tracks();
    let selected: Vec<String> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .map(|t| t.id.to_string())
        .collect();
    if selected.is_empty() {
        return;
    }
    crate::playlist::set_multi_select(window, false);
    remove_rows_by_ids(window, runtime, weak, handle, image_cache, selected);
}

/// Remove rows from the open LOCAL playlist by display id (repo positions
/// through the open snapshot's position map, removed highest first so each
/// removal's compaction never shifts a pending one), then reload. Shared
/// by the bulk Remove and the per-row "Remove from playlist" menu entry
/// (spec §3.1 step 4). UI thread entry; DB work off-thread.
pub fn remove_rows_by_ids(
    window: &AppWindow,
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    ids: Vec<String>,
) {
    let playlist_id = window.global::<PlaylistState>().get_id().to_string();
    if !is_local_id(&playlist_id) {
        return;
    }
    let mut positions: Vec<i32> = {
        let map = ROW_POSITIONS.lock().map(|m| m.clone()).unwrap_or_default();
        ids.iter().filter_map(|id| map.get(id).copied()).collect()
    };
    positions.sort_unstable_by(|a, b| b.cmp(a));
    if positions.is_empty() {
        return;
    }
    let handle2 = handle.clone();
    handle.spawn(async move {
        let pid = playlist_id.clone();
        tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| {
                Ok(db.with_connection(|conn| {
                    for pos in positions {
                        if let Err(e) = repo::remove_track(conn, &pid, pos) {
                            log::error!("[qbz-slint] local playlist remove pos {pos}: {e}");
                        }
                    }
                }))
            })
        })
        .await
        .ok();
        super::detail_local::navigate(runtime, weak, &handle2, image_cache, playlist_id);
    });
}
