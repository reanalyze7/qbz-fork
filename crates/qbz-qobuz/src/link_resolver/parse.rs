use super::LinkResolverError;

/// Strip `https://play.qobuz.com/` or `http://play.qobuz.com/` prefix.
/// Also accepts `https://open.qobuz.com/` variant.
pub(super) fn strip_web_prefix(url: &str) -> Option<&str> {
    let lowered = url.to_ascii_lowercase();
    for prefix in &[
        "https://play.qobuz.com/",
        "http://play.qobuz.com/",
        "https://open.qobuz.com/",
        "http://open.qobuz.com/",
    ] {
        if lowered.starts_with(prefix) {
            return Some(&url[prefix.len()..]);
        }
    }
    None
}

/// Parse `<entity_type>/<id>` from a path, stripping query params and fragments.
pub(super) fn parse_path_segments(path: &str) -> Result<(String, String), LinkResolverError> {
    // Strip query string and fragment
    let path = path.split('?').next().unwrap_or(path);
    let path = path.split('#').next().unwrap_or(path);
    // Strip trailing slashes
    let path = path.trim_end_matches('/');

    if path.is_empty() {
        return Err(LinkResolverError::MalformedUrl);
    }

    let mut parts = path.splitn(2, '/');
    let entity_type = parts.next().unwrap_or("").to_ascii_lowercase();
    let raw_id = parts.next().unwrap_or("").to_string();

    if entity_type.is_empty() {
        return Err(LinkResolverError::MalformedUrl);
    }
    if raw_id.is_empty() {
        return Err(LinkResolverError::MalformedUrl);
    }

    Ok((entity_type, raw_id))
}
