use super::*;
use rusqlite::Connection;

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

fn fresh_store(name: &str) -> (std::path::PathBuf, PlaybackPreferencesStore) {
    let dir = unique_test_dir(name);
    let store = PlaybackPreferencesStore::new_at(&dir).expect("open store in temp dir");
    (dir, store)
}

#[test]
fn playback_preferences_default_values_are_stable() {
    let prefs = PlaybackPreferences::default();

    assert_eq!(prefs.autoplay_mode, AutoplayMode::ContinueWithinSource);
    assert!(prefs.show_context_icon);
    assert!(prefs.persist_session);
    assert!(prefs.resume_playback_position);
}

#[test]
fn playback_preferences_store_returns_defaults() {
    let (dir, store) = fresh_store("playback-default");

    let prefs = store.get_preferences().expect("get prefs");

    assert_eq!(prefs.autoplay_mode, AutoplayMode::ContinueWithinSource);
    assert!(prefs.show_context_icon);
    assert!(prefs.persist_session);
    assert!(prefs.resume_playback_position);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn playback_preferences_persist_all_fields() {
    let dir = unique_test_dir("playback-persist");
    {
        let store = PlaybackPreferencesStore::new_at(&dir).expect("open store");
        store
            .set_autoplay_mode(AutoplayMode::InfiniteRadio)
            .expect("set autoplay");
        store.set_show_context_icon(true).expect("set context icon");
        store
            .set_persist_session(true)
            .expect("set persist session");
        store
            .set_resume_playback_position(true)
            .expect("set resume position");
    }

    let reopened = PlaybackPreferencesStore::new_at(&dir).expect("reopen store");
    let prefs = reopened.get_preferences().expect("get prefs");

    assert_eq!(prefs.autoplay_mode, AutoplayMode::InfiniteRadio);
    assert!(prefs.show_context_icon);
    assert!(prefs.persist_session);
    assert!(prefs.resume_playback_position);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn playback_preferences_migrates_legacy_schema() {
    let dir = unique_test_dir("playback-migrate");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let db_path = dir.join("playback_preferences.db");
    {
        let conn = Connection::open(&db_path).expect("open legacy db");
        conn.execute_batch(
            "CREATE TABLE playback_preferences (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                autoplay_mode TEXT NOT NULL DEFAULT 'continue'
            );
            INSERT INTO playback_preferences (id, autoplay_mode) VALUES (1, 'track_only');",
        )
        .expect("create legacy schema");
    }

    let store = PlaybackPreferencesStore::new_at(&dir).expect("migrate store");
    let prefs = store.get_preferences().expect("get prefs");

    assert_eq!(prefs.autoplay_mode, AutoplayMode::PlayTrackOnly);
    assert!(prefs.show_context_icon);
    assert!(prefs.persist_session);
    assert!(prefs.resume_playback_position);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn playback_preferences_reset_all_preserves_existing_behavior() {
    let (dir, store) = fresh_store("playback-reset");
    store
        .set_autoplay_mode(AutoplayMode::InfiniteRadio)
        .expect("set autoplay");
    store.set_show_context_icon(true).expect("set context icon");
    store.set_persist_session(true).expect("set persist");
    store
        .set_resume_playback_position(true)
        .expect("set resume position");

    let defaults = store.reset_all().expect("reset prefs");
    let prefs = store.get_preferences().expect("get prefs");

    assert_eq!(defaults.autoplay_mode, AutoplayMode::ContinueWithinSource);
    assert_eq!(prefs.autoplay_mode, AutoplayMode::ContinueWithinSource);
    assert!(prefs.show_context_icon);
    assert!(prefs.persist_session);
    assert!(prefs.resume_playback_position);
    let _ = std::fs::remove_dir_all(dir);
}
