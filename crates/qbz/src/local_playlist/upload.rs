//! "Upload to Qobuz" migration (D8-aware): convert a non-offline-only local
//! playlist into a real Qobuz playlist.

use super::repo::{delete_blocking, get_blocking, get_tracks_blocking};
use super::Runtime;
use crate::artwork::ImageCache;
use crate::AppWindow;

/// Convert a non-offline-only local playlist into a real Qobuz playlist:
/// create it, add the Qobuz-source rows, attach local rows via the existing
/// mixed-playlist sidecar (`playlist_local_tracks`), then delete the local
/// entity. On any attach failure the local entity is KEPT so the user can
/// retry. Never reached for offline-only playlists (the UI hides the action
/// and this guards again).
pub fn upload_to_qobuz(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    playlist_id: String,
) {
    handle.clone().spawn(async move {
        let id = playlist_id.clone();
        let (header, rows) = match tokio::task::spawn_blocking({
            let id = id.clone();
            move || (get_blocking(&id), get_tracks_blocking(&id))
        })
        .await
        {
            Ok(pair) => pair,
            Err(_) => return,
        };
        let Some(header) = header else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load this playlist"));
            return;
        };
        if header.offline_only {
            log::warn!("[qbz-slint] upload_to_qobuz refused: {id} is offline-only (D8)");
            return;
        }
        if crate::offline_mode::engine().is_offline() {
            crate::toast::error_weak(&weak, qbz_i18n::t("You're offline — try again when connected"));
            return;
        }

        let desc = header.description.as_deref().filter(|d| !d.trim().is_empty());
        let created = match runtime.core().create_playlist(&header.name, desc, false).await {
            Ok(p) => p,
            Err(e) => {
                log::error!("[qbz-slint] upload to Qobuz: create failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't create the Qobuz playlist"));
                return;
            }
        };
        let new_id = created.id;

        // Qobuz rows -> real membership.
        let qobuz_ids: Vec<u64> = rows.iter().filter_map(|r| r.qobuz_track_id).collect();
        if !qobuz_ids.is_empty() {
            if let Err(e) = runtime.core().add_tracks_to_playlist(new_id, &qobuz_ids).await {
                // Leave BOTH entities in place — the user can retry; deleting
                // the local copy after a partial upload would lose data.
                log::error!("[qbz-slint] upload to Qobuz: add tracks failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Upload incomplete — local playlist kept"));
                return;
            }
        }

        // Local rows -> the existing mixed-playlist sidecar, positioned
        // after the Qobuz block (Tauri's append convention). The local
        // entity is deleted ONLY when the sidecar attach succeeds — on a
        // DB failure it stays so the user can retry.
        let local_paths: Vec<String> = rows.iter().filter_map(|r| r.local_path.clone()).collect();
        let qobuz_count = qobuz_ids.len();
        let id_for_delete = id.clone();
        let attached = tokio::task::spawn_blocking(move || {
            let ok = crate::library_db::with_db(|db| {
                for (i, path) in local_paths.iter().enumerate() {
                    match db.get_track_by_path(path)? {
                        Some(track) => {
                            db.add_local_track_to_playlist(
                                new_id,
                                track.id,
                                (qobuz_count + i) as i32,
                            )?;
                        }
                        None => {
                            log::warn!(
                                "[qbz-slint] upload to Qobuz: local row missing from library: {path}"
                            );
                        }
                    }
                }
                Ok(())
            })
            .is_some();
            if ok {
                delete_blocking(&id_for_delete);
            }
            ok
        })
        .await
        .unwrap_or(false);
        if !attached {
            // The Qobuz playlist exists with its Qobuz tracks, but the
            // local sidecar rows didn't attach — keep the local entity.
            log::error!("[qbz-slint] upload to Qobuz: sidecar attach failed — local playlist kept");
            crate::toast::error_weak(&weak, qbz_i18n::t("Upload incomplete — local playlist kept"));
            let weak2 = weak.clone();
            let r2 = runtime.clone();
            let h2 = handle.clone();
            let _ = weak.upgrade_in_event_loop(move |_w| {
                crate::load_sidebar_playlists(r2, weak2, &h2);
            });
            return;
        }

        crate::toast::success_weak(&weak, qbz_i18n::t("Playlist uploaded to Qobuz"));
        let weak2 = weak.clone();
        let r2 = runtime.clone();
        let h2 = handle.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            crate::load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
            crate::nav::record(crate::nav::NavEntry::Playlist(new_id.to_string()));
            crate::navigate_playlist(r2, weak2.clone(), &h2, image_cache, new_id.to_string());
            crate::update_nav_flags(&w);
        });
    });
}
