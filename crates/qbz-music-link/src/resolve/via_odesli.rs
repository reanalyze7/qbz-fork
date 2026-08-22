//! Non-Qobuz resolution path: platform fast-path / Odesli fallback + Qobuz search.

use crate::detection::MusicProvider as Provider;
use crate::errors::MusicLinkError;
use crate::odesli::{ContentType, SongLinkClient};
use crate::qobuz_search::{self, MusicLinkResult};
use crate::QobuzSearchBridge;
use crate::{fast_path};

/// Identify a cross-platform music URL and search Qobuz for the equivalent.
///
/// Fast path: for Tidal/Deezer calls the platform API directly; for Spotify
/// scrapes the embed page to get title+artist. Fallback: uses Odesli API (~2-3s).
/// Then searches Qobuz with progressively simpler queries.
pub(super) async fn resolve_via_odesli_and_search(
    songlink: &SongLinkClient,
    url: &str,
    provider: Option<&Provider>,
    is_track: bool,
    bridge: &dyn QobuzSearchBridge,
) -> Result<MusicLinkResult, MusicLinkError> {
    let provider_name = provider.map(|p| format!("{:?}", p));

    // 1. Get title + artist: try direct platform API first (fast), fall back to Odesli
    let (title, artist) = if let Some(prov) = provider {
        match fast_path::try_direct_platform_metadata(url, prov, is_track).await {
            Some(meta) => {
                log::info!(
                    "Link resolver: direct API resolved '{}' by '{}'",
                    meta.0,
                    meta.1
                );
                meta
            }
            None => {
                log::info!("Link resolver: direct API failed, falling back to Odesli");
                fetch_metadata_via_odesli(songlink, url).await?
            }
        }
    } else {
        // No provider (song.link URLs) — use Odesli
        fetch_metadata_via_odesli(songlink, url).await?
    };

    if title.is_empty() {
        return Ok(MusicLinkResult::NotOnQobuz {
            provider: provider_name,
        });
    }

    // 2. Search Qobuz with progressively simpler queries
    if let Some(result) =
        qobuz_search::search_qobuz_smart(bridge, &title, &artist, is_track, &provider_name).await?
    {
        return Ok(result);
    }

    log::info!(
        "Link resolver: '{}' by '{}' not found on Qobuz",
        title,
        artist
    );
    Ok(MusicLinkResult::NotOnQobuz {
        provider: provider_name,
    })
}

/// Fetch metadata from Odesli API (with one retry for transient errors).
async fn fetch_metadata_via_odesli(
    songlink: &SongLinkClient,
    url: &str,
) -> Result<(String, String), MusicLinkError> {
    let response = match songlink.get_by_url(url, ContentType::Track).await {
        Ok(r) => r,
        Err(first_err) => {
            log::warn!(
                "Link resolver: Odesli first attempt failed: {}, retrying...",
                first_err
            );
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            songlink
                .get_by_url(url, ContentType::Track)
                .await
                .map_err(|e| MusicLinkError::Internal(format!("Odesli API error: {}", e)))?
        }
    };

    let title = response.title.unwrap_or_default().trim().to_string();
    let artist = response.artist.unwrap_or_default().trim().to_string();
    Ok((title, artist))
}
