mod defaults_and_ops;
mod reconcile_and_migrate;
mod store_roundtrip;

use super::*;

pub(super) fn ids(list: &[SectionPref]) -> Vec<DiscoverySectionId> {
    list.iter().map(|p| p.id).collect()
}

pub(super) fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}
