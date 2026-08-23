use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::local_playlist::row::{LoadedRow, RowItem};

/// Row artwork jobs — Qobuz rows have http URLs, local rows file paths.
/// Returns (http, local-file) job sets targeting `PlaylistTrack{index}` (the
/// same target the Qobuz detail uses; indexes are FULL_ITEMS order).
pub fn artwork_jobs(rows: &[LoadedRow]) -> (Vec<ArtworkJob>, Vec<ArtworkJob>) {
    let mut http = Vec::new();
    let mut local = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        match &row.item {
            RowItem::Qobuz(track) => {
                if let Some(url) = track.album.as_ref().and_then(|a| a.image.smallest().cloned()) {
                    http.push(ArtworkJob {
                        url,
                        target: ArtworkTarget::PlaylistTrack { index },
                    });
                }
            }
            RowItem::Local(track) => {
                if let Some(path) = track.artwork_path.clone().filter(|p| !p.is_empty()) {
                    local.push(ArtworkJob {
                        url: path,
                        target: ArtworkTarget::PlaylistTrack { index },
                    });
                }
            }
            // Offline-resolved Qobuz rows: the cached cover.jpg loads through
            // the same local-file path as Local rows (B5).
            RowItem::Cached { artwork_path, .. } => {
                if let Some(path) = artwork_path.clone().filter(|p| !p.is_empty()) {
                    local.push(ArtworkJob {
                        url: path,
                        target: ArtworkTarget::PlaylistTrack { index },
                    });
                }
            }
            _ => {}
        }
    }
    (http, local)
}
