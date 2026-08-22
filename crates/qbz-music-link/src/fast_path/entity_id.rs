//! Pure URL/URI entity-id extraction helpers for the platform fast-path.

/// Extract a numeric or alphanumeric ID after /track/ or /album/ in a URL.
pub(super) fn extract_entity_id(url: &str, entity_type: &str) -> Option<String> {
    let pattern = format!("/{}/", entity_type);
    let idx = url.find(&pattern)?;
    let rest = &url[idx + pattern.len()..];
    let id = rest.split(['?', '/', '#']).next()?;
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

/// Extract Spotify ID from URL or URI.
pub(super) fn extract_spotify_entity_id(url: &str, entity_type: &str) -> Option<String> {
    // URI format: spotify:track:abc123
    let uri_pattern = format!("spotify:{}:", entity_type);
    if let Some(rest) = url.strip_prefix(&uri_pattern) {
        let id = rest.split(['?', '/']).next()?;
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    extract_entity_id(url, entity_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_plain_entity_id() {
        assert_eq!(
            extract_entity_id("https://x/track/abc123?y=1", "track"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_entity_id("https://x/track/", "track"), None);
    }

    #[test]
    fn extracts_spotify_uri_id() {
        assert_eq!(
            extract_spotify_entity_id("spotify:track:abc123", "track"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_spotify_entity_id("https://open.spotify.com/track/abc123", "track"),
            Some("abc123".to_string())
        );
    }
}
