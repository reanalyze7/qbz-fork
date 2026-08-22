use std::fs;

use crate::crypto::decrypt_credentials_at;
use crate::keys::{derive_key_at, PortalKey};
use crate::oauth_token::{
    clear_oauth_token_at, load_oauth_token_at, oauth_token_file_present_at, save_oauth_token_at,
};
use crate::paths::{oauth_token_path_at, OAUTH_TOKEN_FILE_NAME};
use crate::private_file::write_private_file;

#[test]
fn oauth_token_roundtrip_at_root() {
    let dir = tempfile::tempdir().unwrap();
    save_oauth_token_at(dir.path(), "tok_abc123").unwrap();
    assert_eq!(
        load_oauth_token_at(dir.path()).unwrap().as_deref(),
        Some("tok_abc123")
    );
    clear_oauth_token_at(dir.path()).unwrap();
    assert_eq!(load_oauth_token_at(dir.path()).unwrap(), None);
    // isolation: nothing may touch the fixed ~/.config/qbz path — the salt and
    // token files must exist under the given root
    assert!(dir.path().join(".qbz-cred-salt").exists());
}

/// The daemon key must not depend on anything a session provides: whatever
/// `save_oauth_token_at` wrote has to come back out under the `Never`
/// policy, which is the only policy an init-started daemon can derive.
#[test]
fn daemon_token_key_is_session_independent() {
    let dir = tempfile::tempdir().unwrap();
    let with_portal = derive_key_at(dir.path(), PortalKey::Session).unwrap();
    let without_portal = derive_key_at(dir.path(), PortalKey::Never).unwrap();

    // No portal is reachable from a test process, so the two keys coincide
    // here; the assertion that matters is that `Never` is stable and that
    // the daemon round-trip below rides on it.
    assert_eq!(with_portal.len(), 32);
    assert_eq!(
        derive_key_at(dir.path(), PortalKey::Never).unwrap(),
        without_portal,
        "the portal-free key must be reproducible across calls"
    );

    save_oauth_token_at(dir.path(), "tok_headless").unwrap();
    let raw = fs::read_to_string(dir.path().join(OAUTH_TOKEN_FILE_NAME)).unwrap();
    assert_eq!(
        decrypt_credentials_at(dir.path(), PortalKey::Never, &raw)
            .unwrap()
            .email,
        "tok_headless",
        "a token saved by the daemon must decrypt with the portal-free key"
    );
}

/// `load_oauth_token_at` reports "absent" and "undecryptable" alike as
/// `None`, so the daemon needs this to tell a first run apart from a token
/// it cannot read (the difference between "log in" and "log in AGAIN").
#[test]
fn oauth_token_file_presence_is_reported_separately() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!oauth_token_file_present_at(dir.path()));

    write_private_file(&oauth_token_path_at(dir.path()), "   ").unwrap();
    assert!(
        !oauth_token_file_present_at(dir.path()),
        "a blank file is not a saved token"
    );

    save_oauth_token_at(dir.path(), "tok_present").unwrap();
    assert!(oauth_token_file_present_at(dir.path()));

    // Garbage that cannot decrypt still counts as present — that is exactly
    // the case the daemon must surface instead of "no token saved".
    write_private_file(&oauth_token_path_at(dir.path()), "not-even-json").unwrap();
    assert!(oauth_token_file_present_at(dir.path()));
    assert_eq!(load_oauth_token_at(dir.path()).unwrap(), None);
}
