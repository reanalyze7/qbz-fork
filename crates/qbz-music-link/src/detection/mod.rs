//! Music-URL detection and provider definitions.
//!
//! Ported from `src-tauri/src/playlist_import/providers/{mod,spotify,apple,
//! tidal,deezer}.rs`. Only the URL-detection logic and Spotify's embed-metadata
//! scrape are ported here — the heavy playlist-import machinery (`fetch_playlist`,
//! token plumbing, importer models) is intentionally left in `src-tauri`.

use serde::{Deserialize, Serialize};

pub mod apple;
pub mod deezer;
pub mod spotify;
pub mod tidal;

/// Which streaming platform a music link belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicProvider {
    Spotify,
    AppleMusic,
    Tidal,
    Deezer,
}

/// The kind of resource a music URL points to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicResource {
    /// A native Qobuz URL — resolve directly.
    Qobuz,
    /// A single track on a third-party platform.
    Track {
        provider: MusicProvider,
        url: String,
    },
    /// An album on a third-party platform.
    Album {
        provider: MusicProvider,
        url: String,
    },
    /// A playlist — should be redirected to the Playlist Importer.
    Playlist { provider: MusicProvider },
    /// A song.link / album.link / odesli.co URL — resolve via Odesli API.
    SongLink { url: String },
}

/// Detect what kind of music resource a URL points to.
///
/// Returns `None` for URLs that don't match any supported platform.
pub fn detect_music_resource(url: &str) -> Option<MusicResource> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    // 1. Qobuz — resolve_link() handles this natively
    if qbz_qobuz::resolve_link(url).is_ok() {
        return Some(MusicResource::Qobuz);
    }

    // 2. song.link / album.link / odesli.co URLs
    let lower = url.to_ascii_lowercase();
    if lower.contains("song.link/") || lower.contains("album.link/") || lower.contains("odesli.co/")
    {
        return Some(MusicResource::SongLink {
            url: url.to_string(),
        });
    }

    // 3. Per-provider detection (track/album/playlist)
    if let Some(resource) = spotify::detect_resource(url) {
        return Some(resource);
    }
    if let Some(resource) = apple::detect_resource(url) {
        return Some(resource);
    }
    if let Some(resource) = tidal::detect_resource(url) {
        return Some(resource);
    }
    if let Some(resource) = deezer::detect_resource(url) {
        return Some(resource);
    }

    None
}
