//! Render loaded playlists into `PlaylistPickerState` (see
//! `playlist_picker.rs` for the module split rationale). Split out purely to
//! keep `playlist_picker.rs` under the 130-line budget.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::playlist_picker::PickPlaylist;
use crate::{AppWindow, PlaylistPickItem, PlaylistPickerState};

pub fn apply(window: &AppWindow, playlists: Vec<PickPlaylist>) {
    let items: Vec<PlaylistPickItem> = playlists
        .into_iter()
        .enumerate()
        .map(|(i, p)| PlaylistPickItem {
            id: p.id.into(),
            name: p.name.into(),
            tracks_line: if p.tracks > 0 {
                qbz_i18n::tf("{} track", "{} tracks", p.tracks as i64, &[&p.tracks.to_string()])
                    .into()
            } else {
                "".into()
            },
            is_local: p.is_local,
            already_has: p.already_has,
            // No filter yet on (re)load — every row matches, ranked in
            // list order.
            filter_rank: i as i32,
        })
        .collect();
    let state = window.global::<PlaylistPickerState>();
    state.set_filter_matches(items.len() as i32);
    state.set_playlists(ModelRc::new(VecModel::from(items)));
    // Reset the filter affordance whenever the list is repopulated.
    state.set_filter("".into());
    state.set_loading(false);
}

/// Optimistically flip one row's checkbox after a successful toggle/add-all,
/// without a full reload. No-op if the picker isn't showing that row (e.g.
/// closed, or filtered — the model still holds it either way). UI thread.
pub fn mark_row_already_has(window: &AppWindow, playlist_id: &str, value: bool) {
    let model = window.global::<PlaylistPickerState>().get_playlists();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.id.as_str() == playlist_id {
                item.already_has = value;
                model.set_row_data(i, item);
                break;
            }
        }
    }
}
