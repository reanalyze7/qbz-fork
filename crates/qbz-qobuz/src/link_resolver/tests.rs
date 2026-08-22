use super::*;

mod tests_more;

// ── Happy path: HTTPS URLs ──

#[test]
fn test_https_album() {
    let result = resolve_link("https://play.qobuz.com/album/0060254728933");
    assert_eq!(result, Ok(ResolvedLink::OpenAlbum("0060254728933".into())));
}

#[test]
fn test_https_track() {
    let result = resolve_link("https://play.qobuz.com/track/12345678");
    assert_eq!(result, Ok(ResolvedLink::OpenTrack(12345678)));
}

#[test]
fn test_https_artist() {
    let result = resolve_link("https://play.qobuz.com/artist/56789");
    assert_eq!(result, Ok(ResolvedLink::OpenArtist(56789)));
}

#[test]
fn test_https_interpreter() {
    let result = resolve_link("https://play.qobuz.com/interpreter/56789");
    assert_eq!(result, Ok(ResolvedLink::OpenArtist(56789)));
}

#[test]
fn test_https_playlist() {
    let result = resolve_link("https://play.qobuz.com/playlist/99887766");
    assert_eq!(result, Ok(ResolvedLink::OpenPlaylist(99887766)));
}

// ── Happy path: qobuzapp:// scheme ──

#[test]
fn test_scheme_album() {
    let result = resolve_link("qobuzapp://album/abc123def");
    assert_eq!(result, Ok(ResolvedLink::OpenAlbum("abc123def".into())));
}

#[test]
fn test_scheme_track() {
    let result = resolve_link("qobuzapp://track/42");
    assert_eq!(result, Ok(ResolvedLink::OpenTrack(42)));
}

#[test]
fn test_scheme_artist() {
    let result = resolve_link("qobuzapp://artist/100");
    assert_eq!(result, Ok(ResolvedLink::OpenArtist(100)));
}

#[test]
fn test_scheme_playlist() {
    let result = resolve_link("qobuzapp://playlist/200");
    assert_eq!(result, Ok(ResolvedLink::OpenPlaylist(200)));
}

// ── Edge cases: trimming ──

#[test]
fn test_trailing_slash() {
    let result = resolve_link("https://play.qobuz.com/album/123/");
    assert_eq!(result, Ok(ResolvedLink::OpenAlbum("123".into())));
}

#[test]
fn test_query_params_stripped() {
    let result = resolve_link("https://play.qobuz.com/album/123?ref=share&utm_source=web");
    assert_eq!(result, Ok(ResolvedLink::OpenAlbum("123".into())));
}

#[test]
fn test_fragment_stripped() {
    let result = resolve_link("https://play.qobuz.com/album/123#tracklist");
    assert_eq!(result, Ok(ResolvedLink::OpenAlbum("123".into())));
}

#[test]
fn test_whitespace_trimmed() {
    let result = resolve_link("  https://play.qobuz.com/album/123  ");
    assert_eq!(result, Ok(ResolvedLink::OpenAlbum("123".into())));
}

#[test]
fn test_http_variant() {
    let result = resolve_link("http://play.qobuz.com/track/555");
    assert_eq!(result, Ok(ResolvedLink::OpenTrack(555)));
}

#[test]
fn test_open_qobuz_variant() {
    let result = resolve_link("https://open.qobuz.com/album/999");
    assert_eq!(result, Ok(ResolvedLink::OpenAlbum("999".into())));
}

