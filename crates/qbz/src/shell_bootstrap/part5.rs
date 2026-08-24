use crate::*;

/// The shared genre-filter selection as the `Option<Vec<u64>>` the discover
/// endpoints take (None = no filter). Shared by the home re-fetch and the
/// DiscoverBrowse "View all" page.
pub(crate) fn current_genre_filter() -> Option<Vec<u64>> {
    // The RAW selection (parent or sub-genre id, exactly as toggled). Qobuz's
    // /discover/index honors sub-genre ids server-side — 1:1 with Tauri
    // discovery-v2, which sent getSelectedGenreIds() straight through and did
    // NOT narrow client-side. Sending a top-level ancestor instead silently
    // widened sub-genre selections back to their parent (#618-batch regression).
    let ids = genre_filter::selected_ids_for("discover");
    (!ids.is_empty()).then_some(ids)
}

