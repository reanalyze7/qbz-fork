use slint::ComponentHandle;

use crate::local_playlist::row::{build_row_models, total_duration_label, LoadedRow};
use crate::local_playlist::state::{CURRENT_META, CURRENT_QUEUE, ROW_POSITIONS};
use crate::{AppWindow, PlaylistState};

/// Apply the offline rows of a mixed Qobuz playlist (the cached snapshot
/// block + the sidecar block, see `navigate_qobuz_offline`'s merge rule)
/// into `PlaylistState`. Read-only header (`is_owner` false — Qobuz edits
/// can't run offline); playback flows through this module's queue snapshot,
/// the same machinery the LOCAL detail uses. UI thread.
pub(crate) fn apply_qobuz_offline(
    window: &AppWindow,
    playlist_id: u64,
    name: String,
    description: String,
    custom_artwork_path: Option<String>,
    rows: Vec<LoadedRow>,
) {
    let (queue, items, positions) = build_row_models(&rows);
    if let Ok(mut cur) = CURRENT_QUEUE.lock() {
        *cur = queue;
    }
    // NOT offline-only (D8 stamp stays off): this is a real Qobuz playlist
    // rendered partially.
    if let Ok(mut meta) = CURRENT_META.lock() {
        *meta = Some((playlist_id.to_string(), false));
    }
    if let Ok(mut pos) = ROW_POSITIONS.lock() {
        *pos = positions;
    }

    let duration = total_duration_label(&rows);
    let state = window.global::<PlaylistState>();
    state.set_id(playlist_id.to_string().into());
    state.set_name(name.into());
    state.set_owner(qbz_i18n::t("Available tracks only — offline").into());
    let description = crate::strip_html::strip_html(&description);
    state.set_description(description.clone().into());
    state.set_description_short(description.into());
    state.set_is_local(false);
    state.set_offline_only(false);
    state.set_offline_subset(true);
    // Read-only offline: Qobuz-side edits (rename / remove tracks / custom
    // order writes) can't reach the API, so the owner affordances hide.
    state.set_is_owner(false);
    // Card-tier decode (150px header cover), matching `apply`.
    let custom = custom_artwork_path
        .as_ref()
        .filter(|p| std::path::Path::new(p).exists())
        .and_then(|p| crate::artwork::load_local_cover(p, 264));
    if let Some(img) = custom {
        state.set_cover(img);
        state.set_cover_url(custom_artwork_path.unwrap_or_default().into());
        state.set_has_custom(true);
    } else {
        state.set_cover_url("".into());
        state.set_has_custom(false);
    }
    state.set_total_duration(duration.into());
    crate::playlist::apply_local_items(window, items);
}
