use rusqlite::Connection;

use super::{fresh_remote_store, unique_test_dir};
use crate::settings::remote_control::{RemoteControlSettings, RemoteControlSettingsStore};

#[test]
fn remote_control_default_struct_values_are_stable() {
    let defaults = RemoteControlSettings::default();

    assert!(!defaults.enabled);
    assert_eq!(defaults.port, 8182);
    assert!(defaults.secure);
    assert!(defaults.token.is_empty());
}

#[test]
fn remote_control_store_creates_token_and_defaults_internal_remote_to_http() {
    let (dir, store) = fresh_remote_store("remote-default");

    let settings = store.get_settings().expect("get settings");

    assert!(!settings.enabled);
    assert_eq!(settings.port, 8182);
    assert!(!settings.secure);
    assert!(!settings.token.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn remote_control_settings_persist_across_reopen() {
    let dir = unique_test_dir("remote-persist");
    let token;
    {
        let store = RemoteControlSettingsStore::new_at(&dir).expect("open store");
        token = store.get_settings().expect("get settings").token;
        store.set_enabled(true).expect("set enabled");
        store.set_port(9310).expect("set port");
        store.set_secure(true).expect("set secure");
    }

    let reopened = RemoteControlSettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert!(settings.enabled);
    assert_eq!(settings.port, 9310);
    assert!(settings.secure);
    assert_eq!(settings.token, token);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn remote_control_regenerate_token_changes_and_persists_token() {
    let dir = unique_test_dir("remote-token");
    let original;
    let regenerated;
    {
        let store = RemoteControlSettingsStore::new_at(&dir).expect("open store");
        original = store.get_settings().expect("get settings").token;
        regenerated = store.regenerate_token().expect("regenerate token");
    }

    let reopened = RemoteControlSettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert_ne!(original, regenerated);
    assert_eq!(settings.token, regenerated);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn remote_control_empty_token_is_repaired_on_read() {
    let dir = unique_test_dir("remote-empty-token");
    let db_path = dir.join("remote_control_settings.db");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    {
        let conn = Connection::open(&db_path).expect("open db");
        conn.execute_batch(
            "CREATE TABLE remote_control_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enabled INTEGER NOT NULL DEFAULT 0,
                port INTEGER NOT NULL DEFAULT 8182,
                secure INTEGER NOT NULL DEFAULT 1,
                token TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO remote_control_settings (id, enabled, port, secure, token)
            VALUES (1, 0, 8182, 0, '');",
        )
        .expect("create schema");
    }

    let store = RemoteControlSettingsStore::new_at(&dir).expect("open store");
    let settings = store.get_settings().expect("get settings");

    assert!(!settings.token.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn remote_control_legacy_schema_adds_secure_default_false() {
    let dir = unique_test_dir("remote-legacy");
    let db_path = dir.join("remote_control_settings.db");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    {
        let conn = Connection::open(&db_path).expect("open legacy db");
        conn.execute_batch(
            "CREATE TABLE remote_control_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enabled INTEGER NOT NULL DEFAULT 0,
                port INTEGER NOT NULL DEFAULT 8182,
                token TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO remote_control_settings (id, enabled, port, token)
            VALUES (1, 1, 8183, 'legacy-token');",
        )
        .expect("create legacy schema");
    }

    let store = RemoteControlSettingsStore::new_at(&dir).expect("migrate store");
    let settings = store.get_settings().expect("get settings");

    assert!(settings.enabled);
    assert_eq!(settings.port, 8183);
    assert!(!settings.secure);
    assert_eq!(settings.token, "legacy-token");
    let _ = std::fs::remove_dir_all(dir);
}
