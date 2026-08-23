//! Plain, `Send` artist page data produced on the worker thread.

use crate::album::TrackData;
use crate::home::CardData;

/// Plain, `Send` artist data produced on the worker thread.
pub struct ArtistData {
    pub name: String,
    pub bio: String,
    /// Word-boundary truncated bio used at rest; the Read-more toggle
    /// swaps to `bio`. Equal to `bio` when the text fits in the cap.
    pub bio_short: String,
    pub bio_truncated: bool,
    /// Editorial source for the biography ("TiVo" etc). Empty when absent.
    pub bio_source: String,
    pub artwork_url: String,
    pub top_tracks: Vec<TrackData>,
    /// "Novedad más reciente" — the single highlighted latest release
    /// (`last_release` in /artist/page). None when the API omits it.
    pub last_release: Option<CardData>,
    /// "Appears On" (`tracks_appears_on`) — tracks where the artist guests,
    /// rendered as a flat track section (NOT albums).
    pub appears_on: Vec<TrackData>,
    /// Releases grouped into titled sections (Albums, EPs & Singles, ...).
    pub release_sections: Vec<ReleaseSection>,
    /// Labels collected from the artist's own album releases (deduped
    /// by id, sorted by name) — sidebar Labels section.
    pub labels: Vec<LabelData>,
    /// Similar artists from /artist/page — sidebar Similar Artists.
    pub similar_artists: Vec<SimilarArtistData>,
    /// Curated playlists featuring this artist (the /artist/page `playlists`
    /// section) — main-column Playlists carousel, above the "Other" block.
    pub playlists: Vec<PlaylistSlim>,
}

/// One curated playlist card for the artist Playlists carousel.
#[derive(Clone)]
pub struct PlaylistSlim {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub image_url: String,
}

#[derive(Clone)]
pub struct LabelData {
    pub id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct SimilarArtistData {
    pub id: String,
    pub name: String,
}

/// One titled group of artist releases.
pub struct ReleaseSection {
    /// Raw server `release_type` key (album/epSingle/live/…). Stable id
    /// for jump-tabs, sort persistence (Phase 2) and "see discography".
    pub release_type: String,
    pub title: String,
    /// Server `has_more` for this bucket — gates the per-section load-more.
    pub has_more: bool,
    pub cards: Vec<CardData>,
}

/// Official on-screen order of release buckets, with their display titles
/// (webplayer-faithful). `release_type` keys come straight from the server.
// Display titles are `mark`ed so the extractor registers the English literals;
// they are translated once with `t(...)` at the consumer sites (the section
// header, the jump-tab label, and `release_type_title`).
pub(crate) const RELEASE_SECTION_ORDER: &[(&str, &str)] = &[
    ("album", qbz_i18n::mark("Albums")),
    ("epSingle", qbz_i18n::mark("EPs & Singles")),
    ("ep", qbz_i18n::mark("EPs & Singles")),
    ("single", qbz_i18n::mark("EPs & Singles")),
    ("live", qbz_i18n::mark("Live")),
    ("compilation", qbz_i18n::mark("Compilations")),
    ("download", qbz_i18n::mark("Purchase Only")),
    ("composer", qbz_i18n::mark("Composer")),
    ("other", qbz_i18n::mark("Other")),
    ("awardedRelease", qbz_i18n::mark("Critics' Picks")),
    ("next", qbz_i18n::mark("Upcoming")),
];

/// Display title for a release_type (the dedicated discography page header).
pub fn release_type_title(release_type: &str) -> String {
    RELEASE_SECTION_ORDER
        .iter()
        .find(|(rt, _)| *rt == release_type)
        .map(|(_, title)| qbz_i18n::t(title))
        .unwrap_or_else(|| title_case(release_type))
}

/// Title-case a raw release_type key for unknown buckets (fallback only).
pub(crate) fn title_case(key: &str) -> String {
    let mut chars = key.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
