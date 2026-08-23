use serde_json::json;

use super::fixtures::{bundle_with, cleanup, find, live, scratch, write_of};
use crate::settings::bundle::{plan, write_last_user_id, Bundle, BundleError, ImportOptions};

#[test]
fn bundle_json_roundtrips_flat() {
    // The on-disk shape is flat (§2.9): header + domains at the top level.
    let bundle = bundle_with(json!({ "audio": { "gapless_enabled": true } }));
    let text = bundle.to_json_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["source"]["profile"], "desktop");
    assert_eq!(v["audio"]["gapless_enabled"], true);

    let reparsed = Bundle::parse(&text).unwrap();
    assert_eq!(reparsed.schema_version, 1);
    assert!(reparsed.domains.contains_key("audio"));
}

#[test]
fn parse_rejects_missing_version() {
    let err = Bundle::parse(r#"{ "audio": {} }"#).unwrap_err();
    assert!(matches!(err, BundleError::VersionMalformed));
}

#[test]
fn secret_values_never_render_in_summary() {
    // §5.4: SECRET-class values are MASKED in the summary lines — the raw value
    // rides only the write list (terminal scrollback / CI logs must never see it).
    let p = scratch("mask");
    std::fs::create_dir_all(&p.data_root).unwrap();
    write_last_user_id(&p.data_root, 1).unwrap();
    let bundle = bundle_with(json!({
        "integrations": { "scrobblers": {
            "lastfm_session_key": "d580REALSECRET",
            "listenbrainz_token": ""
        } }
    }));
    let opts = ImportOptions { include_auth: true, ..Default::default() };

    let plan = plan(&bundle, &p, &opts, &live()).expect("plan");

    let key_line = find(&plan.applied, "integrations.scrobblers.lastfm_session_key").unwrap();
    assert_eq!(key_line.new, "(secret, applied)");
    let empty_line = find(&plan.applied, "integrations.scrobblers.listenbrainz_token").unwrap();
    assert_eq!(empty_line.new, "(empty)");

    // No rendered bucket line anywhere carries the raw secret.
    let rendered = format!("{:?} {:?} {:?}", plan.applied, plan.adapted, plan.skipped);
    assert!(!rendered.contains("d580REALSECRET"), "secret leaked into summary: {rendered}");

    // The write list still carries the real value so apply works.
    assert_eq!(
        write_of(&plan, "integrations.scrobblers.lastfm_session_key"),
        Some(&json!("d580REALSECRET"))
    );
    cleanup(&p);
}

#[test]
fn contains_secrets_keys_on_actual_secret_values() {
    // §3 warning trigger: any secret VALUE present (auth token OR a non-blank
    // scrobbler secret) — not the auth domain alone.
    let none = bundle_with(json!({ "audio": { "gapless_enabled": true } }));
    assert!(!none.contains_secrets());

    let blank = bundle_with(json!({
        "integrations": { "scrobblers": { "lastfm_session_key": "", "listenbrainz_token": "" } }
    }));
    assert!(!blank.contains_secrets(), "blank secrets are not secrets");

    let scrob = bundle_with(json!({
        "integrations": { "scrobblers": { "lastfm_session_key": "sk-live" } }
    }));
    assert!(scrob.contains_secrets());

    let auth = bundle_with(json!({ "auth": { "user_auth_token": "tok" } }));
    assert!(auth.contains_secrets());
}

#[test]
fn device_pick_names_the_backend() {
    // §5.4 prompt: "Available on Alsa:" — the pick carries the backend name.
    let p = scratch("pick-backend");
    let bundle = bundle_with(json!({
        "audio": { "output_device": "hw:9,9", "backend_type": "Alsa" }
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    let pick = plan.device_pick.expect("TTY plan must request a pick");
    assert_eq!(pick.backend, "Alsa");
    assert_eq!(pick.wanted, "hw:9,9");
    cleanup(&p);
}
