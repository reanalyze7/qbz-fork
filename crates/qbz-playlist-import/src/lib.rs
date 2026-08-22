//! Public-playlist import for QBZ — provider scrapers, Qobuz matcher,
//! playlist creation (headless, frontend-agnostic).
//!
//! Extracted verbatim from `src-tauri/src/playlist_import/*` and re-typed
//! against the shared crates (`qbz-models`, `qbz-qobuz`), so it runs headless
//! with no Tauri state and no `#[tauri::command]` wrappers (ADR-005: no
//! legacy wrappers; ADR-006: frontend-agnostic core). Progress reaches the
//! frontend through the [`sink::ImportProgressSink`] trait instead of
//! `AppHandle::emit`.
//!
//! Known provider limitations (behavior-faithful copies of the Tauri code;
//! follow-up TODOs, not bugs introduced by the extraction):
//! - Spotify: embed scraping only (API access gone since 2026-03-06) — caps
//!   at ~50 tracks and provides no ISRC or album data.
//!   TODO: pagination/ISRC if a richer public source appears.
//! - Deezer: single public API call, no pagination — truncates around 400
//!   tracks. TODO: paginate `tracks.data`.
//! - Apple Music: scrapes `serialized-server-data` from the playlist page —
//!   the most fragile parser of the four.
//! - Tidal: fetches a fresh proxy token per playlist fetch (no caching/expiry
//!   handling). TODO: cache the token until expiry.
//! - Scrapers send no browser User-Agent (reqwest default) — TODO if any
//!   provider starts gating on UA.

pub mod errors;
pub mod importer;
pub mod match_qobuz;
pub mod models;
pub mod providers;
pub mod sink;

mod http;
mod provider_key;

pub use errors::PlaylistImportError;
pub use importer::{import_public_playlist, preview_public_playlist};
pub use models::{
    ImportPlaylist, ImportProgress, ImportProvider, ImportSummary, ImportTrack, TrackMatch,
};
pub use provider_key::{detect_provider_key, ProviderKey};
pub use providers::{detect_music_resource, MusicProvider, MusicResource};
pub use sink::{ImportEvent, ImportPhase, ImportProgressSink};

/// Cloudflare Workers proxy that holds the third-party API credentials.
/// Hoisted from the Tidal provider so the future link-resolver port shares
/// the one constant instead of duplicating it.
pub const QBZ_PROXY_BASE: &str = "https://qbz-api-proxy.blitzkriegfc.workers.dev";
