use crate::settings::AudioSettingsStore;

pub(super) fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-audio-{name}-{}-{nonce}", std::process::id()))
}

pub(super) fn fresh_store(name: &str) -> (std::path::PathBuf, AudioSettingsStore) {
    let dir = unique_test_dir(name);
    let store = AudioSettingsStore::new_at(&dir).expect("open store in temp dir");
    (dir, store)
}
