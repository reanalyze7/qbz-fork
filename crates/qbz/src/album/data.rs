//! Plain (non-Slint) album/track data types produced on the worker thread.

use qbz_models::Track;

/// One credited album artist for the header credit line (E1). Plain/`Send`.
pub struct ArtistCreditData {
    pub id: String,
    pub name: String,
    /// Localized role suffix ("" for the main artist(s)).
    pub role: String,
}

/// Plain, `Send` album data produced on the worker thread.
pub struct AlbumData {
    pub id: String,
    pub title: String,
    /// Primary interpreter (back-compat: now-playing, fallbacks).
    pub artist: String,
    pub artist_id: String,
    /// Full credited-artist list with roles for the header credit line.
    pub artists: Vec<ArtistCreditData>,
    /// Pre-formatted "year • label • genre • N tracks • duration".
    pub info_line: String,
    /// Meta-line segment BEFORE the label (the year) — rendered with the
    /// label as a clickable link in the header.
    pub meta_pre: String,
    /// Meta-line segment AFTER the label (genre • N tracks • duration).
    pub meta_post: String,
    pub quality_tier: String,
    /// "24-bit / 96 kHz" — the quality-badge detail line.
    pub quality_detail: String,
    /// Editorial description / review (HTML stripped). May be empty.
    pub description: String,
    /// Short, truncated description for the header (full text in a modal).
    pub description_short: String,
    /// Half-length truncation used when the content area is space-constrained.
    pub description_shorter: String,
    pub artwork_url: String,
    /// Record label name, for the sidebar (empty when unknown).
    pub label: String,
    /// Record label id, so the sidebar label card can navigate to the label
    /// page ("" when unknown).
    pub label_id: String,
    /// True when the album bundles a downloadable booklet/liner-notes PDF
    /// (Qobuz goodies) — gates the header booklet button.
    pub has_booklet: bool,
    /// URL of the booklet PDF goody (the controller downloads + rasterizes it
    /// on demand). Empty when the album bundles no booklet.
    pub booklet_url: String,
    pub tracks: Vec<TrackData>,
    /// Raw catalog tracks, kept for the multi-select bulk actions.
    pub raw_tracks: Vec<Track>,
}

pub struct TrackData {
    pub id: String,
    pub number: String,
    pub title: String,
    pub artist: String,
    /// Performer id for the clickable artist link ("" = plain text).
    pub artist_id: String,
    /// Album id for the clickable album link ("" = plain text). Album view
    /// leaves this empty (its rows belong to the album being viewed, so the
    /// apply layer stamps the viewed album's id); artist top-tracks set it
    /// per-track since they span different albums.
    pub album_id: String,
    /// Album TITLE for the row's album column ("" when unavailable). Album view
    /// leaves this empty (the apply layer stamps the viewed album's title);
    /// artist top-tracks fill it per-track since they span different albums.
    pub album: String,
    /// Album cover URL for the row thumbnail ("" when unavailable). Same
    /// ownership split as `album`/`album_id`: filled by artist top-tracks,
    /// left empty by the album view (whose rows share the header cover).
    pub artwork_url: String,
    pub duration: String,
    pub quality_tier: String,
    pub quality_detail: String,
    pub explicit: bool,
    /// Disc/media number (Qobuz `media_number`, defaulting to 1 when absent).
    /// Used after mapping to decide where the "Disc N" headers fall when the
    /// album spans more than one disc.
    pub disc: u32,
    /// Classical work TITLE (e.g. "Symphony No. 9"), or "" when the track
    /// carries no `work` metadata. Used after mapping to run-length stamp the
    /// per-work headers (PR #536). E3: the composer is split out below so the
    /// view can render its name as a clickable artist link.
    pub work: String,
    /// Work composer display name ("" when none); shown in the work header's
    /// parentheses as a link.
    pub work_composer_name: String,
    /// Work composer artist id ("" => the name renders as plain text).
    pub work_composer_id: String,
}
