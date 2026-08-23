//! Playlist-add events (W4) — log the FULL requested Qobuz track ids (Tauri
//! parity; the deduped/inserted subset under-weights train()). Qobuz tracks
//! only — callers skip local-ref adds, whose ids don't resolve against Qobuz.
//! `playlist_id` is the Qobuz playlist id for a Qobuz target, else None (a new
//! or local playlist has no Qobuz id; the per-track signal is what scores).

use qbz_app::settings::reco_store::{RecoEventInput, RecoEventType, RecoItemType};

use super::RECO;

/// Log a playlist-add, one PlaylistAdd/Track event per requested id.
pub fn log_playlist_add(playlist_id: Option<u64>, track_ids: Vec<u64>) {
    if track_ids.is_empty() {
        return;
    }
    if let Ok(guard) = RECO.lock() {
        if let Some(store) = guard.as_ref() {
            for tid in track_ids {
                let ev = RecoEventInput {
                    event_type: RecoEventType::PlaylistAdd,
                    item_type: RecoItemType::Track,
                    track_id: Some(tid),
                    album_id: None,
                    artist_id: None,
                    playlist_id,
                    genre_id: None,
                };
                if let Err(e) = store.insert_event(&ev) {
                    log::warn!("[reco] log_playlist_add failed: {e}");
                    break;
                }
            }
        }
    }
}
