//! Smart Qobuz search: progressively simpler queries until a match is found.
//!
//! Ported verbatim from the `src-tauri` link resolver, with the search calls
//! routed through the frontend-agnostic `QobuzSearchBridge`. The 4th
//! `search_type: Option<&str>` arg of the original (always `None`) is dropped
//! from the bridge signature.

mod search;
mod title;

pub(crate) use search::search_qobuz_smart;
#[allow(unused_imports)] // kept reachable at crate::qobuz_search::clean_title for tests/future callers
pub(crate) use title::clean_title;

/// Result of resolving a cross-platform music link.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind")]
pub enum MusicLinkResult {
    /// Successfully resolved to a Qobuz entity.
    Resolved {
        link: qbz_qobuz::ResolvedLink,
        provider: Option<String>,
    },
    /// The URL is a playlist — redirect to the Playlist Importer.
    PlaylistDetected { provider: String },
    /// The content exists on the source platform but is not available on Qobuz.
    NotOnQobuz { provider: Option<String> },
}
