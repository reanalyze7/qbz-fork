//! Apple Music URL detection.

use super::{MusicProvider, MusicResource};

/// Detect if a URL is an Apple Music track, album, or playlist.
pub fn detect_resource(url: &str) -> Option<MusicResource> {
    if !url.contains("music.apple.com/") {
        return None;
    }

    // Playlist
    if parse_playlist_id(url).is_some() {
        return Some(MusicResource::Playlist {
            provider: MusicProvider::AppleMusic,
        });
    }

    // Song page (explicit song URL)
    if url.contains("/song/") {
        return Some(MusicResource::Track {
            provider: MusicProvider::AppleMusic,
            url: url.to_string(),
        });
    }

    // Album page — with ?i= parameter means specific track
    if url.contains("/album/") {
        if url.contains("?i=") || url.contains("&i=") {
            return Some(MusicResource::Track {
                provider: MusicProvider::AppleMusic,
                url: url.to_string(),
            });
        }
        return Some(MusicResource::Album {
            provider: MusicProvider::AppleMusic,
            url: url.to_string(),
        });
    }

    None
}

pub fn parse_playlist_id(url: &str) -> Option<(String, String)> {
    if !url.contains("music.apple.com/") {
        return None;
    }

    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 6 {
        return None;
    }

    let storefront = parts.get(3)?.to_string();
    let playlist_id = parts.last()?.split('?').next()?.to_string();

    if playlist_id.starts_with("pl.") || playlist_id.starts_with("pl.u-") {
        Some((storefront, playlist_id))
    } else {
        None
    }
}
