use super::*;
use crate::error::ApiError;

/// With the offline gate closed, any public API method must fail fast
/// with the typed `ApiError::OfflineMode` — no network access, no
/// connect timeout. The gate is process-global and tests run in
/// parallel, so the shared lock serializes gate-touching tests and the
/// drop guard reopens the gate even if the test panics.
#[tokio::test]
async fn offline_gate_fails_fast_with_typed_error() {
    let _lock = crate::offline_gate::test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = crate::offline_gate::TestGateReset;

    crate::offline_gate::set_offline(true);

    let client = QobuzClient::new().expect("client construction is local-only");
    let started = std::time::Instant::now();
    let result = client.get_album_suggest("0060254735180").await;
    let elapsed = started.elapsed();

    let err = result.err().expect("offline gate must fail the request");
    assert!(
        matches!(err, ApiError::OfflineMode),
        "expected ApiError::OfflineMode, got: {err}"
    );
    // OfflineMode must never be retried by the retry layer.
    assert!(!err.is_transient(), "OfflineMode must be non-transient");
    // Fail-fast: well under the 10s connect timeout — proves no network.
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "offline gate took {elapsed:?} — it must not touch the network"
    );
}

/// The sign-in methods are EXEMPT from the offline gate: user-initiated
/// authentication is an explicit intent to reach Qobuz. On an
/// uninitialized client each method gets PAST the closed gate and fails
/// on the missing bundle tokens instead — never `ApiError::OfflineMode`,
/// and without touching the network.
#[tokio::test]
async fn offline_gate_exempts_login_methods() {
    let _lock = crate::offline_gate::test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = crate::offline_gate::TestGateReset;

    crate::offline_gate::set_offline(true);

    let client = QobuzClient::new().expect("client construction is local-only");

    let err = client
        .login("user@example.com", "pw")
        .await
        .err()
        .expect("uninitialized client must fail");
    assert!(
        !matches!(err, ApiError::OfflineMode),
        "login must bypass the offline gate, got: {err}"
    );

    let err = client
        .login_with_oauth_code("code")
        .await
        .err()
        .expect("uninitialized client must fail");
    assert!(
        !matches!(err, ApiError::OfflineMode),
        "login_with_oauth_code must bypass the offline gate, got: {err}"
    );

    let err = client
        .login_with_token("token")
        .await
        .err()
        .expect("uninitialized client must fail");
    assert!(
        !matches!(err, ApiError::OfflineMode),
        "login_with_token must bypass the offline gate, got: {err}"
    );
}
