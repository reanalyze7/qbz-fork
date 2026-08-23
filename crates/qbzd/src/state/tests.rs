use super::*;

#[test]
fn latched_errors_default_all_none() {
    let e = LatchedErrors::default();
    assert!(e.stream.is_none());
    assert!(e.auth.is_none());
    assert!(e.transport.is_none());
}

#[test]
fn auth_state_serializes_to_contract_strings() {
    // 02-cli-and-api.md §3.3.3: auth.state ∈ logged_in|needs_auth|restoring
    assert_eq!(
        serde_json::to_string(&AuthState::NeedsAuth).unwrap(),
        "\"needs_auth\""
    );
    assert_eq!(
        serde_json::to_string(&AuthState::Restoring).unwrap(),
        "\"restoring\""
    );
    assert_eq!(
        serde_json::to_string(&AuthState::LoggedIn).unwrap(),
        "\"logged_in\""
    );
}

#[test]
fn daemon_shared_holds_the_fields_the_status_route_needs() {
    // Construction smoke test: DaemonShared has no derive (Instant isn't
    // Serialize) so this is the only compile-time guard that the field
    // set/types stay what api::status::assemble expects.
    let shared = DaemonShared {
        auth: AuthState::LoggedIn,
        user_id: Some(1234567),
        subscription: Some("studio".into()),
        last_errors: LatchedErrors::default(),
        driver_last_tick: None,
        muted: false,
        premute_volume: 1.0,
        started_at: std::time::Instant::now(),
        startup_warnings: 0,
        credential_fingerprint: None,
        network_online: std::sync::atomic::AtomicBool::new(true),
    };
    assert_eq!(shared.auth, AuthState::LoggedIn);
    assert_eq!(shared.user_id, Some(1234567));
}

#[test]
fn network_online_latches_false_then_true() {
    // Pure latch semantics (01 §9.3): a real network-class failure flips
    // it false, a real success flips it back true — the exact two
    // transitions every call site above drives. Defaults true.
    let shared = DaemonShared {
        auth: AuthState::Restoring,
        user_id: None,
        subscription: None,
        last_errors: LatchedErrors::default(),
        driver_last_tick: None,
        muted: false,
        premute_volume: 1.0,
        started_at: std::time::Instant::now(),
        startup_warnings: 0,
        credential_fingerprint: None,
        network_online: std::sync::atomic::AtomicBool::new(true),
    };
    assert!(shared.network_online(), "defaults true (optimistic)");

    shared.set_network_online(false);
    assert!(!shared.network_online(), "set false -> reads back false");

    shared.set_network_online(true);
    assert!(shared.network_online(), "set true -> reads back true");
}

#[test]
fn token_fingerprint_is_stable_and_distinguishes_tokens() {
    let a = token_fingerprint("token-a");
    let a_again = token_fingerprint("token-a");
    let b = token_fingerprint("token-b");
    assert_eq!(a, a_again);
    assert_ne!(a, b);
}
