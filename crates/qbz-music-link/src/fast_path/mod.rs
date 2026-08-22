//! Direct platform metadata (bypass Odesli for speed).
//!
//! Ported verbatim from the `src-tauri` link resolver. For Tidal/Deezer this
//! calls the platform API directly; for Spotify it scrapes the embed page.
//! Apple Music has no direct API and falls through to Odesli.

mod entity_id;
mod providers;

use crate::detection::MusicProvider;

/// Public Cloudflare-worker proxy base. NOT a secret — this is the same public
/// URL hardcoded in the `src-tauri` original. No API keys are embedded here.
const QBZ_PROXY_BASE: &str = "https://qbz-api-proxy.blitzkriegfc.workers.dev";

/// Try to get title+artist directly from the platform API.
/// Returns None if the platform isn't supported or the request fails.
pub(crate) async fn try_direct_platform_metadata(
    url: &str,
    provider: &MusicProvider,
    is_track: bool,
) -> Option<(String, String)> {
    match provider {
        MusicProvider::Deezer => providers::try_deezer_metadata(url, is_track).await,
        MusicProvider::Spotify => providers::try_spotify_metadata(url, is_track).await,
        MusicProvider::Tidal => providers::try_tidal_metadata(url, is_track).await,
        MusicProvider::AppleMusic => None, // No direct API available
    }
}
