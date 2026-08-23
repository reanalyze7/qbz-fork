use super::*;

#[test]
fn matches_open_qobuz_album_https_and_http() {
    assert!(is_qobuz_link("https://open.qobuz.com/album/kq3s910v1qufc"));
    assert!(is_qobuz_link("http://open.qobuz.com/album/kq3s910v1qufc"));
}

#[test]
fn matches_play_qobuz_https_and_http() {
    assert!(is_qobuz_link("https://play.qobuz.com/album/kq3s910v1qufc"));
    assert!(is_qobuz_link("http://play.qobuz.com/track/123456"));
}

#[test]
fn matches_qobuzapp_scheme() {
    assert!(is_qobuz_link("qobuzapp://album/kq3s910v1qufc"));
    assert!(is_qobuz_link("qobuzapp://artist/123"));
}

#[test]
fn ignores_non_link_args() {
    assert!(!is_qobuz_link("--verbose"));
    assert!(!is_qobuz_link("/home/user/music.flac"));
    assert!(!is_qobuz_link("https://example.com/album/kq3s910v1qufc"));
    assert!(!is_qobuz_link("https://open.qobuz.com.evil.test/album/x"));
    // The dead `qbz://` scheme resolves nowhere and stays unmatched.
    assert!(!is_qobuz_link("qbz://album/kq3s910v1qufc"));
    assert!(!is_qobuz_link(""));
}

#[test]
fn select_link_takes_first_match() {
    let args = vec![
        "--flag".to_string(),
        "https://open.qobuz.com/album/first".to_string(),
        "qobuzapp://album/second".to_string(),
    ];
    assert_eq!(
        select_link(&args),
        Some("https://open.qobuz.com/album/first".to_string())
    );
}

#[test]
fn select_link_returns_none_without_match() {
    let args = vec!["--flag".to_string(), "cover.jpg".to_string()];
    assert_eq!(select_link(&args), None);
}

/// Single stateful test: PENDING is process-global and cargo runs tests
/// on threads, so the round-trip + overwrite checks stay sequential here.
#[test]
fn pending_drains_once_and_newest_wins() {
    stash("qobuzapp://album/old".to_string());
    stash("qobuzapp://album/new".to_string());
    assert_eq!(take_pending(), Some("qobuzapp://album/new".to_string()));
    assert_eq!(take_pending(), None);
}
