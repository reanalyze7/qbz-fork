//! Tidal URL detection.

use super::{MusicProvider, MusicResource};

/// Detect if a URL is a Tidal track, album, or playlist.
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
