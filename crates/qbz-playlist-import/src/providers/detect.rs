//! URL-to-resource detection shared across providers.

use super::{apple, deezer, spotify, tidal, MusicResource};

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
