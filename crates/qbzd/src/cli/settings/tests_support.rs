// crates/qbzd/src/cli/settings/tests_support.rs — per-test scratch profile
// roots (safe in parallel: nonce + pid in the temp dir name).

use crate::paths::ProfileRoots;

pub(super) fn scratch_roots(name: &str) -> ProfileRoots {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!(
        "qbzd-cli-settings-{name}-{}-{nonce}",
        std::process::id()
    ));
    ProfileRoots {
        config: base.join("config"),
        data: base.join("data"),
        cache: base.join("cache"),
    }
}

pub(super) fn cleanup(roots: &ProfileRoots) {
    let _ = std::fs::remove_dir_all(roots.data.parent().unwrap_or(&roots.data));
}
