use super::*;

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn scrobbler_settings_default_is_unconfigured() {
    let s = ScrobblerSettings::default();
    assert!(!s.enabled);
    assert!(!s.ui_collapsed);
    assert!(!s.lastfm_enabled);
    assert!(s.lastfm_session_key.is_empty());
    assert!(s.lastfm_username.is_empty());
    assert!(!s.listenbrainz_enabled);
    assert!(s.listenbrainz_token.is_empty());
    assert!(s.listenbrainz_username.is_empty());
    assert!(!s.lastfm_is_authed());
    assert!(!s.listenbrainz_is_authed());
    assert!(!s.lastfm_active());
    assert!(!s.listenbrainz_active());
}

#[test]
fn scrobbler_store_returns_defaults() {
    let dir = unique_test_dir("scrobbler-default");
    let store = ScrobblerSettingsStore::new_at(&dir).expect("open store");
    let s = store.get_settings().expect("get settings");
    assert!(!s.enabled);
    assert!(!s.lastfm_is_authed());
    assert!(!s.listenbrainz_is_authed());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scrobbler_persists_all_fields() {
    let dir = unique_test_dir("scrobbler-persist");
    {
        let store = ScrobblerSettingsStore::new_at(&dir).expect("open store");
        store.set_enabled(true).expect("enabled");
        store.set_ui_collapsed(true).expect("collapsed");
        store.set_lastfm_enabled(true).expect("lfm enabled");
        store
            .set_lastfm_session("  sk-123  ", "  alice  ")
            .expect("lfm session");
        store.set_listenbrainz_enabled(true).expect("lb enabled");
        store
            .set_listenbrainz_token("  tok-456  ", "  bob  ")
            .expect("lb token");
    }
    let reopened = ScrobblerSettingsStore::new_at(&dir).expect("reopen store");
    let s = reopened.get_settings().expect("get settings");
    assert!(s.enabled);
    assert!(s.ui_collapsed);
    assert!(s.lastfm_enabled);
    // set_lastfm_session trims whitespace.
    assert_eq!(s.lastfm_session_key, "sk-123");
    assert_eq!(s.lastfm_username, "alice");
    assert!(s.listenbrainz_enabled);
    assert_eq!(s.listenbrainz_token, "tok-456");
    assert_eq!(s.listenbrainz_username, "bob");
    assert!(s.lastfm_active());
    assert!(s.listenbrainz_active());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scrobbler_disconnect_keeps_enable_flags() {
    let dir = unique_test_dir("scrobbler-disconnect");
    let store = ScrobblerSettingsStore::new_at(&dir).expect("open store");
    store.set_enabled(true).expect("enabled");
    store.set_lastfm_enabled(true).expect("lfm enabled");
    store.set_lastfm_session("sk", "alice").expect("lfm session");
    store.set_listenbrainz_enabled(true).expect("lb enabled");
    store.set_listenbrainz_token("tok", "bob").expect("lb token");

    store.disconnect_lastfm().expect("disconnect lfm");
    store.disconnect_listenbrainz().expect("disconnect lb");

    let s = store.get_settings().expect("get settings");
    // Creds cleared.
    assert!(s.lastfm_session_key.is_empty());
    assert!(s.lastfm_username.is_empty());
    assert!(s.listenbrainz_token.is_empty());
    assert!(s.listenbrainz_username.is_empty());
    // Enable flags preserved (so re-auth resumes scrobbling).
    assert!(s.enabled);
    assert!(s.lastfm_enabled);
    assert!(s.listenbrainz_enabled);
    // No longer authed -> not active.
    assert!(!s.lastfm_active());
    assert!(!s.listenbrainz_active());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scrobbler_state_requires_init() {
    let state = ScrobblerSettingsState::new_empty();
    assert!(state.get_settings().is_err());

    let dir = unique_test_dir("scrobbler-state");
    state.init_at(&dir).expect("init at temp dir");
    state
        .set_lastfm_session("sk", "alice")
        .expect("set session via state");
    let s = state.get_settings().expect("get via state");
    assert!(s.lastfm_is_authed());
    assert_eq!(s.lastfm_username, "alice");

    state.teardown().expect("teardown");
    assert!(state.get_settings().is_err());
    let _ = std::fs::remove_dir_all(dir);
}
