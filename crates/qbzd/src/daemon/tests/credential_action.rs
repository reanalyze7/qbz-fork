use crate::state::AuthState;

use crate::daemon::reload::{decide_credential_action, CredentialAction};

#[test]
fn credential_action_noop_when_absent_and_already_needs_auth() {
    assert_eq!(
        decide_credential_action(AuthState::NeedsAuth, None, None),
        CredentialAction::NoOp
    );
}

#[test]
fn credential_action_enters_needs_auth_when_file_cleared_while_logged_in() {
    // The `qbzd logout` case: file now absent, daemon still thinks it's
    // LoggedIn (or mid-Restoring) — must tear down.
    let fp = crate::state::token_fingerprint("old-token");
    assert_eq!(
        decide_credential_action(AuthState::LoggedIn, Some(fp), None),
        CredentialAction::EnterNeedsAuth
    );
    assert_eq!(
        decide_credential_action(AuthState::Restoring, None, None),
        CredentialAction::EnterNeedsAuth
    );
}

#[test]
fn credential_action_noop_when_token_unchanged_and_logged_in() {
    let fp = crate::state::token_fingerprint("same-token");
    assert_eq!(
        decide_credential_action(AuthState::LoggedIn, Some(fp), Some("same-token".into())),
        CredentialAction::NoOp
    );
}

#[test]
fn credential_action_applies_new_token_out_of_needs_auth() {
    assert_eq!(
        decide_credential_action(AuthState::NeedsAuth, None, Some("fresh-token".into())),
        CredentialAction::Apply("fresh-token".into())
    );
}

#[test]
fn credential_action_applies_changed_token_while_already_logged_in() {
    // Account-switch / re-login case: fingerprint differs even though the
    // daemon is already LoggedIn.
    let old_fp = crate::state::token_fingerprint("old-token");
    assert_eq!(
        decide_credential_action(AuthState::LoggedIn, Some(old_fp), Some("new-token".into())),
        CredentialAction::Apply("new-token".into())
    );
}

#[test]
fn credential_action_applies_when_fingerprint_missing_even_if_marked_logged_in() {
    // Defensive: a LoggedIn state with no recorded fingerprint (should not
    // happen post-T11, but a stale/older state) must not be treated as a
    // match — never silently skip a real token on disk.
    assert_eq!(
        decide_credential_action(AuthState::LoggedIn, None, Some("token".into())),
        CredentialAction::Apply("token".into())
    );
}
