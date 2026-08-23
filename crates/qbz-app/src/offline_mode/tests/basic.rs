use super::*;

#[test]
fn starts_online_with_unknown_connectivity() {
    let _gate = serialize();
    let engine = OfflineModeEngine::new();
    let status = engine.status();
    assert_eq!(status.mode, OfflineMode::Online);
    assert_eq!(status.connectivity, Connectivity::Unknown);
}

#[test]
fn connectivity_down_is_real_offline_and_back() {
    let _gate = serialize();
    let engine = OfflineModeEngine::new();
    engine.on_connectivity(down());
    assert_eq!(engine.status().mode, OfflineMode::RealOffline);
    assert!(qbz_qobuz::offline_gate::is_offline());

    engine.on_connectivity(up());
    assert_eq!(engine.status().mode, OfflineMode::Online);
    assert!(!qbz_qobuz::offline_gate::is_offline());
}

#[test]
fn induced_wins_over_connectivity() {
    let _gate = serialize();
    let dir = unique_test_dir("engine-induced");
    let engine = OfflineModeEngine::new();
    engine.init_for_user(&dir).unwrap();

    engine.on_connectivity(up());
    engine.set_induced(true, None).unwrap();
    assert_eq!(engine.status().mode, OfflineMode::InducedOffline);
    assert!(engine.status().induced);
    assert!(qbz_qobuz::offline_gate::is_offline());

    // Exit always allowed; connectivity Up => back Online.
    engine.set_induced(false, None).unwrap();
    assert_eq!(engine.status().mode, OfflineMode::Online);
    assert!(!qbz_qobuz::offline_gate::is_offline());

    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn watch_broadcasts_on_change() {
    let _gate = serialize();
    let engine = std::sync::Arc::new(OfflineModeEngine::new());
    let mut rx = engine.subscribe();

    engine.on_connectivity(down());
    rx.changed().await.unwrap();
    assert_eq!(rx.borrow().mode, OfflineMode::RealOffline);
}
