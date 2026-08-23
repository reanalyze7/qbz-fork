use super::*;
use crate::runtime::RuntimeState;
use crate::session_store::PersistedSessionSnapshot;
use qbz_audio::AudioSettings;
use qbz_core::NoOpAdapter;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    static NONCE: AtomicU64 = AtomicU64::new(0);
    let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

fn test_runtime() -> AppRuntime<NoOpAdapter> {
    AppRuntime::with_audio_settings(NoOpAdapter, None, AudioSettings::default(), None)
}

#[test]
fn builds_with_explicit_audio_settings() {
    let rt = test_runtime();
    let _core = rt.core();
    assert!(!rt.is_session_active());
    assert_eq!(rt.active_user_id(), None);
}

#[tokio::test]
async fn runtime_state_machine_starts_uninitialized() {
    let rt = test_runtime();
    assert_eq!(
        rt.runtime().get_status().await.state,
        RuntimeState::Uninitialized
    );
}

#[tokio::test]
async fn core_reports_no_session_before_login() {
    let rt = test_runtime();
    assert!(!rt.core().has_session().await);
    assert!(!rt.core().is_api_initialized().await);
}

#[tokio::test]
async fn activate_at_opens_session_and_marks_runtime() {
    let rt = test_runtime();
    let data_dir = unique_test_dir("activate-data");
    let cache_dir = unique_test_dir("activate-cache");

    rt.activate_at(42, &data_dir, &cache_dir)
        .await
        .expect("activation succeeds");

    assert!(rt.is_session_active());
    assert_eq!(rt.active_user_id(), Some(42));
    assert!(rt.runtime().get_status().await.session_activated);
    assert!(data_dir.join("session.db").exists());
    assert!(cache_dir.exists());

    let _ = std::fs::remove_dir_all(&data_dir);
    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn deactivate_clears_session_and_runtime() {
    let rt = test_runtime();
    let data_dir = unique_test_dir("deactivate-data");
    let cache_dir = unique_test_dir("deactivate-cache");

    rt.activate_at(7, &data_dir, &cache_dir)
        .await
        .expect("activation succeeds");
    rt.deactivate().await.expect("deactivation succeeds");

    assert!(!rt.is_session_active());
    assert_eq!(rt.active_user_id(), None);
    assert!(!rt.runtime().get_status().await.session_activated);

    let _ = std::fs::remove_dir_all(&data_dir);
    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn with_session_store_round_trips_through_active_session() {
    let rt = test_runtime();
    let data_dir = unique_test_dir("store-data");
    let cache_dir = unique_test_dir("store-cache");

    // No session yet: closure is not run.
    assert!(rt.with_session_store(|_| ()).is_none());

    rt.activate_at(1, &data_dir, &cache_dir)
        .await
        .expect("activation succeeds");

    let snapshot = PersistedSessionSnapshot::default();
    rt.with_session_store(|store| store.save_session(&snapshot))
        .expect("session is active")
        .expect("save succeeds");

    let loaded = rt
        .with_session_store(|store| store.load_session())
        .expect("session is active")
        .expect("load succeeds");
    assert_eq!(
        loaded.playback.queue_tracks.len(),
        snapshot.playback.queue_tracks.len()
    );

    let _ = std::fs::remove_dir_all(&data_dir);
    let _ = std::fs::remove_dir_all(&cache_dir);
}
