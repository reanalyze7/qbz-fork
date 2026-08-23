mod origins;
mod rc;

use crate::settings::remote_control::{AllowedOriginsStore, RemoteControlSettingsStore};

pub(super) fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

pub(super) fn fresh_remote_store(name: &str) -> (std::path::PathBuf, RemoteControlSettingsStore) {
    let dir = unique_test_dir(name);
    let store = RemoteControlSettingsStore::new_at(&dir).expect("open store in temp dir");
    (dir, store)
}

pub(super) fn fresh_origins_store(name: &str) -> (std::path::PathBuf, AllowedOriginsStore) {
    let dir = unique_test_dir(name);
    let store = AllowedOriginsStore::new_at(&dir).expect("open origins store in temp dir");
    (dir, store)
}
