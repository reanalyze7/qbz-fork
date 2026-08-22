//! Apple Music URL detection/parsing — pure, no I/O.

use crate::providers::{MusicProvider, MusicResource};

/// Detect if a URL is an Apple Music track, album, or playlist.
///
/// Apple Music URLs:
/// - Track: `music.apple.com/{storefront}/album/{name}/{id}?i={track_id}`
/// - Album: `music.apple.com/{storefront}/album/{name}/{id}` (no `?i=`)
/// - Playlist: `music.apple.com/{storefront}/playlist/{name}/{pl.xxx}`
/// - Song: `music.apple.com/{storefront}/song/{name}/{id}`
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_playlist_id_table() {
        // Editorial pl. id
        assert_eq!(
            parse_playlist_id(
                "https://music.apple.com/us/playlist/top-100-global/pl.d25f5d1181894928af76c85c967f8f31"
            ),
            Some((
                "us".to_string(),
                "pl.d25f5d1181894928af76c85c967f8f31".to_string()
            ))
        );
        // User pl.u- id + query strip
        assert_eq!(
            parse_playlist_id("https://music.apple.com/mx/playlist/mias/pl.u-abc123?l=en"),
            Some(("mx".to_string(), "pl.u-abc123".to_string())),
        );
        // Album URL is not a playlist
        assert_eq!(
            parse_playlist_id("https://music.apple.com/us/album/abbey-road/1441164426"),
            None
        );
        // Too few path segments
        assert_eq!(
            parse_playlist_id("https://music.apple.com/us/playlist"),
            None
        );
        // Wrong host
        assert_eq!(
            parse_playlist_id("https://example.com/us/playlist/x/pl.123"),
            None
        );
    }

    #[test]
    fn detect_resource_song_album_track() {
        assert!(matches!(
            detect_resource("https://music.apple.com/us/song/hey-jude/1441164589"),
            Some(MusicResource::Track { .. })
        ));
        assert!(matches!(
            detect_resource("https://music.apple.com/us/album/abbey-road/1441164426?i=1441164589"),
            Some(MusicResource::Track { .. })
        ));
        assert!(matches!(
            detect_resource("https://music.apple.com/us/album/abbey-road/1441164426"),
            Some(MusicResource::Album { .. })
        ));
        assert!(matches!(
            detect_resource("https://music.apple.com/us/playlist/x/pl.123"),
            Some(MusicResource::Playlist { .. })
        ));
        assert_eq!(detect_resource("https://example.com/us/album/x/1"), None);
    }
}
