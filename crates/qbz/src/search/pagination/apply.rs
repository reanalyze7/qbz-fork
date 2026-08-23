use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::MoreRows;
use crate::search::apply::{album_item, artist_item, playlist_item, recompute_hi_res_filtered, track_item};
use crate::{AlbumCardItem, AppWindow, SearchPlaylistItem, SearchState, SlimItem, TrackItem};

/// Append fetched rows to the matching `SearchState` list. Pushes onto the
/// existing `VecModel` so already-loaded rows (and any resolved artwork)
/// are untouched. Runs on the Slint event loop.
pub fn append_results(window: &AppWindow, more: MoreRows) {
    let state = window.global::<SearchState>();
    match more {
        MoreRows::Albums(rows) => {
            if let Some(vm) = state
                .get_albums()
                .as_any()
                .downcast_ref::<VecModel<AlbumCardItem>>()
            {
                for row in rows {
                    vm.push(album_item(row));
                }
            }
        }
        MoreRows::Tracks(rows) => {
            if let Some(vm) = state
                .get_tracks()
                .as_any()
                .downcast_ref::<VecModel<TrackItem>>()
            {
                for row in rows {
                    vm.push(track_item(row));
                }
            }
        }
        MoreRows::Artists(rows) => {
            if let Some(vm) = state
                .get_artists()
                .as_any()
                .downcast_ref::<VecModel<SlimItem>>()
            {
                for row in rows {
                    vm.push(artist_item(row));
                }
            }
        }
        MoreRows::Playlists(rows) => {
            if let Some(vm) = state
                .get_playlists()
                .as_any()
                .downcast_ref::<VecModel<SearchPlaylistItem>>()
            {
                for row in rows {
                    vm.push(playlist_item(row));
                }
            }
        }
    }
    // Cheap even for the Artists/Playlists arms above (re-filters the
    // unchanged albums/tracks lists) — simpler than matching on `more`
    // twice, and this is a rare load-more click, not a hot path.
    recompute_hi_res_filtered(window);
}

/// Replace one category's `SearchState` list wholesale — used when the
/// searchType filter changes and the category is re-queried from offset 0.
pub fn replace_category(window: &AppWindow, more: MoreRows) {
    let state = window.global::<SearchState>();
    match more {
        MoreRows::Albums(rows) => {
            let items: Vec<AlbumCardItem> = rows.into_iter().map(album_item).collect();
            state.set_albums(ModelRc::new(VecModel::from(items)));
        }
        MoreRows::Tracks(rows) => {
            let items: Vec<TrackItem> = rows.into_iter().map(track_item).collect();
            state.set_tracks(ModelRc::new(VecModel::from(items)));
        }
        MoreRows::Artists(rows) => {
            // Rebuild both lists: the Artists tab keeps every result; the
            // All-tab carousel drops the duplicate next to the Most-popular
            // hero.
            let items: Vec<SlimItem> = rows.into_iter().map(artist_item).collect();
            let mp_id = if state.get_most_popular_kind().as_str() == "artist" {
                Some(state.get_most_popular_artist().id)
            } else {
                None
            };
            let carousel: Vec<SlimItem> = match (mp_id, items.first()) {
                (Some(id), Some(first)) if first.id == id.as_str() => items[1..].to_vec(),
                _ => items.clone(),
            };
            state.set_artists(ModelRc::new(VecModel::from(items)));
            state.set_artists_carousel(ModelRc::new(VecModel::from(carousel)));
        }
        MoreRows::Playlists(rows) => {
            let items: Vec<SearchPlaylistItem> = rows.into_iter().map(playlist_item).collect();
            state.set_playlists(ModelRc::new(VecModel::from(items)));
        }
    }
    recompute_hi_res_filtered(window);
}
