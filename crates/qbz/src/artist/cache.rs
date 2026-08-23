//! Full (unfiltered) rendered models cached on the UI thread, shared by
//! `apply`, `search`, `sort_page`, and `favorites` (release-card flip). This
//! is the single source of truth for the four `thread_local!` caches so no
//! consumer file re-declares them.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{ArtistReleaseSection, TrackItem};

// Mirrors album::FULL_TRACKS.
thread_local! {
    pub(crate) static FULL_TOP_TRACKS: RefCell<Vec<TrackItem>> = RefCell::new(Vec::new());
    pub(crate) static FULL_APPEARS_ON: RefCell<Vec<TrackItem>> = RefCell::new(Vec::new());
    pub(crate) static FULL_RELEASE_SECTIONS: RefCell<Vec<ArtistReleaseSection>> =
        RefCell::new(Vec::new());
    // Per-release-type pages already loaded into the index (1 = the initial
    // /artist/page bucket). The index caps at MAX_INDEX_PAGES; beyond that
    // the dedicated discography page takes over.
    pub(crate) static LOADED_PAGES: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
}

/// Index-page load-more cap (the user asked EPs & Singles / Live to page
/// up to 4). Page 1 is the embedded bucket; 3 more loads reach the cap.
pub const MAX_INDEX_PAGES: u32 = 4;
