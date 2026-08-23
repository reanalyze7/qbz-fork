use super::*;
use rusqlite::Connection;

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

fn fresh_store(name: &str) -> (std::path::PathBuf, TraySettingsStore) {
    let dir = unique_test_dir(name);
    let store = TraySettingsStore::new_at(&dir).expect("open store in temp dir");
    (dir, store)
}

#[test]
fn tray_settings_default_values_are_stable() {
    let settings = TraySettings::default();

    assert!(settings.enable_tray);
    assert!(!settings.minimize_to_tray);
    assert!(settings.close_to_tray);
    assert_eq!(settings.tray_icon_theme, "auto");
}

#[test]
fn tray_settings_store_returns_defaults() {
    let (dir, store) = fresh_store("tray-default");

    let settings = store.get_settings().expect("get settings");

    assert!(settings.enable_tray);
    assert!(!settings.minimize_to_tray);
    assert!(settings.close_to_tray);
    assert_eq!(settings.tray_icon_theme, "auto");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tray_settings_persist_all_fields() {
    let dir = unique_test_dir("tray-persist");
    {
        let store = TraySettingsStore::new_at(&dir).expect("open store");
        store.set_enable_tray(false).expect("set enable");
        store.set_minimize_to_tray(true).expect("set minimize");
        store.set_close_to_tray(true).expect("set close");
        store
            .set_tray_icon_theme("mono-light")
            .expect("set icon theme");
    }

    let reopened = TraySettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert!(!settings.enable_tray);
    assert!(settings.minimize_to_tray);
    assert!(settings.close_to_tray);
    assert_eq!(settings.tray_icon_theme, "mono-light");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tray_icon_theme_normalizes_supported_legacy_and_unknown_values() {
    assert_eq!(normalize_tray_icon_theme("auto"), "auto");
    assert_eq!(normalize_tray_icon_theme("mono-light"), "mono-light");
    assert_eq!(normalize_tray_icon_theme("mono-dark"), "mono-dark");
    assert_eq!(normalize_tray_icon_theme("color"), "color");
    assert_eq!(normalize_tray_icon_theme("light"), "mono-light");
    assert_eq!(normalize_tray_icon_theme("dark"), "mono-dark");
    assert_eq!(normalize_tray_icon_theme("invalid"), "auto");
}

#[test]
fn tray_settings_migrates_legacy_schema() {
    let dir = unique_test_dir("tray-migrate");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let db_path = dir.join("tray_settings.db");
    {
        let conn = Connection::open(&db_path).expect("open legacy db");
        conn.execute_batch(
            "CREATE TABLE tray_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enable_tray INTEGER NOT NULL DEFAULT 1,
                minimize_to_tray INTEGER NOT NULL DEFAULT 0,
                close_to_tray INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO tray_settings (id, enable_tray, minimize_to_tray, close_to_tray)
            VALUES (1, 0, 1, 1);",
        )
        .expect("create legacy schema");
    }

    let store = TraySettingsStore::new_at(&dir).expect("migrate store");
    let settings = store.get_settings().expect("get settings");

    assert!(!settings.enable_tray);
    assert!(settings.minimize_to_tray);
    assert!(settings.close_to_tray);
    assert_eq!(settings.tray_icon_theme, "auto");
    let _ = std::fs::remove_dir_all(dir);
}
