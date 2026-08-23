//! Delete-confirm + bulk-remove-selected handlers.

use slint::ComponentHandle;

use crate::artwork::ImageCache;
use crate::{AppWindow, MyQbzEditState, NavState};

use super::super::modal::close_modal;
use super::super::reload::reload;
use super::super::with_repo;

/// Delete-confirm: `repo::delete_collection` (CASCADE) -> navigate BACK (which
/// re-applies the previous grid entry, so the deleted row is gone) -> close.
/// "Failed to delete" toast on error.
pub fn delete(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle, id: String) {
    if id.is_empty() {
        close_modal(&weak);
        return;
    }
    super::super::modal::set_busy(&weak, true);
    handle.spawn(async move {
        let write_id = id.clone();
        let result = tokio::task::spawn_blocking(move || {
            with_repo(|conn| qbz_mixtape::repo::delete_collection(conn, &write_id))
        })
        .await
        .unwrap_or_else(|e| Err(format!("delete task panicked: {e}")));

        let _ = weak.upgrade_in_event_loop(move |w| {
            let es = w.global::<MyQbzEditState>();
            es.set_busy(false);
            match result {
                Ok(()) => {
                    es.set_open(false);
                    es.set_mode("".into());
                    // Clean up the deleted collection's persisted view-prefs key
                    // so it doesn't orphan in the store (spec 12 §18 / §11.3).
                    crate::myqbz_view_prefs::remove(&id);
                    // Navigate back: re-applies the previous grid entry, which
                    // re-lists collections from the DB (the deleted one is gone).
                    w.global::<NavState>().invoke_request_back();
                }
                Err(e) => {
                    log::warn!("[qbz-slint] myqbz_edit delete failed: {e}");
                    crate::toast::error(&w, qbz_i18n::t("Failed to delete"));
                }
            }
        });
    });
}

/// Bulk-remove the selected items from the collection (spec 12 §13.3). The
/// positions are removed **highest-first** so each `repo::remove_item`'s
/// position-compaction (spec 40 §3.10) never shifts a position we still have to
/// delete. After the batch: reload the detail (re-fetches the now-compacted
/// list), clear the selection, and toast "Removed {n}". A repo error per item is
/// logged and the batch continues; only a hard DB-unavailable surfaces the
/// "Failed to remove items" error.
pub fn remove_selected(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
    mut positions: Vec<i32>,
) {
    if id.is_empty() || positions.is_empty() {
        return;
    }
    // Highest position first (descending) so compaction is harmless.
    positions.sort_unstable_by(|a, b| b.cmp(a));
    let count = positions.len();
    handle.clone().spawn(async move {
        let write_id = id.clone();
        let result = tokio::task::spawn_blocking(move || {
            with_repo(|conn| {
                for pos in &positions {
                    if let Err(e) = qbz_mixtape::repo::remove_item(conn, &write_id, *pos) {
                        log::warn!(
                            "[qbz-slint] myqbz_edit remove_item({write_id}, {pos}) failed: {e}"
                        );
                    }
                }
                Ok(())
            })
        })
        .await
        .unwrap_or_else(|e| Err(format!("bulk-remove task panicked: {e}")));

        match result {
            Ok(()) => {
                let _ = weak.upgrade_in_event_loop(|w| {
                    crate::myqbz_detail::clear_selection(&w);
                });
                crate::toast::info_weak(
                    &weak,
                    qbz_i18n::tf("Removed {} item", "Removed {} items", count as i64, &[&count.to_string()]),
                );
                reload(&weak, &handle, &image_cache, id);
            }
            Err(e) => {
                log::warn!("[qbz-slint] myqbz_edit bulk-remove failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Failed to remove items"));
            }
        }
    });
}
