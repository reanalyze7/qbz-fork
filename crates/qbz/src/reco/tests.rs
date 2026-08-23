use super::*;
use qbz_app::settings::reco_store::HomeSeedLimits;

#[test]
fn helpers_are_noop_when_uninitialized() {
    teardown(); // ensure no store is open
    // Logging with no open store must not panic and must report "not logged".
    assert!(!log_play_gated(123, None, None, Some("qobuz")));
    // Reading seeds with no open store yields None (caller falls back to local).
    assert!(home_seeds(HomeSeedLimits::default()).is_none());
}

#[test]
fn qobuz_source_gate_excludes_local_ephemeral() {
    assert!(is_qobuz_source(None)); // queue default = "qobuz"
    assert!(is_qobuz_source(Some("qobuz")));
    assert!(is_qobuz_source(Some("qobuz_download"))); // purchased Qobuz track, resolvable id
    assert!(!is_qobuz_source(Some("local")));
    assert!(!is_qobuz_source(Some("ephemeral")));
}
