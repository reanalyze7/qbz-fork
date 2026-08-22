//! `QueueTrack` — the track shape carried by the playback queue.

use serde::{Deserialize, Serialize};

/// Track info stored in the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueTrack {
    pub id: u64,
    pub title: String,
    /// Subtitle/edition info from Qobuz (e.g. "Player's Ball Mix") that
    /// the frontend renders parenthesized after the title (issue #360).
    #[serde(default)]
    pub version: Option<String>,
    pub artist: String,
    pub album: String,
    /// Album-level release variant ("2009 Remaster", "Hi-Res", …) — distinct
    /// from the per-track `version`. Appended to the album name for the
    /// now-playing bar + MPRIS (NOT for Last.fm scrobbling, which wants the
    /// clean album name). Populated on the album-play path; None elsewhere.
    #[serde(default)]
    pub album_version: Option<String>,
    pub duration_secs: u64,
    pub artwork_url: Option<String>,
    #[serde(default)]
    pub hires: bool,
    pub bit_depth: Option<u32>,
    pub sample_rate: Option<f64>,
    /// Whether this is a local library track (not from streaming service)
    #[serde(default)]
    pub is_local: bool,
    /// Album ID for navigation
    pub album_id: Option<String>,
    /// Artist ID for navigation
    pub artist_id: Option<u64>,
    /// Whether the track is streamable (false = removed/unavailable)
    #[serde(default = "default_streamable")]
    pub streamable: bool,
    /// Source identifier (e.g., "qobuz", "local")
    #[serde(default)]
    pub source: Option<String>,
    /// Parental advisory / explicit content
    #[serde(default)]
    pub parental_warning: bool,
    /// Opaque identifier of the Mixtape/Collection item that produced this track,
    /// used by v2_skip_to_next_item / v2_skip_to_previous_item to detect boundaries.
    /// For non-Mixtape enqueue paths, set to the track's album_id so boundary
    /// detection still works for "play album" flows. None is a safe fallback
    /// (the skip commands fall back to album_id when this is absent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_item_id_hint: Option<String>,
    /// The container this track was launched FROM — the "playing from" origin
    /// used by the now-playing song-card "layers" button. `context_kind` is one
    /// of "album" | "artist" | "playlist" | "label"; `context_id` is that
    /// container's navigation id. Stamped per-track at enqueue time so the
    /// button always carries the CURRENT track's true source and is re-derived
    /// on every track change (never a stale single global). None = no container
    /// origin (bare single-track / favorites / mix / search play) → the button
    /// falls back to the track's own album. `serde(default)` keeps the persisted
    /// session-queue back-compatible (older payloads restore as None).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
}

pub(crate) fn default_streamable() -> bool {
    true
}
