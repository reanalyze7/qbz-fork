//! UI-facing provider-key detection (looser than [`crate::providers::detect_provider`]).

/// Provider key for the UI gate ("spotify" | "apple" | "tidal" | "deezer").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKey {
    Spotify,
    Apple,
    Tidal,
    Deezer,
}

impl ProviderKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKey::Spotify => "spotify",
            ProviderKey::Apple => "apple",
            ProviderKey::Tidal => "tidal",
            ProviderKey::Deezer => "deezer",
        }
    }
}

/// UI-gate provider detection — the exact substring rules of the Svelte
/// `detectProvider` (looser than [`crate::providers::detect_provider`], which stays
/// the authoritative backend validation and may reject what this gate
/// passed). Kept here so the UI enable/disable logic and the backend share
/// one source of truth.
pub fn detect_provider_key(url: &str) -> Option<ProviderKey> {
    let url = url.trim();

    if url.starts_with("spotify:playlist:")
        || url.contains("open.spotify.com/playlist/")
        || url.contains("open.spotify.com/embed/playlist/")
    {
        return Some(ProviderKey::Spotify);
    }
    if url.contains("music.apple.com/") && url.contains("/playlist/") {
        return Some(ProviderKey::Apple);
    }
    if url.contains("tidal.com/") && url.contains("/playlist/") {
        return Some(ProviderKey::Tidal);
    }
    if url.contains("deezer.com/") && url.contains("/playlist/") {
        return Some(ProviderKey::Deezer);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_provider_key_table() {
        let cases: &[(&str, Option<ProviderKey>)] = &[
            // Spotify: URI, URL, embed
            (
                "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M",
                Some(ProviderKey::Spotify),
            ),
            (
                "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M",
                Some(ProviderKey::Spotify),
            ),
            (
                "https://open.spotify.com/embed/playlist/37i9dQZF1DXcBWIGoYBM5M",
                Some(ProviderKey::Spotify),
            ),
            // Apple: needs both music.apple.com/ AND /playlist/
            (
                "https://music.apple.com/us/playlist/top-100/pl.123",
                Some(ProviderKey::Apple),
            ),
            ("https://music.apple.com/us/album/x/123", None),
            // Tidal
            (
                "https://tidal.com/browse/playlist/abc-def",
                Some(ProviderKey::Tidal),
            ),
            ("https://tidal.com/browse/album/123", None),
            // Deezer
            (
                "https://www.deezer.com/en/playlist/1234567",
                Some(ProviderKey::Deezer),
            ),
            ("https://www.deezer.com/en/album/1234567", None),
            // Rejects
            ("https://open.spotify.com/track/abc", None),
            ("https://example.com/playlist/1", None),
            ("", None),
        ];

        for (url, expected) in cases {
            assert_eq!(detect_provider_key(url), *expected, "url: {}", url);
        }
    }

    #[test]
    fn detect_provider_key_trims_whitespace() {
        assert_eq!(
            detect_provider_key("  spotify:playlist:abc  "),
            Some(ProviderKey::Spotify)
        );
    }

    #[test]
    fn provider_key_as_str() {
        assert_eq!(ProviderKey::Spotify.as_str(), "spotify");
        assert_eq!(ProviderKey::Apple.as_str(), "apple");
        assert_eq!(ProviderKey::Tidal.as_str(), "tidal");
        assert_eq!(ProviderKey::Deezer.as_str(), "deezer");
    }
}
