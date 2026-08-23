use qbz_core::CoreError;
use qbz_models::UserSession;

use crate::config::QbzdConfig;
use crate::state::AuthState;

use crate::daemon::session::{
    is_auth_rejection, latch_auth_error, new_shared, set_logged_in, set_needs_auth,
};

#[test]
fn is_auth_rejection_matches_only_explicit_rejections() {
    // Explicit rejections → clear the token.
    assert!(is_auth_rejection(&CoreError::Api(
        qbz_qobuz::ApiError::AuthenticationError("401".into())
    )));
    assert!(is_auth_rejection(&CoreError::Api(
        qbz_qobuz::ApiError::IneligibleUser
    )));
    // Network-class / other → KEEP the token (the boot-token-loss guard).
    assert!(!is_auth_rejection(&CoreError::Api(
        qbz_qobuz::ApiError::ServerError(503)
    )));
    assert!(!is_auth_rejection(&CoreError::Api(
        qbz_qobuz::ApiError::RateLimited(30)
    )));
    assert!(!is_auth_rejection(&CoreError::NotInitialized));
}

#[test]
fn no_credentials_enters_needs_auth() {
    let shared = new_shared(&QbzdConfig::default());
    set_needs_auth(&shared, None);
    let s = shared.lock().unwrap();
    assert_eq!(s.auth, AuthState::NeedsAuth);
    assert!(s.user_id.is_none());
    assert!(s.last_errors.auth.is_none());
}

#[test]
fn explicit_rejection_latches_and_needs_auth() {
    let shared = new_shared(&QbzdConfig::default());
    let err = CoreError::Api(qbz_qobuz::ApiError::AuthenticationError("401".into()));
    latch_auth_error(&shared, &err);
    set_needs_auth(&shared, Some(err));
    let s = shared.lock().unwrap();
    assert_eq!(s.auth, AuthState::NeedsAuth);
    assert!(s.last_errors.auth.is_some());
}

#[test]
fn logged_in_records_user_and_subscription() {
    let shared = new_shared(&QbzdConfig::default());
    let session = UserSession {
        user_auth_token: "secret".into(),
        user_id: 1234567,
        email: "a@b.c".into(),
        display_name: "Tester".into(),
        subscription_label: "studio".into(),
        subscription_valid_until: None,
        country_code: None,
        language_code: None,
    };
    set_logged_in(&shared, &session);
    let s = shared.lock().unwrap();
    assert_eq!(s.auth, AuthState::LoggedIn);
    assert_eq!(s.user_id, Some(1234567));
    assert_eq!(s.subscription.as_deref(), Some("studio"));
}
