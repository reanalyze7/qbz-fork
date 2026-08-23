//! Shared album → card mapping for every Qobuz album surface (label
//! releases, favorites albums, and any toolbar-driven album list).
//!
//! Owns the V2-nested-first-with-flat-fallback decode so the list-row
//! extras (TYPE / QUALITY / TRACKS / YEAR columns of `AlbumListRow`)
//! populate consistently, the quality-tier classification, and the
//! local sort used by the grid/list toolbar views. Both `label.rs` and
//! `favorites.rs` map through here so there is one implementation to
//! maintain.

mod map;
mod sort;
mod tier;
mod to_item;

pub use map::map_album;
pub use sort::sort_album_items;
pub use tier::{classify_release_type, tier, tier_hires};
pub use to_item::to_item;

/// Plain album card — every field an `AlbumCard`/`AlbumListRow` can show.
#[derive(Clone)]
pub struct AlbumCard {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub genre: String,
    pub year: String,
    pub quality_tier: String,
    pub quality_label: String,
    pub artwork_url: String,
    // Qobuz label id ("" when the album surface carries no label object) —
    // feeds the per-label library index behind the LabelPage catalog/library
    // toggle. Not rendered by any card.
    pub label_id: String,
    // List-row extras (AlbumListRow columns; ignored by the grid card).
    pub release_type: String,   // "Album" | "EP" | "Single" (TYPE column)
    // "local" | "qobuz_download" | "" — SOURCE column + the
    // always-visible source badge on the Local Library grid card.
    pub source: String,
    pub quality_detail: String, // "24-bit / 96 kHz"
    pub track_count: String,    // "12"
    pub plain_year: String,     // "1973"
}

/// Append a release `version` to a title the way the Qobuz web player does:
/// `Octavarium (2009 Remaster)`, `A Dramatic Turn Of Events (Hi-Res)`. No-op
/// when the version is absent/empty. Used by every album-title surface
/// (discography, search, album header, suggestions) so re-editions of the same
/// album are distinguishable instead of rendering as identical duplicates.
pub fn format_album_title(title: &str, version: Option<&str>) -> String {
    match version.map(str::trim).filter(|v| !v.is_empty()) {
        Some(v) => format!("{title} ({v})"),
        None => title.to_string(),
    }
}
