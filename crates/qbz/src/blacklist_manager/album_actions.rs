//! Album-axis actions: tab switch, block, remove, clear-all.

use crate::AppWindow;

use super::build::push;

/// Switch the manager's active tab (0 = Artists, 1 = Albums, 2 = Recommendations).
pub fn set_tab(w: &AppWindow, tab: i32) {
    w.global::<crate::BlacklistState>().set_active_tab(tab);
}

/// Block an album from a context menu (grid card / list row). Adds it and
/// re-pushes the manager state + count badges; the source grid drops the card
/// on its next navigation (no global observer — the artist-block convention).
pub fn block_album(w: &AppWindow, id: String, title: String, artist: String, cover: String) {
    if id.is_empty() {
        return;
    }
    match crate::artist_blacklist::add_album(&id, &title, &artist, &cover, None) {
        Ok(()) => {
            // If the blocked album is the one currently open, reflect the header
            // toggle immediately.
            let album_st = w.global::<crate::AlbumState>();
            if album_st.get_id().as_str() == id {
                album_st.set_is_album_blocked(true);
            }
            push(w);
            crate::toast::success(w, qbz_i18n::t_args("Album \"{}\" blocked", &[&title]));
        }
        Err(e) => {
            log::error!("[qbz-slint] album block failed: {e}");
            crate::toast::error(w, qbz_i18n::t("Failed to block album"));
        }
    }
}

/// Remove one album from the blacklist (optimistic re-push) + toast.
pub fn remove_album(w: &AppWindow, album_id: String) {
    let title = crate::artist_blacklist::get_all_albums()
        .into_iter()
        .find(|a| a.album_id == album_id)
        .map(|a| a.album_title)
        .unwrap_or_else(|| qbz_i18n::t("Album"));
    match crate::artist_blacklist::remove_album(&album_id) {
        Ok(()) => {
            let album_st = w.global::<crate::AlbumState>();
            if album_st.get_id().as_str() == album_id {
                album_st.set_is_album_blocked(false);
            }
            push(w);
            crate::toast::success(w, qbz_i18n::t_args("Album \"{}\" unblocked", &[&title]));
        }
        Err(e) => {
            log::error!("[qbz-slint] album remove failed: {e}");
            crate::toast::error(w, qbz_i18n::t("Failed to unblock album"));
        }
    }
}

/// Clear every blocked album + toast (count captured before).
pub fn clear_all_albums(w: &AppWindow) {
    let count = crate::artist_blacklist::album_count();
    match crate::artist_blacklist::clear_all_albums() {
        Ok(()) => {
            push(w);
            crate::toast::success(w, qbz_i18n::tf("Removed {} album from blacklist", "Removed {} albums from blacklist", count as i64, &[&count.to_string()]));
        }
        Err(e) => {
            log::error!("[qbz-slint] album clear-all failed: {e}");
            crate::toast::error(w, qbz_i18n::t("Failed to clear album blacklist"));
        }
    }
}
