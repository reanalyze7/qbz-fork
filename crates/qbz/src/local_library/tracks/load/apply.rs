//! Apply a fetched page (replace) or appended page (load-more) to the model.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, LocalLibraryState, TrackItem};

use crate::local_library::tracks::derive::derive_tracks;
use crate::local_library::tracks::map::map_local_track;

use super::state::tracks_current;

pub(crate) fn apply_tracks(window: &AppWindow, rows: Vec<qbz_library::LocalTrack>, has_more: bool) {
    // Keep the selection-source cache in lockstep (clone BEFORE the move).
    *tracks_current() = rows.clone();
    let items: Vec<TrackItem> = rows.into_iter().map(map_local_track).collect();
    let s = window.global::<LocalLibraryState>();
    let n = items.len() as i32;
    s.set_tracks(ModelRc::new(VecModel::from(items)));
    s.set_tracks_next_offset(n);
    s.set_tracks_has_more(has_more);
    s.set_tracks_loading(false);
    s.set_tracks_loading_more(false);
    s.set_tracks_load_failed(false);
    derive_tracks(window);
}

pub(crate) fn append_tracks(window: &AppWindow, rows: Vec<qbz_library::LocalTrack>, has_more: bool) {
    tracks_current().extend(rows.clone());
    let new_items: Vec<TrackItem> = rows.into_iter().map(map_local_track).collect();
    let s = window.global::<LocalLibraryState>();
    let model = s.get_tracks();
    let mut combined: Vec<TrackItem> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .collect();
    let added = new_items.len() as i32;
    combined.extend(new_items);
    s.set_tracks(ModelRc::new(VecModel::from(combined)));
    s.set_tracks_next_offset(s.get_tracks_next_offset() + added);
    s.set_tracks_has_more(has_more);
    s.set_tracks_loading_more(false);
    derive_tracks(window);
}
