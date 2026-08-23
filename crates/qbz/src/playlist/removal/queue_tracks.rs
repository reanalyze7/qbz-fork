//! The selected rows as ready-to-enqueue `QueueTrack`s (bulk Play next /
//! Add to queue, spec §1.5).

use qbz_models::QueueTrack;
use slint::{ComponentHandle, Model};

use crate::{AppWindow, PlaylistState};

/// The selected rows as ready-to-enqueue, SOURCE-AWARE QueueTracks in
/// visible order — the bulk Play next / Add to queue (spec §1.5). Rows of
/// a snapshot-backed detail (local / offline subset / online mixed)
/// resolve through `local_playlist`'s merged queue snapshot, which keeps
/// each row's source (local/cached — the T2 fix-forward: Tauri's
/// bulk path rebuilds catalog tracks and drops `source`); pure-Qobuz
/// details resolve through the loaded `CURRENT` Track cache. Unplayable
/// rows (file:/broken:/unresolved) drop out. UI thread.
pub fn selected_queue_tracks(window: &AppWindow) -> Vec<QueueTrack> {
    let model = window.global::<PlaylistState>().get_tracks();
    let qobuz = super::super::apply::current_tracks();
    let mut out: Vec<QueueTrack> = Vec::new();
    for i in 0..model.row_count() {
        let Some(item) = model.row_data(i) else {
            continue;
        };
        if !item.selected {
            continue;
        }
        let id = item.id.to_string();
        // Snapshot first (covers ALL sources of the mixed/local detail;
        // empty for pure-Qobuz details, see clear_open_snapshot).
        if let Some(qt) = crate::local_playlist::queue_track_for_row(&id) {
            out.push(qt);
            continue;
        }
        // Pure-Qobuz detail: build from the loaded Track cache.
        if let Some(track) = id
            .parse::<u64>()
            .ok()
            .and_then(|tid| qobuz.iter().find(|t| t.id == tid))
        {
            let (album_id, album_title, album_artwork) = track
                .album
                .as_ref()
                .map(|a| {
                    (
                        a.id.clone(),
                        a.title.clone(),
                        a.image.best().cloned().unwrap_or_default(),
                    )
                })
                .unwrap_or_default();
            let album_artist = track
                .performer
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_default();
            out.push(crate::playback::make_queue_track(
                track,
                &album_id,
                &album_title,
                &album_artist,
                &album_artwork,
                None,
            ));
        } else {
            log::warn!("[qbz-slint] bulk queue: row {id} not resolvable — skipped");
        }
    }
    out
}
