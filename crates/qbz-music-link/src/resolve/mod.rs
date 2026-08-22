//! Top-level music-link resolution orchestration.

mod via_odesli;

use crate::detection::{self, MusicResource};
use crate::errors::MusicLinkError;
use crate::odesli::SongLinkClient;
use crate::qobuz_search::MusicLinkResult;
use crate::QobuzSearchBridge;

use via_odesli::resolve_via_odesli_and_search;

/// Resolve a cross-platform music link to a Qobuz navigation action.
///
/// Accepts URLs from Qobuz, Spotify, Apple Music, Tidal, Deezer, song.link, and
/// album.link. For non-Qobuz tracks/albums, uses the Odesli API (or a direct
/// platform fast-path) to identify the content, then searches Qobuz by
/// title+artist to find the equivalent album. For playlists, returns
/// `PlaylistDetected` so the frontend can redirect to the importer.
pub async fn resolve_music_link(
    url: &str,
    songlink: &SongLinkClient,
    bridge: &dyn QobuzSearchBridge,
) -> Result<MusicLinkResult, MusicLinkError> {
    let url = url.trim().to_string();
    if url.is_empty() {
        return Err(MusicLinkError::EmptyUrl);
    }

    // 1. Try Qobuz native resolve first (sync, no network)
    if let Ok(resolved) = qbz_qobuz::resolve_link(&url) {
        return Ok(MusicLinkResult::Resolved {
            link: resolved,
            provider: None,
        });
    }

    // 2. Detect what kind of resource this is
    let resource = detection::detect_music_resource(&url).ok_or(MusicLinkError::Unsupported)?;

    match resource {
        MusicResource::Qobuz => {
            // Already handled above, but just in case
            let resolved = qbz_qobuz::resolve_link(&url)
                .map_err(|e| MusicLinkError::Internal(e.to_string()))?;
            Ok(MusicLinkResult::Resolved {
                link: resolved,
                provider: None,
            })
        }

        MusicResource::Playlist { provider } => Ok(MusicLinkResult::PlaylistDetected {
            provider: format!("{:?}", provider),
        }),

        MusicResource::Track {
            provider,
            url: source_url,
        } => {
            resolve_via_odesli_and_search(songlink, &source_url, Some(&provider), true, bridge).await
        }

        MusicResource::Album {
            provider,
            url: source_url,
        } => {
            resolve_via_odesli_and_search(songlink, &source_url, Some(&provider), false, bridge)
                .await
        }

        MusicResource::SongLink { url: source_url } => {
            // song.link URLs: try to detect track vs album from the URL format
            let is_track_hint = source_url.contains("song.link/");
            resolve_via_odesli_and_search(songlink, &source_url, None, is_track_hint, bridge).await
        }
    }
}
