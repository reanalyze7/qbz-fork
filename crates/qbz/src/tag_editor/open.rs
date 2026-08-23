//! Open the modal, seed state.

use slint::{ComponentHandle, ModelRc, VecModel, Weak};

use crate::{AppWindow, TagEditorState, TagTrackEdit};

/// Open the editor for a local album. Pre-fetches the album's tracks off-thread
/// (LocalTrack carries file_path/cue_* the AlbumState rows lack), then seeds +
/// opens on the UI thread. `group_key` and `directory_path` are equal for
/// folder-grouped local albums (the common case).
pub fn open_tag_editor(
    weak: Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    group_key: String,
    directory_path: String,
) {
    let gk = group_key.clone();
    handle.spawn(async move {
        let tracks = tokio::task::spawn_blocking(move || {
            crate::local_library::fetch_album_tracks_blocking(&gk)
        })
        .await
        .unwrap_or_default();
        let _ = weak.upgrade_in_event_loop(move |w| {
            populate(&w, group_key, directory_path, tracks);
        });
    });
}

pub(super) fn populate(
    w: &AppWindow,
    group_key: String,
    directory_path: String,
    tracks: Vec<qbz_library::LocalTrack>,
) {
    let album_title = tracks
        .first()
        .map(|t| {
            if !t.album_group_title.is_empty() {
                t.album_group_title.clone()
            } else {
                t.album.clone()
            }
        })
        .unwrap_or_default();
    let album_artist = qbz_library::compute_track_artist_match(&tracks).unwrap_or_default();
    let year = tracks
        .iter()
        .find_map(|t| t.year)
        .map(|y| y.to_string())
        .unwrap_or_default();
    let genre = tracks
        .iter()
        .find_map(|t| t.genre.clone().filter(|g| !g.trim().is_empty()))
        .unwrap_or_default();
    let catalog = tracks
        .iter()
        .find_map(|t| t.catalog_number.clone().filter(|c| !c.trim().is_empty()))
        .unwrap_or_default();
    let total_discs = tracks
        .iter()
        .filter_map(|t| t.disc_number)
        .max()
        .unwrap_or(1)
        .max(1) as i32;
    let can_direct = tracks
        .iter()
        .all(|t| t.cue_file_path.is_none() && t.cue_start_secs.is_none());

    let rows: Vec<TagTrackEdit> = tracks
        .iter()
        .map(|t| TagTrackEdit {
            id: t.id as i32,
            file_path: t.file_path.clone().into(),
            cue_file_path: t.cue_file_path.clone().unwrap_or_default().into(),
            cue_start_secs: t.cue_start_secs.unwrap_or(-1.0) as f32,
            has_cue: t.cue_file_path.is_some() || t.cue_start_secs.is_some(),
            title: t.title.clone().into(),
            disc_number: t.disc_number.map(|n| n.to_string()).unwrap_or_default().into(),
            track_number: t.track_number.map(|n| n.to_string()).unwrap_or_default().into(),
        })
        .collect();

    let s = w.global::<TagEditorState>();
    s.set_album_group_key(group_key.into());
    s.set_directory_path(directory_path.into());
    s.set_album_title(album_title.into());
    s.set_album_artist(album_artist.into());
    s.set_year_input(year.into());
    s.set_genre(genre.into());
    s.set_catalog_number(catalog.into());
    s.set_album_total_discs(total_discs);
    s.set_can_direct_write(can_direct);
    s.set_persistence_index(0);
    s.set_saving(false);
    s.set_write_progress_current(0);
    s.set_write_progress_total(0);
    s.set_tracks(ModelRc::new(VecModel::from(rows)));
    // Reset remote-lookup state.
    s.set_remote_provider_index(0);
    s.set_remote_searching(false);
    s.set_remote_loading(false);
    s.set_remote_results(ModelRc::new(VecModel::from(Vec::<crate::RemoteResultItem>::new())));
    s.set_selected_result_id("".into());
    s.set_show_remote_panel(false);
    s.set_has_searched(false);
    s.set_open(true);
}

pub fn close_tag_editor(weak: Weak<AppWindow>) {
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<TagEditorState>().set_open(false);
    });
}
