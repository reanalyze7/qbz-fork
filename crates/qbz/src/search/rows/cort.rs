//! Plain (`Send`) row types for the cortinilla (live dropdown).

/// One plain (`Send`) cortinilla row, before it becomes a Slint
/// `CortinillaRow`. `source` selects the click seam ("qobuz" media/nav vs
/// "local" play); `kind` is the navigable category. `flat_index` is the stable
/// 0-based selection index across the WHOLE navigable list (top-result = 0,
/// then section rows in display order), assigned by `map_search_all_to_cortinilla`.
#[derive(Debug, Clone, PartialEq)]
pub struct CortRow {
    pub kind: String,
    pub id: String,
    pub source: String,
    pub title: String,
    pub subtitle: String,
    pub artwork_url: String,
    pub flat_index: usize,
}

/// One labelled cortinilla section (e.g. "Artists", "Albums").
#[derive(Debug, Clone, PartialEq)]
pub struct CortSection {
    pub title: String,
    pub kind: String,
    pub rows: Vec<CortRow>,
    pub has_more: bool,
}

/// The full cortinilla payload, as plain `Send` data.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CortinillaData {
    pub query: String,
    pub top: Option<CortRow>,
    pub sections: Vec<CortSection>,
}

/// How many rows each cortinilla category shows before "View more".
/// Per-category row caps in the cortinilla. Artists are rarely opened past the
/// first hit, so they get the smallest cap and the freed space goes to albums
/// (the most-scanned category). Tracks/playlists keep the default 3.
pub(crate) const CORTINILLA_CAP_ALBUMS: usize = 5;
pub(crate) const CORTINILLA_CAP_ARTISTS: usize = 2;
pub(crate) const CORTINILLA_CAP_TRACKS: usize = 3;
pub(crate) const CORTINILLA_CAP_PLAYLISTS: usize = 3;
