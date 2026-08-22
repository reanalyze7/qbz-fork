//! Splits matched tracks into ≤2000-track parts, creating a Qobuz playlist
//! per part and chunked-adding its tracks with progress events.

use std::sync::Arc;

use qbz_qobuz::QobuzClient;

use crate::errors::PlaylistImportError;
use crate::models::ImportProgress;
use crate::sink::{ImportEvent, ImportPhase, ImportProgressSink};

use super::{ADD_CHUNK_SIZE, QOBUZ_PLAYLIST_TRACK_LIMIT};

/// Creates one Qobuz playlist per ≤2000-track part of `matched_track_ids`,
/// adding tracks in `ADD_CHUNK_SIZE` chunks with progress events, and
/// returns the created playlist ids in order.
pub(crate) async fn create_playlist_parts(
    client: &QobuzClient,
    matched_track_ids: &[u64],
    base_name: &str,
    description: Option<String>,
    is_public: bool,
    matched_count: u32,
    progress: Arc<dyn ImportProgressSink>,
) -> Result<Vec<u64>, PlaylistImportError> {
    let mut qobuz_playlist_ids = Vec::new();

    let parts: Vec<&[u64]> = matched_track_ids
        .chunks(QOBUZ_PLAYLIST_TRACK_LIMIT)
        .collect();
    let total_parts = parts.len();

    for (part_idx, part_tracks) in parts.iter().enumerate() {
        // Phase: creating (per part)
        progress.emit(ImportEvent::Phase(ImportPhase::Creating));

        let playlist_name = if total_parts == 1 {
            base_name.to_string()
        } else {
            format!("{} (Part {})", base_name, part_idx + 1)
        };

        let part_desc = if total_parts == 1 {
            description.clone()
        } else {
            Some(format!(
                "Part {} of {} — {}",
                part_idx + 1,
                total_parts,
                description.as_deref().unwrap_or("")
            ))
        };

        let created = client
            .create_playlist(&playlist_name, part_desc.as_deref(), is_public)
            .await
            .map_err(|e| PlaylistImportError::Qobuz(e.to_string()))?;

        qobuz_playlist_ids.push(created.id);

        // Phase: adding
        progress.emit(ImportEvent::Phase(ImportPhase::Adding));

        let chunks: Vec<&[u64]> = part_tracks.chunks(ADD_CHUNK_SIZE).collect();
        let total_chunks = chunks.len() as u32;

        for (i, chunk) in chunks.iter().enumerate() {
            client
                .add_tracks_to_playlist(created.id, chunk)
                .await
                .map_err(|e| PlaylistImportError::Qobuz(e.to_string()))?;

            progress.emit(ImportEvent::Progress(ImportProgress {
                phase: "adding".to_string(),
                current: (i as u32) + 1,
                total: total_chunks,
                matched_so_far: matched_count,
                current_track: if total_parts > 1 {
                    Some(format!("Part {}/{}", part_idx + 1, total_parts))
                } else {
                    None
                },
            }));
        }
    }

    Ok(qobuz_playlist_ids)
}
