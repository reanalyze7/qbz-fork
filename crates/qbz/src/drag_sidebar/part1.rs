use crate::*;

/// Type a LocalLibrary row for the drag payload: its real library row id.
pub(crate) fn local_drag_track(track: &qbz_library::LocalTrack) -> drag::DragTrack {
    drag::DragTrack::LocalRow(track.id)
}

/// Build a playlist-picker local-mode ref for a LocalLibrary row: its
/// library row id.
pub(crate) fn local_picker_ref(track: &qbz_library::LocalTrack) -> String {
    track.id.to_string()
}

/// Type a model row (Playlist / Artist surfaces) for the drag payload.
/// library row ids on `source == "local"` rows, Qobuz catalog ids on
/// everything else (incl. offline-cached rows). Render-only rows
/// ("file:"/"broken:" fallbacks) type to None and drop out of the drag.
pub(crate) fn row_drag_track(row: &TrackItem) -> Option<drag::DragTrack> {
    let id = row.id.to_string();
    if row.source.as_str() == "local" {
        return id.parse::<i64>().ok().map(drag::DragTrack::LocalRow);
    }
    id.parse::<u64>().ok().map(drag::DragTrack::Qobuz)
}

/// Resolve the SOURCE-TYPED track refs for a drag started on `track_id`
/// — the id namespace depends on the view the drag started in (Qobuz
/// surfaces carry catalog ids; LocalLibrary surfaces carry library row
/// ids). If the current view has a multi-selection
/// that includes the dragged row (and is >1), the whole selection is
/// dragged; otherwise just the row. Mirrors Tauri's group-drag rule.
pub(crate) fn gather_drag_tracks(w: &AppWindow, track_id: &str) -> Vec<drag::DragTrack> {
    use slint::Model;
    let view = w.global::<NavState>().get_view();
    match view {
        ContentView::LocalAlbum => {
            // Single-row surface; resolve through the open album's version
            // cache.
            local_library::current_album_version_tracks(w)
                .iter()
                .find(|t| t.id.to_string() == track_id)
                .map(|t| vec![local_drag_track(t)])
                .unwrap_or_default()
        }
        ContentView::LocalLibrary => {
            // Tracks tab (group-drag over the multi-selection first).
            let selected = local_library::selected_local_tracks(w);
            if selected.len() > 1 && selected.iter().any(|t| t.id.to_string() == track_id) {
                return selected.iter().map(local_drag_track).collect();
            }
            if let Some(track) = local_library::local_track_by_id(track_id) {
                return vec![local_drag_track(&track)];
            }
            // Folder-detail rows aren't in the Tracks cache but are real
            // library rows — type by row id (resolved at insert).
            track_id
                .parse::<i64>()
                .map(|id| vec![drag::DragTrack::LocalRow(id)])
                .unwrap_or_default()
        }
        ContentView::Playlist | ContentView::Artist => {
            let model = match view {
                ContentView::Playlist => w.global::<PlaylistState>().get_tracks(),
                _ => w.global::<ArtistState>().get_top_tracks(),
            };
            let rows: Vec<TrackItem> = (0..model.row_count())
                .filter_map(|i| model.row_data(i))
                .collect();
            let selected: Vec<drag::DragTrack> = rows
                .iter()
                .filter(|t| t.selected)
                .filter_map(row_drag_track)
                .collect();
            if selected.len() > 1 && rows.iter().any(|t| t.selected && t.id == track_id) {
                return selected;
            }
            if let Some(row) = rows.iter().find(|t| t.id == track_id) {
                return row_drag_track(row).map(|d| vec![d]).unwrap_or_default();
            }
            track_id
                .parse::<u64>()
                .map(|id| vec![drag::DragTrack::Qobuz(id)])
                .unwrap_or_default()
        }
        // Every other surface (album / search / favorites / mix / …) is
        // Qobuz-backed: rows carry catalog ids.
        _ => track_id
            .parse::<u64>()
            .map(|id| vec![drag::DragTrack::Qobuz(id)])
            .unwrap_or_default(),
    }
}

/// Load (or reload) the sidebar playlists list off-thread.
pub(crate) fn load_sidebar_playlists(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
) {
    let _ = weak.upgrade_in_event_loop(|w| sidebar::set_loading(&w, true));
    handle.spawn(async move {
        let data = sidebar::load(&runtime).await;
        let _ = weak.upgrade_in_event_loop(move |w| {
            sidebar::apply(&w, data);
            refresh_sidebar_covers(&w);
        });
    });
}

