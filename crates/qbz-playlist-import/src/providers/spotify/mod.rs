//! Spotify playlist import
//!
//! As of 2026-03-06, Spotify API access via client_credentials is no longer available.
//! All playlist imports now use the embed page scraping fallback.
//! Embed is limited to ~50 tracks and provides no ISRC or album data.

mod embed;
mod html;

pub use embed::{fetch_embed_metadata, fetch_playlist};

use crate::providers::{MusicProvider, MusicResource};

/// Detect if a URL is a Spotify track, album, or playlist.
pub fn detect_resource(url: &str) -> Option<MusicResource> {
    let lower = url.to_ascii_lowercase();
    if !lower.contains("spotify.com/") && !lower.starts_with("spotify:") {
        return None;
    }

    // Playlist check first (so parse_playlist_id takes priority for playlists)
    if parse_playlist_id(url).is_some() {
        return Some(MusicResource::Playlist {
            provider: MusicProvider::Spotify,
        });
    }

    // Track: open.spotify.com/track/<id> or spotify:track:<id>
    if lower.contains("/track/") || lower.contains(":track:") {
        return Some(MusicResource::Track {
            provider: MusicProvider::Spotify,
            url: url.to_string(),
        });
    }

    // Album: open.spotify.com/album/<id> or spotify:album:<id>
    if lower.contains("/album/") || lower.contains(":album:") {
        return Some(MusicResource::Album {
            provider: MusicProvider::Spotify,
            url: url.to_string(),
        });
    }

    None
}

pub fn parse_playlist_id(url: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix("spotify:playlist:") {
        if !rest.is_empty() {
            return Some(rest.to_string());
        }
    }

    let patterns = [
        "open.spotify.com/playlist/",
        "open.spotify.com/embed/playlist/",
    ];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_playlist_id_table() {
        let cases: &[(&str, Option<&str>)] = &[
            (
                "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M",
                Some("37i9dQZF1DXcBWIGoYBM5M"),
            ),
            (
                "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M",
                Some("37i9dQZF1DXcBWIGoYBM5M"),
            ),
            (
                "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=xyz&utm=1",
                Some("37i9dQZF1DXcBWIGoYBM5M"),
            ),
            (
                "https://open.spotify.com/embed/playlist/37i9dQZF1DXcBWIGoYBM5M",
                Some("37i9dQZF1DXcBWIGoYBM5M"),
            ),
            ("spotify:playlist:", None),
            ("https://open.spotify.com/playlist/", None),
            ("https://open.spotify.com/track/abc", None),
            ("https://example.com/", None),
        ];

        for (url, expected) in cases {
            assert_eq!(parse_playlist_id(url).as_deref(), *expected, "url: {}", url);
        }
    }

    #[test]
    fn detect_resource_track_album_playlist() {
        assert_eq!(
            detect_resource("https://open.spotify.com/playlist/abc"),
            Some(MusicResource::Playlist {
                provider: MusicProvider::Spotify
            })
        );
        assert!(matches!(
            detect_resource("https://open.spotify.com/track/abc"),
            Some(MusicResource::Track { .. })
        ));
        assert!(matches!(
            detect_resource("spotify:album:abc"),
            Some(MusicResource::Album { .. })
        ));
        assert_eq!(detect_resource("https://example.com/track/abc"), None);
    }
}
