use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{AlbumTagSidecar, LocalTrack};

pub(super) fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub(super) fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Apply a per-album sidecar override to a scanned track, caching the sidecar
/// read per album group key. Mirrors Tauri's
/// `library_apply_sidecar_override_if_present`.
pub(super) fn apply_sidecar_override(
    track: &mut LocalTrack,
    cache: &mut HashMap<String, Option<AlbumTagSidecar>>,
) {
    let group_key = track.album_group_key.trim();
    if group_key.is_empty() {
        return;
    }
    let cached = cache.entry(group_key.to_string()).or_insert_with(|| {
        let album_dir = Path::new(group_key);
        if !album_dir.is_dir() {
            return None;
        }
        crate::tag_sidecar::read_album_sidecar(album_dir).unwrap_or(None)
    });
    if let Some(sidecar) = cached.as_ref() {
        crate::tag_sidecar::apply_sidecar_to_track(track, sidecar);
    }
}
