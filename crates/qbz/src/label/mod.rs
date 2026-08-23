//! Label views: `LabelReleasesView` (header + paginated album catalog) and
//! `LabelPageView` (the rich landing page: header + popular tracks +
//! releases/critics/playlists/artists/more-labels carousels).
//!
//! Mirrors Tauri's LabelReleasesView.svelte / LabelView.svelte data flow.

mod page;
mod releases;

use std::collections::HashSet;

pub use page::*;
pub use releases::*;

/// Snapshot both blacklist axes (artist ids + album ids), empty when the
/// feature is disabled. Label surfaces were entirely unfiltered before; this
/// closes both axes at once (the artist leak too).
pub(crate) fn bl_snapshots() -> (HashSet<u64>, HashSet<String>) {
    if crate::artist_blacklist::is_enabled() {
        (
            crate::artist_blacklist::ids_snapshot(),
            crate::artist_blacklist::album_ids_snapshot(),
        )
    } else {
        (HashSet::new(), HashSet::new())
    }
}

/// Extract the best URL from /label/page's flexible image value. It
/// can be a bare string or an object with mega/extralarge/large/...
/// keys (mirrors the Svelte extraction order). Reused by the favorites
/// Labels tab, whose wire `image` is a bare string per the Android DTO.
pub(crate) fn extract_label_image(image: Option<&serde_json::Value>) -> String {
    let Some(image) = image else {
        return String::new();
    };
    if let Some(s) = image.as_str() {
        return s.to_string();
    }
    for key in ["mega", "extralarge", "large", "thumbnail", "small"] {
        if let Some(s) = image.get(key).and_then(|v| v.as_str()) {
            return s.to_string();
        }
    }
    String::new()
}
