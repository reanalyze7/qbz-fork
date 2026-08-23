//! Reco-dismissal axis actions.

use crate::AppWindow;

use super::build::push;

/// Undo one "Not interested" dismissal (optimistic — the re-push drops the row
/// immediately) + toast. The artist becomes eligible for the Recommendations
/// rails again on their next paint (the §B filter reads the store).
pub fn remove_dismissed(w: &AppWindow, artist_id: i32) {
    // Capture the name before removing, for the toast (falls back to the
    // generic "Artist" for rows persisted without a resolved name).
    let name = crate::reco_dismiss::list()
        .into_iter()
        .find(|a| a.artist_id == artist_id as u64)
        .map(|a| a.name)
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| qbz_i18n::t("Artist"));
    crate::reco_dismiss::remove(artist_id as u64);
    push(w);
    crate::toast::success(w, qbz_i18n::t_args("{} restored to Recommendations", &[&name]));
}
