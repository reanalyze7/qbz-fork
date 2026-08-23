use super::*;

#[test]
fn induced_flag_is_persisted_and_reloaded() {
    let _gate = serialize();
    let dir = unique_test_dir("engine-persist");
    {
        let engine = OfflineModeEngine::new();
        engine.init_for_user(&dir).unwrap();
        engine.set_induced(true, None).unwrap();
    }
    {
        let engine = OfflineModeEngine::new();
        engine.init_for_user(&dir).unwrap();
        assert_eq!(engine.status().mode, OfflineMode::InducedOffline);
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn teardown_ends_offline_session_and_reopens_gate() {
    let _gate = serialize();
    let engine = OfflineModeEngine::new();
    engine.on_connectivity(up());
    engine.set_offline_session(true);
    assert_eq!(engine.status().mode, OfflineMode::RealOffline);
    assert!(qbz_qobuz::offline_gate::is_offline());

    // Logout: the session-scoped flag must NOT survive — a stale flag
    // kept the gate closed and refused the next login.
    engine.teardown();
    let status = engine.status();
    assert_eq!(status.mode, OfflineMode::Online);
    assert!(!status.offline_session);
    assert!(!qbz_qobuz::offline_gate::is_offline());
}

#[test]
fn teardown_clears_induced_cache_but_disk_restores_it() {
    let _gate = serialize();
    let dir = unique_test_dir("engine-teardown-induced");
    let engine = OfflineModeEngine::new();
    engine.init_for_user(&dir).unwrap();
    engine.on_connectivity(up());
    engine.set_induced(true, None).unwrap();
    assert!(qbz_qobuz::offline_gate::is_offline());

    // Logout: no user ⇒ no induced opt-in active ⇒ gate open.
    engine.teardown();
    let status = engine.status();
    assert_eq!(status.mode, OfflineMode::Online);
    assert!(!status.induced);
    assert!(!qbz_qobuz::offline_gate::is_offline());

    // The persisted preference survives on disk: the next activation on
    // the same dir restores induced offline.
    engine.init_for_user(&dir).unwrap();
    assert_eq!(engine.status().mode, OfflineMode::InducedOffline);
    assert!(qbz_qobuz::offline_gate::is_offline());

    let _ = std::fs::remove_dir_all(dir);
}
