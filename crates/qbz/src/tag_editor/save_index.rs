//! The blocking write (sidecar or direct file tags) + DB index update.

use std::path::Path;

use slint::{ComponentHandle, Weak};

use qbz_library::LibraryError;

use crate::{AppWindow, TagEditorState};

use super::save_payload::SavePayload;

pub(super) async fn write_and_index(weak: Weak<AppWindow>, payload: SavePayload) -> Result<(), LibraryError> {
    let SavePayload {
        group_key,
        album_title,
        album_artist,
        album_dir,
        direct,
        year,
        genre_opt,
        catalog_opt,
        track_updates,
        tw_tracks,
        track_overs,
        album_over,
        tw_album,
    } = payload;

    tokio::task::spawn_blocking(move || {
        let dir = Path::new(&album_dir);
        if direct {
            qbz_library::write_album_tags_to_files(&tw_album, &tw_tracks, |cur, tot| {
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let s = w.global::<TagEditorState>();
                    s.set_write_progress_current(cur as i32);
                    s.set_write_progress_total(tot as i32);
                });
            })?;
            let _ = qbz_library::delete_album_sidecar(dir);
        } else {
            let sidecar = qbz_library::AlbumTagSidecar::new(album_over, track_overs);
            qbz_library::write_album_sidecar(dir, &sidecar)?;
        }
        // DB index update (transactional -> &mut db).
        crate::library_db::with_db_mut(|db| {
            let existing = db.get_album_tracks(&group_key)?;
            let m = qbz_library::compute_track_artist_match(&existing);
            db.update_album_group_metadata(
                &group_key,
                &album_title,
                &album_artist,
                year,
                genre_opt.as_deref(),
                catalog_opt.as_deref(),
                m.as_deref(),
                &track_updates,
            )
        })
        .ok_or_else(|| LibraryError::Database("library index update failed".to_string()))?;
        Ok(())
    })
    .await
    .unwrap_or_else(|e| Err(LibraryError::Other(format!("save task panicked: {e}"))))
}
