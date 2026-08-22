//! Deezer URL detection.

use super::{MusicProvider, MusicResource};

/// Detect if a URL is a Deezer track, album, or playlist.
pub fn detect_resource(url: &str) -> Option<MusicResource> {
    if !url.contains("deezer.com") {
        return None;
    }

    // Playlist
    if parse_playlist_id(url).is_some() {
        return Some(MusicResource::Playlist {
            provider: MusicProvider::Deezer,
        });
    }

    let parts: Vec<&str> = url.split('/').collect();
    for (idx, part) in parts.iter().enumerate() {
        match *part {
            "track" => {
                if parts.get(idx + 1).map(|s| !s.is_empty()).unwrap_or(false) {
                    return Some(MusicResource::Track {
                        provider: MusicProvider::Deezer,
                        url: url.to_string(),
                    });
                }
            }
            "album" => {
                if parts.get(idx + 1).map(|s| !s.is_empty()).unwrap_or(false) {
                    return Some(MusicResource::Album {
                        provider: MusicProvider::Deezer,
                        url: url.to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    None
}

pub fn parse_playlist_id(url: &str) -> Option<String> {
    if !url.contains("deezer.com") {
        return None;
    }

    let parts: Vec<&str> = url.split('/').collect();
    for (idx, part) in parts.iter().enumerate() {
        if *part == "playlist" {
            let id = parts.get(idx + 1)?.split('?').next()?;
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }

    None
}
