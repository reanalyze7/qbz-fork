//! Artist-axis actions: search, toggle-enabled, remove, clear-all.

use crate::AppWindow;

use super::build::push;
use super::state::set_query;

/// Search-as-you-type: store the query and re-push the filtered list. `count`
/// stays the full length (so the empty-vs-no-results split stays correct).
pub fn search_changed(w: &AppWindow, query: String) {
    set_query(query);
    push(w);
}

/// Toggle the global enable flag; on success re-read + re-push and info-toast.
/// On error, surface the wrapper's message (no state change).
pub fn toggle_enabled(w: &AppWindow) {
    let new_state = !crate::artist_blacklist::is_enabled();
    match crate::artist_blacklist::set_enabled(new_state) {
        Ok(()) => {
            push(w);
            let msg = if new_state {
                qbz_i18n::t("Blacklist enabled")
            } else {
                qbz_i18n::t("Blacklist disabled")
            };
            crate::toast::info(w, msg);
        }
        Err(e) => {
            log::error!("[qbz-slint] blacklist toggle-enabled failed: {e}");
            crate::toast::error(w, qbz_i18n::t("Failed to toggle blacklist"));
        }
    }
}

/// Remove one artist (optimistic — re-push drops the row immediately) + toast.
pub fn remove(w: &AppWindow, artist_id: i32) {
    // Capture the name before removing, for the toast.
    let name = crate::artist_blacklist::get_all()
        .into_iter()
        .find(|a| a.artist_id == artist_id as u64)
        .map(|a| a.artist_name)
        .unwrap_or_else(|| qbz_i18n::t("Artist"));
    match crate::artist_blacklist::remove(artist_id as u64) {
        Ok(()) => {
            push(w);
            crate::toast::success(w, qbz_i18n::t_args("{} removed from blacklist", &[&name]));
        }
        Err(e) => {
            log::error!("[qbz-slint] blacklist remove failed: {e}");
            crate::toast::error(w, qbz_i18n::t("Failed to remove artist"));
        }
    }
}

/// Clear every blacklisted artist + toast (the count is captured before).
pub fn clear_all(w: &AppWindow) {
    let count = crate::artist_blacklist::count();
    match crate::artist_blacklist::clear_all() {
        Ok(()) => {
            push(w);
            crate::toast::success(w, qbz_i18n::tf("Removed {} artist from blacklist", "Removed {} artists from blacklist", count as i64, &[&count.to_string()]));
        }
        Err(e) => {
            log::error!("[qbz-slint] blacklist clear-all failed: {e}");
            crate::toast::error(w, qbz_i18n::t("Failed to clear blacklist"));
        }
    }
}
