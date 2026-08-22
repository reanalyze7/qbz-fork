//! Tidal URL detection/parsing — pure, no I/O.

use crate::providers::{MusicProvider, MusicResource};

/// Detect if a URL is a Tidal track, album, or playlist.
///
/// Tidal URLs:
/// - Track: `tidal.com/browse/track/{id}` or `tidal.com/track/{id}`
/// - Album: `tidal.com/browse/album/{id}` or `tidal.com/album/{id}`
/// - Playlist: `tidal.com/browse/playlist/{id}` or `tidal.com/playlist/{id}`
pub fn detect_resource(url: &str) -> Option<MusicResource> {
    if !url.contains("tidal.com") {
        return None;
    }

    // Playlist
    if parse_playlist_id(url).is_some() {
        return Some(MusicResource::Playlist {
            provider: MusicProvider::Tidal,
        });
    }

    let lower = url.to_ascii_lowercase();

    // Track
    if lower.contains("/track/") || lower.contains("/browse/track/") {
        return Some(MusicResource::Track {
            provider: MusicProvider::Tidal,
            url: url.to_string(),
        });
    }

    // Album
    if lower.contains("/album/") || lower.contains("/browse/album/") {
        return Some(MusicResource::Album {
            provider: MusicProvider::Tidal,
            url: url.to_string(),
        });
    }

    None
}

pub fn parse_playlist_id(url: &str) -> Option<String> {
    if !url.contains("tidal.com") {
        return None;
    }

    let patterns = ["/browse/playlist/", "/playlist/"];
    for pattern in patterns {
        if let Some(idx) = url.find(pattern) {
            let mut part = &url[idx + pattern.len()..];
            if let Some(end) = part.find('?') {
                part = &part[..end];
            }
            if !part.is_empty() {
                return Some(part.to_string());
            }
        }
    }

    None
}
