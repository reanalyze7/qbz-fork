use slint::ComponentHandle;

use crate::local_playlist::row::{build_row_models, total_duration_label, LocalPlaylistData};
use crate::local_playlist::state::{CURRENT_META, CURRENT_QUEUE, ROW_POSITIONS};
use crate::{AppWindow, PlaylistState};

/// Apply loaded data into `PlaylistState` (header + rows through the shared
/// `playlist.rs` row machinery) and snapshot the playable queue. UI thread.
pub fn apply(window: &AppWindow, data: LocalPlaylistData) {
    let (queue, items, positions) = build_row_models(&data.rows);

    if let Ok(mut cur) = CURRENT_QUEUE.lock() {
        *cur = queue;
    }
    if let Ok(mut meta) = CURRENT_META.lock() {
        *meta = Some((data.id.clone(), data.offline_only));
    }
    if let Ok(mut pos) = ROW_POSITIONS.lock() {
        *pos = positions;
    }

    let duration = total_duration_label(&data.rows);
    let state = window.global::<PlaylistState>();
    state.set_id(data.id.into());
    state.set_name(data.name.into());
    state.set_owner(if data.offline_only {
        "Offline-only playlist"
    } else {
        "Local playlist"
    }
    .into());
    let description = crate::strip_html::strip_html(&data.description);
    state.set_description(description.clone().into());
    state.set_description_short(description.into());
    state.set_is_local(true);
    state.set_offline_only(data.offline_only);
    state.set_is_owner(true);
    // Custom artwork (local file) or the row-collage fallback. Decoded to the
    // card tier (the header cover renders at 150px).
    let custom = data
        .custom_artwork_path
        .as_ref()
        .filter(|p| std::path::Path::new(p).exists())
        .and_then(|p| crate::artwork::load_local_cover(p, 264));
    if let Some(img) = custom {
        state.set_cover(img);
        state.set_cover_url(data.custom_artwork_path.clone().unwrap_or_default().into());
        state.set_has_custom(true);
    } else {
        state.set_cover_url("".into());
        state.set_has_custom(false);
    }
    state.set_total_duration(duration.into());
    crate::playlist::apply_local_items(window, items);
}
