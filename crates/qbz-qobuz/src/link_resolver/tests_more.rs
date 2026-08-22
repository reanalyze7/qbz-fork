use super::super::*;

// ── Error cases ──

#[test]
fn test_empty_input() {
    assert_eq!(resolve_link(""), Err(LinkResolverError::EmptyInput));
}

#[test]
fn test_whitespace_only() {
    assert_eq!(resolve_link("   "), Err(LinkResolverError::EmptyInput));
}

#[test]
fn test_unsupported_scheme() {
    assert_eq!(
        resolve_link("https://www.google.com/album/123"),
        Err(LinkResolverError::UnsupportedScheme)
    );
}

#[test]
fn test_random_text() {
    assert_eq!(
        resolve_link("not a url at all"),
        Err(LinkResolverError::UnsupportedScheme)
    );
}

#[test]
fn test_unknown_entity() {
    assert_eq!(
        resolve_link("https://play.qobuz.com/label/123"),
        Err(LinkResolverError::UnknownEntityType("label".into()))
    );
}

#[test]
fn test_invalid_track_id() {
    assert_eq!(
        resolve_link("https://play.qobuz.com/track/not-a-number"),
        Err(LinkResolverError::InvalidId("not-a-number".into()))
    );
}

#[test]
fn test_missing_id() {
    assert_eq!(
        resolve_link("https://play.qobuz.com/album/"),
        Err(LinkResolverError::MalformedUrl)
    );
}

#[test]
fn test_scheme_no_path() {
    assert_eq!(
        resolve_link("qobuzapp://"),
        Err(LinkResolverError::MalformedUrl)
    );
}

#[test]
fn test_scheme_only_entity_no_id() {
    assert_eq!(
        resolve_link("qobuzapp://album"),
        Err(LinkResolverError::MalformedUrl)
    );
}
