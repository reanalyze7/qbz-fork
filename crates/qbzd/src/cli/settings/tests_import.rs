// crates/qbzd/src/cli/settings/tests_import.rs — `present_keys` and the
// import flow's reload-disposition mapping.

use super::reload::reload_disposition;
use super::tests_support::{cleanup, scratch_roots};
use super::verbs_config::present_keys;

#[test]
fn present_keys_empty_for_a_missing_file() {
    let roots = scratch_roots("config-missing");
    let keys = present_keys(&roots.config.join("qbzd.toml"));
    assert!(keys.is_empty());
    cleanup(&roots);
}

#[test]
fn reload_disposition_maps_the_three_outcomes() {
    use crate::login::NudgeOutcome::*;

    let (line, err, code) = reload_disposition(Reloaded, false);
    assert_eq!(line, "daemon reloaded (was running)");
    assert!(err.is_none());
    assert_eq!(code, 0);

    // §5.3 step 7 honesty rule: routing-critical + reloaded names the gap.
    let (line, err, code) = reload_disposition(Reloaded, true);
    assert!(line.contains("output device reinitialized"), "{line}");
    assert!(line.contains("gap"), "{line}");
    assert!(err.is_none());
    assert_eq!(code, 0);

    // Daemon simply not running is NOT an error, routing-critical or not.
    let (line, err, code) = reload_disposition(DaemonDown, true);
    assert_eq!(line, "daemon not running (changes apply on next start)");
    assert!(err.is_none());
    assert_eq!(code, 0);

    // Up-but-refused → exit 1 with the verbatim restart hint.
    let (_, err, code) = reload_disposition(ReloadRefused, false);
    let msg = err.expect("refused must carry the stderr copy");
    assert_eq!(
        msg,
        "error: settings saved but the daemon did not reload — restart it: systemctl --user restart qbzd"
    );
    assert_eq!(code, 1);
}

#[test]
fn present_keys_reports_nested_and_top_level_keys() {
    let roots = scratch_roots("config-present");
    std::fs::create_dir_all(&roots.config).unwrap();
    std::fs::write(
        roots.config.join("qbzd.toml"),
        "config_version = 1\n[server]\nport = 9000\n",
    )
    .unwrap();
    let keys = present_keys(&roots.config.join("qbzd.toml"));
    assert!(keys.contains("config_version"));
    assert!(keys.contains("server.port"));
    assert!(!keys.contains("server.bind"));
    cleanup(&roots);
}
