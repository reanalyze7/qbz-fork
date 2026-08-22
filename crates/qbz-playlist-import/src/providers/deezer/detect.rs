//! Deezer URL detection — pure parsing, no I/O.

use crate::providers::{MusicProvider, MusicResource};

/// Detect if a URL is a Deezer track, album, or playlist.
///
/// Deezer URLs:
/// - Track: `deezer.com/track/{id}` or `deezer.com/{locale}/track/{id}`
/// - Album: `deezer.com/album/{id}` or `deezer.com/{locale}/album/{id}`
/// - Playlist: `deezer.com/playlist/{id}` or `deezer.com/{locale}/playlist/{id}`
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_playlist_id_table() {
        let cases: &[(&str, Option<&str>)] = &[
            (
                "https://www.deezer.com/playlist/1234567890",
                Some("1234567890"),
            ),
            (
                "https://www.deezer.com/en/playlist/1234567890",
                Some("1234567890"),
            ),
            (
                "https://www.deezer.com/en/playlist/1234567890?utm_source=x",
                Some("1234567890"),
            ),
            ("https://www.deezer.com/en/album/1234567890", None),
            ("https://www.deezer.com/en/playlist/", None),
            ("https://example.com/playlist/123", None),
        ];

        for (url, expected) in cases {
            assert_eq!(parse_playlist_id(url).as_deref(), *expected, "url: {}", url);
        }
    }

    #[test]
    fn detect_resource_track_album_playlist() {
        assert!(matches!(
            detect_resource("https://www.deezer.com/en/playlist/123"),
            Some(MusicResource::Playlist { .. })
        ));
        assert!(matches!(
            detect_resource("https://www.deezer.com/track/456"),
            Some(MusicResource::Track { .. })
        ));
        assert!(matches!(
            detect_resource("https://www.deezer.com/fr/album/789"),
            Some(MusicResource::Album { .. })
        ));
        assert_eq!(detect_resource("https://example.com/track/1"), None);
        assert_eq!(detect_resource("https://www.deezer.com/"), None);
    }
}
