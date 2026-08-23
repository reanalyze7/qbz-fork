use super::*;
use std::path::PathBuf;

fn unique_test_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn favorites_preferences_default_tab_order_is_stable() {
    let prefs = FavoritesPreferences::default();

    assert_eq!(
        prefs.tab_order,
        vec![
            "tracks".to_string(),
            "albums".to_string(),
            "artists".to_string(),
            "playlists".to_string()
        ]
    );
    assert_eq!(prefs.custom_icon_preset, Some("heart".to_string()));
}

#[test]
fn favorites_preferences_store_returns_default_when_empty() {
    let dir = unique_test_dir("favorites-default");
    let store = FavoritesPreferencesStore::new_at(&dir).unwrap();

    let prefs = store.get_preferences().unwrap();

    assert_eq!(prefs.tab_order, FavoritesPreferences::default().tab_order);
    assert_eq!(
        prefs.custom_icon_preset,
        FavoritesPreferences::default().custom_icon_preset
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn favorites_preferences_persist_without_custom_icon() {
    let dir = unique_test_dir("favorites-persist");
    let store = FavoritesPreferencesStore::new_at(&dir).unwrap();
    let prefs = FavoritesPreferences {
        custom_icon_path: None,
        custom_icon_preset: Some("star".to_string()),
        icon_background: Some("#112233".to_string()),
        tab_order: vec!["artists".to_string(), "tracks".to_string()],
    };

    let saved = store.save_preferences(prefs.clone()).unwrap();
    let loaded = store.get_preferences().unwrap();

    assert_eq!(saved.custom_icon_preset, prefs.custom_icon_preset);
    assert_eq!(loaded.custom_icon_preset, prefs.custom_icon_preset);
    assert_eq!(loaded.icon_background, prefs.icon_background);
    assert_eq!(loaded.tab_order, prefs.tab_order);
    let _ = std::fs::remove_dir_all(dir);
}
