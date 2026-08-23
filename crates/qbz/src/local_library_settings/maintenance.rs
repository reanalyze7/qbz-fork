//! Remove tracks whose files no longer exist. Reads paths, drops the DB
//! lock, stats outside the lock, then deletes in chunks (avoids the Tauri
//! under-lock stat stall). Inline status auto-clears after 3s.

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibraryFoldersState};

use super::load::load_folders;

pub fn cleanup_missing(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    {
        // Re-entry guard.
        if let Some(w) = weak.upgrade() {
            let s = w.global::<LibraryFoldersState>();
            if s.get_cleaning_missing() {
                return;
            }
            s.set_cleaning_missing(true);
            s.set_cleanup_status(qbz_i18n::t("Scanning track paths...").into());
        }
    }
    let h = handle.clone();
    handle.spawn(async move {
        let result = tokio::task::spawn_blocking(|| {
            let paths = crate::library_db::with_db(|db| db.get_all_track_paths())?;
            // Same guard as the scan's cleanup phase: a network folder whose
            // mount is DOWN right now stats as missing for every file — that
            // is "unreachable", not "deleted". Skip those subtrees so a
            // maintenance click while a share is unmounted can't wipe its
            // index.
            let skip: Vec<String> =
                crate::library_db::with_db(|db| db.get_folders_with_metadata())
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|f| f.is_network && std::fs::read_dir(&f.path).is_err())
                    .map(|f| {
                        if f.path.ends_with('/') {
                            f.path
                        } else {
                            format!("{}/", f.path)
                        }
                    })
                    .collect();
            let checked = paths.len();
            let missing: Vec<i64> = paths
                .into_iter()
                .filter(|(_, p)| {
                    !skip.iter().any(|pre| p.starts_with(pre.as_str()))
                        && !std::path::Path::new(p).exists()
                })
                .map(|(id, _)| id)
                .collect();
            let mut removed = 0usize;
            if !missing.is_empty() {
                for chunk in missing.chunks(500) {
                    removed += crate::library_db::with_db(|db| db.delete_tracks_by_ids(chunk))
                        .unwrap_or(0);
                }
            }
            Some((checked, removed))
        })
        .await
        .ok()
        .flatten();

        let (status, toast_ok) = match result {
            Some((checked, removed)) if removed > 0 => (
                qbz_i18n::t_args(
                    "Removed {} of {} tracks",
                    &[&removed.to_string(), &checked.to_string()],
                ),
                true,
            ),
            Some((checked, _)) => (
                qbz_i18n::t_args("Checked {} tracks - all OK", &[&checked.to_string()]),
                true,
            ),
            None => (qbz_i18n::t("Cleanup failed"), false),
        };

        if let Some(w) = weak.upgrade() {
            let s = w.global::<LibraryFoldersState>();
            s.set_cleaning_missing(false);
            s.set_cleanup_status(status.clone().into());
        }
        if toast_ok {
            crate::toast::success_weak(&weak, status);
        } else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't clean up missing files"));
        }

        // Auto-clear the inline status after 3s.
        let weak_clear = weak.clone();
        h.spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            let _ = weak_clear.upgrade_in_event_loop(|w| {
                w.global::<LibraryFoldersState>().set_cleanup_status("".into());
            });
        });
        load_folders(weak, h);
    });
}
