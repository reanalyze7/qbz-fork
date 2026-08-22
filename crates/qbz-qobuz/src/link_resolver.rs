//! Qobuz link resolver
//!
//! Parses Qobuz URLs (both `qobuzapp://` scheme and `https://play.qobuz.com/`)
//! into typed navigation actions. Pure function, no I/O, no Tauri dependency.

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod build;
mod parse;

use build::build_resolved_link;
use parse::{parse_path_segments, strip_web_prefix};

/// A resolved Qobuz link — tells the frontend which view to navigate to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum ResolvedLink {
    /// Navigate to album view. ID is a string (matches qbz-models Album.id).
    OpenAlbum(String),
    /// Navigate to track's album. ID is numeric.
    OpenTrack(u64),
    /// Navigate to artist view. ID is numeric.
    OpenArtist(u64),
    /// Navigate to playlist view. ID is numeric.
    OpenPlaylist(u64),
}

/// Errors that can occur when resolving a link.
#[derive(Debug, Clone, PartialEq, Error, Serialize, Deserialize)]
pub enum LinkResolverError {
    #[error("empty input")]
    EmptyInput,
    #[error("malformed URL")]
    MalformedUrl,
    #[error("unsupported scheme — expected qobuzapp:// or https://play.qobuz.com/")]
    UnsupportedScheme,
    #[error("unknown entity type: {0}")]
    UnknownEntityType(String),
    #[error("invalid ID: {0}")]
    InvalidId(String),
}

/// Resolve a Qobuz URL into a navigation action.
///
/// Accepted formats:
/// - `qobuzapp://album/<id>`
/// - `qobuzapp://track/<id>`
/// - `qobuzapp://artist/<id>`
/// - `qobuzapp://playlist/<id>`
/// - `https://play.qobuz.com/album/<id>`
/// - `http://play.qobuz.com/album/<id>` (auto-upgraded)
/// - Same patterns for track, artist, playlist
///
/// Query parameters, fragments, and trailing slashes are stripped.
pub fn resolve_link(url: &str) -> Result<ResolvedLink, LinkResolverError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(LinkResolverError::EmptyInput);
    }

    let (entity_type, raw_id) = if let Some(rest) = url.strip_prefix("qobuzapp://") {
        parse_path_segments(rest)?
    } else if let Some(rest) = strip_web_prefix(url) {
        parse_path_segments(rest)?
    } else {
        return Err(LinkResolverError::UnsupportedScheme);
    };

    build_resolved_link(&entity_type, &raw_id)
}

#[cfg(test)]
mod tests;
