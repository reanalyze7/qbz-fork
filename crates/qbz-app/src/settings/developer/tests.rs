use super::{DeveloperSettings, DeveloperSettingsStore};

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

fn fresh_store(name: &str) -> (std::path::PathBuf, DeveloperSettingsStore) {
    let dir = unique_test_dir(name);
    let store = DeveloperSettingsStore::new_at(&dir).expect("open store in temp dir");
    (dir, store)
}

#[test]
fn developer_settings_default_values_are_stable() {
    let settings = DeveloperSettings::default();

    assert!(!settings.force_dmabuf);
}

#[test]
fn developer_settings_store_returns_defaults() {
    let (dir, store) = fresh_store("developer-default");

    let settings = store.get_settings().expect("get settings");

    assert!(!settings.force_dmabuf);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn developer_settings_persist_force_dmabuf() {
    let dir = unique_test_dir("developer-force-dmabuf");
    {
        let store = DeveloperSettingsStore::new_at(&dir).expect("open store");
        store.set_force_dmabuf(true).expect("set force dmabuf");
    }

    let reopened = DeveloperSettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert!(settings.force_dmabuf);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn developer_settings_readonly_opens_existing_db() {
    let dir = unique_test_dir("developer-readonly");
    let db_path = dir.join("developer_settings.db");
    {
        let store = DeveloperSettingsStore::new_at(&dir).expect("open store");
        store.set_force_dmabuf(true).expect("set force dmabuf");
    }

    let readonly = DeveloperSettingsStore::new_readonly_at_path(&db_path)
        .expect("open existing store read-only");
    let settings = readonly.get_settings().expect("get settings");

    assert!(settings.force_dmabuf);
    let _ = std::fs::remove_dir_all(dir);
}
