//! Toolbar filter state (Rust-side source of truth).
//!
//! The rebuild runs on a tokio task and can't read Slint props cross-thread,
//! so the artist selection / sort / show-only-failed live here. The UI
//! actions update these + trigger a rebuild; the rebuild mirrors them back
//! onto OfflineManagerState for the dropdown / toggle / rail display.

use std::sync::{Mutex as StdMutex, OnceLock};

#[derive(Clone)]
pub(super) struct Filters {
    pub selected_artist: String, // "" = all
    pub sort: i32,               // 0 alpha / 1 recent / 2 largest / 3 smallest
    pub show_only_failed: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            selected_artist: String::new(),
            sort: 0,
            show_only_failed: false,
        }
    }
}

static FILTERS: OnceLock<StdMutex<Filters>> = OnceLock::new();

pub(super) fn filters() -> &'static StdMutex<Filters> {
    FILTERS.get_or_init(|| StdMutex::new(Filters::default()))
}

pub(super) fn current_filters() -> Filters {
    filters().lock().map(|f| f.clone()).unwrap_or_default()
}
