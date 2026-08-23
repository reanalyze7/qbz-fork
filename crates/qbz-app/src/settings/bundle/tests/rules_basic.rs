use serde_json::json;

use super::fixtures::{bundle_with, cleanup, find, live, scratch, write_of};
use crate::settings::bundle::{plan, ImportOptions};

#[test]
fn portable_fields_apply_verbatim() {
    // §3 PORTABLE: playback.*, the audio portable subset, prefs.streaming_quality.
    let p = scratch("portable");
    let bundle = bundle_with(json!({
        "playback": {
            "autoplay_mode": "infinite",
            "show_context_icon": false,
            "persist_session": true,
            "resume_playback_position": false
        },
        "audio": {
            "gapless_enabled": true,
            "stream_buffer_seconds": 4,
            "normalization_target_lufs": -18.0,
            "sync_audio_on_startup": true
        },
        "prefs": { "streaming_quality": "hires_plus" }
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    assert!(plan.adapted.is_empty(), "portable fields must not adapt: {:?}", plan.adapted);
    assert_eq!(find(&plan.applied, "playback.autoplay_mode").unwrap().new, "infinite");
    assert_eq!(find(&plan.applied, "playback.show_context_icon").unwrap().new, "false");
    assert_eq!(find(&plan.applied, "audio.gapless_enabled").unwrap().new, "true");
    assert_eq!(find(&plan.applied, "audio.stream_buffer_seconds").unwrap().new, "4");
    assert_eq!(find(&plan.applied, "prefs.streaming_quality").unwrap().new, "hires_plus");
    cleanup(&p);
}

#[test]
fn volume_is_never_class_even_hand_added() {
    // §1 corollary: a hand-edited bundle with `volume` anywhere -> skipped, always.
    let p = scratch("volume");
    let bundle = bundle_with(json!({
        "audio": { "volume": 0.8, "gapless_enabled": true },
        "prefs": { "streaming_quality": "cd", "volume": 0.9 }
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    let a = find(&plan.skipped, "audio.volume").expect("audio.volume skipped");
    assert!(a.why.contains("never imported"), "{}", a.why);
    let pr = find(&plan.skipped, "prefs.volume").expect("prefs.volume skipped");
    assert!(pr.why.contains("never imported"), "{}", pr.why);
    // No write ever carries a volume.
    assert!(plan.writes.iter().all(|(k, _)| !k.contains("volume")));
    cleanup(&p);
}

#[test]
fn dsd_downgrades_without_trust_flag() {
    // §5.3 step 4: dop/native → convert unless --trust-dsd (current is convert,
    // so this is a CHANGE — the no-change short-circuit does not fire).
    let p = scratch("dsd");
    let bundle = bundle_with(json!({ "audio": { "dsd_mode": "dop" } }));

    let plan_no_trust = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");
    let line = find(&plan_no_trust.adapted, "audio.dsd_mode").expect("dsd adapted");
    assert_eq!(line.old.as_deref(), Some("dop"));
    assert_eq!(line.new, "convert");
    assert_eq!(write_of(&plan_no_trust, "audio.dsd_mode"), Some(&json!("convert")));

    let opts = ImportOptions { trust_dsd: true, ..Default::default() };
    let plan_trust = plan(&bundle, &p, &opts, &live()).expect("plan");
    assert!(find(&plan_trust.adapted, "audio.dsd_mode").is_none());
    assert_eq!(find(&plan_trust.applied, "audio.dsd_mode").unwrap().new, "dop");
    cleanup(&p);
}

#[test]
fn ask_maps_to_always_fallback_in_adapted() {
    // §5.5: "ask" needs a UI the daemon lacks → always_fallback, in adapted,
    // never a silent skip.
    let p = scratch("ask");
    let bundle = bundle_with(json!({ "audio": { "quality_fallback_behavior": "ask" } }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    let line = find(&plan.adapted, "audio.quality_fallback_behavior").expect("adapted");
    assert_eq!(line.old.as_deref(), Some("ask"));
    assert_eq!(line.new, "always_fallback");
    assert!(find(&plan.skipped, "audio.quality_fallback_behavior").is_none());
    cleanup(&p);
}

#[test]
fn unknown_field_skipped_never_error() {
    // §5.3 step 3 / §7: unknown keys → skipped, never an error.
    let p = scratch("unknown");
    let bundle = bundle_with(json!({
        "audio": { "some_future_flag": true },
        "brand_new_domain": { "x": 1 }
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("must not error");

    let a = find(&plan.skipped, "audio.some_future_flag").expect("unknown audio key skipped");
    assert!(a.why.contains("unknown field"), "{}", a.why);
    let d = find(&plan.skipped, "brand_new_domain").expect("unknown domain skipped");
    assert!(d.why.contains("unknown field"), "{}", d.why);
    cleanup(&p);
}
