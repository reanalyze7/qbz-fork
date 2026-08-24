//! Cross-tab helpers used by more than one of the Albums/Tracks/Folders/
//! Artists submodules — kept here instead of duplicated per-tab.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AlbumCardItem, AppWindow, LocalArtistItem, LocalLibraryState, TrackItem};

// NETWORK-FOLDER VISIBILITY (owner verdict 2026-06-10, refined same day):
// hiding network-folder content is keyed on RAW CONNECTIVITY, never on the
// offline MODE. A logged-out session or induced offline with the link up
// says nothing about LAN mounts — content stays visible there (offline mode
// exists precisely to use the local library). Only a CONFIRMED-down link
// (hard offline: no default route / probes dead) hides network folders, the
// one state where LAN mounts are gone too. An unmounted-while-online path is
// handled at PLAYBACK time instead (existence guard + friendly toast in
// playback.rs), not by hiding library content.
//
// Known approximation, accepted with the model: an ISP outage with a live
// LAN reads as hard offline and hides NAS content; per-mount accessibility
// checks in every browse query would be the exact-but-costly alternative.

/// True only under HARD offline (connectivity confirmed down). See the
/// NETWORK-FOLDER VISIBILITY note above.
pub(crate) fn exclude_network_folders_now() -> bool {
    crate::offline_mode::engine().status().connectivity
        == qbz_app::offline_mode::Connectivity::Down
}

/// Reset the four browse-tab models so each tab re-fetches on its next visit
/// (the `ensure_*_loaded` guards key on an empty model). Used after a scan,
/// after the danger-zone clear, and on offline-mode flips (where the
/// connectivity-keyed network-folder gate may change the browse SET — see
/// the NETWORK-FOLDER VISIBILITY note above).
pub fn reset_browse_models(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let empty_albums = ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new()));
    let empty_tracks = ModelRc::new(VecModel::from(Vec::<TrackItem>::new()));
    s.set_albums(empty_albums.clone());
    s.set_folders(empty_albums);
    s.set_tracks(empty_tracks);
    s.set_artists(ModelRc::new(VecModel::from(Vec::<LocalArtistItem>::new())));
}

/// Synthetic-id namespace floor for legacy non-catalog track rows: `2^40`.
/// Offsetting synthetic ids into `[2^40, 2^41)` keeps them clear of local ids
/// (`< 2^40`) AND of ephemeral ids (`>= 2^48 = EPHEMERAL_ID_FLOOR`), so
/// `is_ephemeral_id` still returns false. Used as a guard against legacy
/// mis-typed rows (a stale garbage class from a since-removed integration)
/// that must never resolve as a Qobuz catalog id.
pub(crate) const LEGACY_SYNTHETIC_ID_FLOOR: u64 = 1 << 40;

/// Fetch an album's tracks by group key, trying the metadata grouping first
/// (Albums tab) then the folder grouping (Folders tab). Blocking.
pub fn fetch_album_tracks_blocking(group_key: &str) -> Vec<qbz_library::LocalTrack> {
    crate::library_db::with_db(|db| {
        let meta = db.get_album_tracks_metadata(group_key)?;
        if !meta.is_empty() {
            return Ok(meta);
        }
        db.get_album_tracks(group_key)
    })
    .unwrap_or_default()
}

/// First-letter bucket key for alpha grouping (`#` for non-alphabetic).
/// Mirrors favorites' `album_alpha_key` so the two surfaces sort identically.
pub(crate) fn folder_alpha_key(title: &str) -> String {
    title
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| {
            let up = c.to_uppercase().next().unwrap_or(c);
            if up.is_ascii_digit() {
                "#".to_string()
            } else {
                up.to_string()
            }
        })
        .unwrap_or_else(|| "#".to_string())
}

