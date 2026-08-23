//! `apply` / `apply_local_items` / `artwork_jobs` — push loaded playlist
//! data into `PlaylistState`.

use std::sync::atomic::Ordering;

use qbz_models::Track;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::local_playlist::RowItem;
use crate::{AppWindow, PlaylistState, TrackItem};

use super::statics::{CURRENT, MIXED};
use crate::playlist::load::PlaylistData;
use crate::playlist::view_state::FULL_ITEMS;

pub fn apply(window: &AppWindow, data: PlaylistData) {
    // One row-identity contract with the LOCAL/offline details (E11):
    // Qobuz rows keep catalog ids, local rows their library row id —
    // built by the shared `build_row_models` so selection, drag, picker
    // refs and scroll restore behave identically across the connectivity
    // flip.
    let (queue, items, positions) = crate::local_playlist::build_row_models(&data.rows);
    let qobuz_tracks: Vec<Track> = data
        .rows
        .iter()
        .filter_map(|r| match &r.item {
            RowItem::Qobuz(track) => Some((**track).clone()),
            _ => None,
        })
        .collect();
    let mixed = data.rows.len() != qobuz_tracks.len();
    // Merged header counts (Tauri shows qobuz + local combined).
    let count = items.len() as i32;
    let duration = crate::local_playlist::total_duration_label(&data.rows);
    FULL_ITEMS.with(|cell| *cell.borrow_mut() = items.clone());
    if let Ok(mut cur) = CURRENT.lock() {
        *cur = qobuz_tracks;
    }
    MIXED.store(mixed, Ordering::Relaxed);
    if mixed {
        // Seam B: the mixed detail plays through local_playlist's queue
        // snapshot (source-aware QueueTracks); pure-Qobuz details keep the
        // CURRENT-cache path unchanged.
        crate::local_playlist::set_open_mixed_snapshot(&data.id, queue, positions);
    } else {
        crate::local_playlist::clear_open_snapshot();
    }
    // Custom artwork overrides the collage / server image. Load the
    // local file directly (it lives in the artwork cache on disk),
    // decoded to the card tier (the header cover renders at 150px).
    let custom = data
        .custom_artwork_path
        .as_ref()
        .filter(|p| std::path::Path::new(p).exists())
        .and_then(|p| crate::artwork::load_local_cover(p, 264));
    let state = window.global::<PlaylistState>();
    // Seed the pin state from the pinned store (Home "Pinned" section) —
    // before set_id, which moves data.id.
    state.set_pinned(crate::pinned::is_pinned("playlist", &data.id));
    state.set_id(data.id.into());
    state.set_name(data.name.into());
    state.set_owner(data.owner.into());
    state.set_description(data.description.into());
    state.set_description_short(data.description_short.into());
    if let Some(img) = custom {
        state.set_cover(img);
        state.set_cover_url(data.custom_artwork_path.clone().unwrap_or_default().into());
        state.set_has_custom(true);
    } else {
        state.set_cover_url(data.cover_url.into());
        state.set_has_custom(false);
    }
    state.set_tracks(ModelRc::new(VecModel::from(items)));
    state.set_track_count(count);
    state.set_total_duration(duration.into());
    state.set_loading(false);
}

/// Apply a prebuilt row list (the LOCAL-playlist detail path, which
/// resolves its rows from the local repo instead of a Qobuz fetch) into the
/// SAME per-view statics this module owns, so in-page search / sort /
/// multi-select / the artwork pipeline all work unchanged. Clears the Qobuz
/// `CURRENT` track cache — local playlists drive playback from
/// `crate::local_playlist`'s own queue snapshot. UI thread.
pub fn apply_local_items(window: &AppWindow, items: Vec<TrackItem>) {
    FULL_ITEMS.with(|cell| *cell.borrow_mut() = items.clone());
    if let Ok(mut cur) = CURRENT.lock() {
        cur.clear();
    }
    let state = window.global::<PlaylistState>();
    state.set_track_count(items.len() as i32);
    state.set_tracks(ModelRc::new(VecModel::from(items)));
    state.set_loading(false);
}

/// Artwork jobs for the loaded playlist — one per row plus the header
/// cover (resolved into PlaylistState.cover). Returns (http, local-file)
/// job sets: Qobuz rows carry http URLs, local sidecar rows file paths —
/// the same loader split the LOCAL detail uses.
pub fn artwork_jobs(data: &PlaylistData) -> (Vec<ArtworkJob>, Vec<ArtworkJob>) {
    let (mut http, local) = crate::local_playlist::artwork_jobs(&data.rows);
    // Skip the server-cover job when a local custom artwork is set
    // (it's already loaded in apply and cover_url holds a file path).
    if data.custom_artwork_path.is_none() && !data.cover_url.is_empty() {
        http.push(ArtworkJob {
            url: data.cover_url.clone(),
            target: ArtworkTarget::PlaylistCover,
        });
    }
    (http, local)
}
