//! Persist the edits — the public entry point. Validation lives in
//! `save_build.rs`, payload building in `save_payload.rs`, and the async
//! confirm/write/apply flow in `save_write.rs`.

use slint::{ComponentHandle, Model, Weak};

use crate::{AppWindow, TagEditorState, TagTrackEdit};

use super::save_build::validate;
use super::save_payload::build_payload;
use super::save_write::run_save;

/// Persist the edits. Validates, gates the directory + CUE for direct mode,
/// confirms direct-write once, then writes (sidecar or files) + updates the DB
/// index on a blocking thread, and refreshes the open album.
pub fn save_tags(
    weak: Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: crate::artwork::ImageCache,
) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    let s = w.global::<TagEditorState>();

    let group_key = s.get_album_group_key().to_string();
    let album_title = s.get_album_title().trim().to_string();
    let album_artist = s.get_album_artist().to_string();
    let year_input = s.get_year_input().to_string();
    let genre = s.get_genre().to_string();
    let catalog = s.get_catalog_number().to_string();
    let directory_path = s.get_directory_path().to_string();
    let direct = s.get_persistence_index() == 1;

    let model = s.get_tracks();
    let rows: Vec<TagTrackEdit> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .collect();

    let Some((year, album_dir)) =
        validate(&weak, &group_key, &album_title, &year_input, direct, &rows)
    else {
        return;
    };

    let payload = build_payload(
        group_key,
        album_title,
        album_artist,
        album_dir,
        direct,
        year,
        &genre,
        &catalog,
        &rows,
    );

    let handle2 = handle.clone();
    handle.spawn(run_save(weak, handle2, image_cache, directory_path, payload));
}
