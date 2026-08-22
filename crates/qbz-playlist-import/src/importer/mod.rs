//! Orchestrates playlist import

use std::sync::Arc;

use qbz_qobuz::QobuzClient;

use crate::errors::PlaylistImportError;
use crate::match_qobuz::match_tracks;
use crate::models::{ImportPlaylist, ImportSummary};
use crate::providers::{detect_provider, fetch_playlist};
use crate::sink::{ImportEvent, ImportPhase, ImportProgressSink};

mod create_parts;

use create_parts::create_playlist_parts;

const ADD_CHUNK_SIZE: usize = 50;
const QOBUZ_PLAYLIST_TRACK_LIMIT: usize = 2000;

pub async fn preview_public_playlist(url: &str) -> Result<ImportPlaylist, PlaylistImportError> {
    let provider = detect_provider(url)?;
    fetch_playlist(provider).await
}

pub async fn import_public_playlist(
    url: &str,
    client: &QobuzClient,
    name_override: Option<&str>,
    is_public: bool,
    progress: Arc<dyn ImportProgressSink>,
) -> Result<ImportSummary, PlaylistImportError> {
    let playlist = preview_public_playlist(url).await?;

    // Phase: matching
    progress.emit(ImportEvent::Phase(ImportPhase::Matching));
    let matches = match_tracks(client, &playlist.tracks, Arc::clone(&progress)).await?;

    let mut matched_track_ids = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for entry in &matches {
        if let Some(id) = entry.qobuz_track_id {
            if seen.insert(id) {
                matched_track_ids.push(id);
            }
        }
    }

    let matched_count = matched_track_ids.len() as u32;
    let total_tracks = playlist.tracks.len() as u32;
    let skipped_tracks = total_tracks.saturating_sub(matched_count);

    let qobuz_playlist_ids = if !matched_track_ids.is_empty() {
        let base_name = name_override.unwrap_or(&playlist.name);
        let description = playlist
            .description
            .clone()
            .or_else(|| Some(format!("Imported from {}", playlist.provider.as_str())));

        create_playlist_parts(
            client,
            &matched_track_ids,
            base_name,
            description,
            is_public,
            matched_count,
            Arc::clone(&progress),
        )
        .await?
    } else {
        Vec::new()
    };

    let parts_created = qobuz_playlist_ids.len() as u32;

    Ok(ImportSummary {
        provider: playlist.provider,
        // Deliberate fix vs the Tauri original (owner decision): the summary
        // reports the name the playlist was actually created under — the
        // rename when one was given — not the original source name.
        playlist_name: match name_override {
            Some(name) => name.to_string(),
            None => playlist.name,
        },
        total_tracks,
        matched_tracks: matched_count,
        skipped_tracks,
        qobuz_playlist_ids,
        parts_created,
        matches,
    })
}
