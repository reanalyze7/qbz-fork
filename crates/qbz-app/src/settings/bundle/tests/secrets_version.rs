use serde_json::json;

use super::fixtures::{bundle_with, cleanup, find, live, scratch};
use crate::settings::bundle::{plan, write_last_user_id, BundleError, ImportOptions};

#[test]
fn secrets_double_gate() {
    // §3/§6: secrets present but no import-side --include-auth → skipped, and the
    // auth token is NOT queued for validation.
    let p = scratch("secrets");
    // A user must exist for the scrobbler secret to reach the gate (else §5.7
    // no-user skip fires first).
    std::fs::create_dir_all(&p.data_root).unwrap();
    write_last_user_id(&p.data_root, 1234567).unwrap();

    let bundle = bundle_with(json!({
        "integrations": { "scrobblers": { "lastfm_session_key": "d580secret" } },
        "auth": { "user_auth_token": "Bo4Asecret", "user_id": 1234567 }
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    let token_line = find(&plan.skipped, "auth.user_auth_token").expect("auth token skipped");
    assert!(token_line.why.contains("--include-auth"), "{}", token_line.why);
    assert!(plan.auth_token.is_none(), "token must not be queued without the gate");

    let secret = find(&plan.skipped, "integrations.scrobblers.lastfm_session_key")
        .expect("scrobbler secret skipped");
    assert!(secret.why.contains("--include-auth"), "{}", secret.why);
    cleanup(&p);
}

#[test]
fn secret_applies_with_gate() {
    // The other half of the double gate: with --include-auth the token is queued
    // for validation and the scrobbler secret applies.
    let p = scratch("secret-gate");
    std::fs::create_dir_all(&p.data_root).unwrap();
    write_last_user_id(&p.data_root, 1234567).unwrap();
    let bundle = bundle_with(json!({
        "integrations": { "scrobblers": { "lastfm_session_key": "d580secret" } },
        "auth": { "user_auth_token": "Bo4Asecret", "user_id": 1234567 }
    }));
    let opts = ImportOptions { include_auth: true, ..Default::default() };

    let plan = plan(&bundle, &p, &opts, &live()).expect("plan");

    assert_eq!(plan.auth_token.as_deref(), Some("Bo4Asecret"));
    assert_eq!(plan.bundle_user_id, Some(1234567));
    assert!(find(&plan.applied, "integrations.scrobblers.lastfm_session_key").is_some());
    cleanup(&p);
}

#[test]
fn version_gate_rejects_newer() {
    // §5.6: a bundle newer than this importer is rejected.
    let p = scratch("version");
    let mut bundle = bundle_with(json!({ "audio": { "gapless_enabled": true } }));
    bundle.schema_version = 2;

    let err = plan(&bundle, &p, &ImportOptions::default(), &live()).unwrap_err();
    match err {
        BundleError::VersionTooNew { bundle, supported } => {
            assert_eq!(bundle, 2);
            assert_eq!(supported, 1);
        }
        other => panic!("expected VersionTooNew, got {other:?}"),
    }
    cleanup(&p);
}
