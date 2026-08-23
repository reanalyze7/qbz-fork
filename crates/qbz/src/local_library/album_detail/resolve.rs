//! Resolve the open album's cached versions (play / shuffle / tag-edit / the
//! per-disc header menu).

use slint::ComponentHandle;

use crate::AppWindow;

use super::state::album_versions;

/// The source directory of version `index` (for the tag editor — a real dir).
pub fn album_version_dir(index: i32) -> Option<String> {
    album_versions().get(index as usize).map(|(dir, _)| dir.clone())
}

/// The currently-selected album version's tracks (play / shuffle / add / edit).
pub fn current_album_version_tracks(window: &AppWindow) -> Vec<qbz_library::LocalTrack> {
    let idx = window.global::<crate::LocalAlbumState>().get_version_index();
    album_versions()
        .get(idx as usize)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

/// The current version's tracks for one disc (for the per-disc "Disc N" header
/// menu), filtered by `disc_number` (defaulting to 1 — exactly as
/// `apply_album_version` stamps the "Disc N" header). Preserves the upstream
/// (disc, track) order.
pub fn current_album_disc_tracks(
    window: &AppWindow,
    disc: i32,
) -> Vec<qbz_library::LocalTrack> {
    current_album_version_tracks(window)
        .into_iter()
        .filter(|t| t.disc_number.unwrap_or(1) as i32 == disc)
        .collect()
}
