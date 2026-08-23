use super::store::SubscriptionStateStore;
use super::GRACE_PERIOD_SECS;

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn subscription_state_defaults_to_valid_access() {
    let dir = unique_test_dir("subscription-default");
    let store = SubscriptionStateStore::new_at(&dir).unwrap();

    let state = store.get_state().unwrap();

    assert!(state.invalid_since.is_none());
    assert!(!store.should_purge_offline_cache(0).unwrap());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn subscription_invalid_since_preserves_first_invalid_observation() {
    let dir = unique_test_dir("subscription-invalid");
    let store = SubscriptionStateStore::new_at(&dir).unwrap();

    store.mark_invalid(100).unwrap();
    store.mark_invalid(200).unwrap();
    let state = store.get_state().unwrap();

    assert_eq!(state.invalid_since, Some(100));
    assert_eq!(state.last_invalid_at, Some(200));
    assert_eq!(state.last_checked_at, Some(200));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn subscription_purge_waits_for_grace_period_and_only_runs_once() {
    let dir = unique_test_dir("subscription-purge");
    let store = SubscriptionStateStore::new_at(&dir).unwrap();

    store.mark_invalid(100).unwrap();

    assert!(!store
        .should_purge_offline_cache(100 + GRACE_PERIOD_SECS - 1)
        .unwrap());
    assert!(store
        .should_purge_offline_cache(100 + GRACE_PERIOD_SECS)
        .unwrap());

    store
        .mark_offline_cache_purged(100 + GRACE_PERIOD_SECS)
        .unwrap();

    assert!(!store
        .should_purge_offline_cache(100 + GRACE_PERIOD_SECS + 1)
        .unwrap());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn offline_playback_allowed_within_grace_refused_after() {
    let dir = unique_test_dir("subscription-playback-gate");
    let store = SubscriptionStateStore::new_at(&dir).unwrap();

    // Never observed invalid (default): allowed.
    assert!(store.offline_playback_allowed(0).unwrap());

    store.mark_invalid(100).unwrap();
    assert!(store
        .offline_playback_allowed(100 + GRACE_PERIOD_SECS - 1)
        .unwrap());
    assert!(!store
        .offline_playback_allowed(100 + GRACE_PERIOD_SECS)
        .unwrap());

    // A valid login verdict re-opens playback.
    store.mark_valid(100 + GRACE_PERIOD_SECS + 10).unwrap();
    assert!(store
        .offline_playback_allowed(100 + GRACE_PERIOD_SECS + 20)
        .unwrap());

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn subscription_mark_valid_clears_invalid_since() {
    let dir = unique_test_dir("subscription-valid");
    let store = SubscriptionStateStore::new_at(&dir).unwrap();

    store.mark_invalid(100).unwrap();
    store.mark_valid(200).unwrap();
    let state = store.get_state().unwrap();

    assert_eq!(state.invalid_since, None);
    assert_eq!(state.last_valid_at, Some(200));
    assert_eq!(state.last_checked_at, Some(200));
    assert!(!store.should_purge_offline_cache(1000).unwrap());
    let _ = std::fs::remove_dir_all(dir);
}
