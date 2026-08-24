//! First-class LOCAL playlists — Slint glue (offline-mode port, D7/D8).
//!
//! Storage lives in the shared per-user `library.db`
//! (`qbz_library::local_playlists`, ids `local:<uuid>`). This module routes
//! everything id-prefixed `local:` away from the Qobuz-bound playlist paths:
//! the detail view renders from the local repo through the SAME
//! `PlaylistState` + `playlist.rs` row machinery (search / sort /
//! multi-select / artwork reuse), playback builds `QueueTrack`s from the
//! resolvable rows, and an offline-only playlist stamps the queue
//! (`QbzCore::set_queue_offline_only`) so the QConnect push site skips the
//! cloud (D8: NOTHING from an offline-only playlist ever reaches Qobuz).

mod detail_local;
mod detail_offline_mixed;
mod enqueue;
mod playback;
mod remove;
mod reorder;
mod repo;
mod row;
mod state;
mod upload;

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;

pub(crate) type Runtime = Arc<AppRuntime<SlintAdapter>>;

/// Type guard (D7): a playlist reference is EITHER a Qobuz `u64` id or a
/// `local:<uuid>` string — Qobuz-bound calls take `u64` only, so a Local ref
/// is unrepresentable there by construction.
#[derive(Debug, Clone)]
pub enum PlaylistRef {
    Qobuz(u64),
    Local(String),
}

impl PlaylistRef {
    pub fn parse(id: &str) -> Option<Self> {
        if qbz_library::local_playlists::is_local_playlist_id(id) {
            Some(Self::Local(id.to_string()))
        } else {
            id.parse::<u64>().ok().map(Self::Qobuz)
        }
    }
}

/// True when `id` names a local playlist.
pub fn is_local_id(id: &str) -> bool {
    qbz_library::local_playlists::is_local_playlist_id(id)
}

// Re-export the full public surface so `crate::local_playlist::X` paths are
// unchanged for every caller across the `qbz` crate.
pub use detail_local::{artwork_jobs, navigate, read_sidecar_rows_blocking};
pub use detail_offline_mixed::navigate_qobuz_offline;
pub use enqueue::enqueue_by_id;
pub use playback::{play_all, play_from_visible};
pub use remove::{remove_rows_by_ids, remove_selected};
pub use reorder::{move_row, reorder_row};
pub use repo::{
    add_drag_tracks_blocking, add_local_refs_blocking, add_qobuz_tracks_blocking,
    clear_custom_artwork_blocking, create_blocking, delete_blocking,
    get_tracks_blocking, list_blocking, resolve_cover_urls, set_custom_artwork_blocking,
    set_favorite_blocking, set_hidden_blocking, update_blocking,
};
pub(crate) use repo::local_row_input;
pub use row::{LoadedRow, RowItem};
pub(crate) use row::{build_row_models, row_queue_track, total_duration_label};
pub use state::{
    clear_open_snapshot, local_picker_ref_for_row, queue_track_for_row, set_open_mixed_snapshot,
};
pub use upload::upload_to_qobuz;
