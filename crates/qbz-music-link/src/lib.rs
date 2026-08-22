//! Frontend-agnostic cross-platform music-link resolver.
//!
//! Extracted from `src-tauri/src/commands_v2/link_resolver.rs` so any frontend
//! (Tauri, Slint, TUI/CLI) can resolve music links without depending on
//! `src-tauri` (ADR-006). Accepts URLs from Qobuz, Spotify, Apple Music, Tidal,
//! Deezer, song.link, and album.link. For non-Qobuz tracks/albums it identifies
//! the content (direct platform API fast-path, else the Odesli API) and searches
//! Qobuz by title+artist to find the equivalent. For playlists it returns
//! `PlaylistDetected` so the frontend can redirect to its importer.
//!
//! The Qobuz search itself is decoupled via the [`QobuzSearchBridge`] trait —
//! the frontend implements it over its own core and passes a `&dyn` to
//! [`resolve_music_link`].

mod bridge;
mod detection;
mod errors;
mod fast_path;
mod odesli;
mod qobuz_search;
mod resolve;

// ── Public API surface ──

pub use bridge::QobuzSearchBridge;
pub use detection::{detect_music_resource, MusicProvider, MusicResource};
pub use errors::MusicLinkError;
pub use odesli::{ContentType, ShareError, SongLinkClient};
pub use qobuz_search::MusicLinkResult;
pub use resolve::resolve_music_link;

// Re-export the native Qobuz parser so frontends can do native parsing too,
// without taking a direct `qbz-qobuz` dependency just for this.
pub use qbz_qobuz::{resolve_link, LinkResolverError, ResolvedLink};
