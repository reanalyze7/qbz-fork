//! Persisted data shapes for the recently-played store.

use serde::{Deserialize, Serialize};

/// One recently-played track, with the album it belongs to and enough
/// context (quality, ids) that re-playing it or rendering its album
/// card does not depend on a re-fetch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentTrack {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub artwork_url: String,
    #[serde(default)]
    pub album_id: String,
    #[serde(default)]
    pub album_title: String,
    #[serde(default)]
    pub album_artist: String,
    #[serde(default)]
    pub album_artwork_url: String,
    /// "hires" | "cd" | "" — drives the album card quality badge.
    #[serde(default)]
    pub quality_tier: String,
    /// "Hi-Res: 24-bit / 96 kHz" — quality badge hover tooltip.
    #[serde(default)]
    pub quality_label: String,
    /// Album genre, for the Recently Played album card overlay. Empty for
    /// entries recorded before genre capture (serde default).
    #[serde(default)]
    pub genre: String,
    /// Raw ISO album release date, localized to "MMM D, YYYY" at render.
    /// Empty for entries recorded before release-date capture.
    #[serde(default)]
    pub release_date: String,
    /// Artist id for navigation / scrobble context.
    #[serde(default)]
    pub artist_id: Option<u64>,
    /// Origin: "qobuz" | "local". Drives source-aware artwork
    /// (local file) and routing for the Recently Played cards.
    /// Empty for pre-source entries (serde default) → treated as "qobuz".
    #[serde(default)]
    pub source: String,
}

/// One recently-played album. Since #567 this is its OWN persisted history
/// (deduplicated by album id at record time, capped at `MAX_RECENT_ALBUMS`),
/// no longer derived from the 24-track window at read time. Every field takes
/// a serde default so entries written by older builds (or the legacy-derive
/// path) stay readable if fields are added later.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecentAlbum {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub artwork_url: String,
    #[serde(default)]
    pub quality_tier: String,
    #[serde(default)]
    pub quality_label: String,
    #[serde(default)]
    pub genre: String,
    /// Raw ISO release date; localized at render time.
    #[serde(default)]
    pub release_date: String,
    /// "qobuz" | "local" — see RecentTrack::source.
    #[serde(default)]
    pub source: String,
}

/// The persisted store shape since #567: the track history plus the separate
/// album history. The legacy shape (a bare `Vec<RecentTrack>` array) is
/// handled by `store_io::read_store`'s fallback branch.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(super) struct RecentStore {
    #[serde(default)]
    pub(super) tracks: Vec<RecentTrack>,
    #[serde(default)]
    pub(super) albums: Vec<RecentAlbum>,
}
