use serde_json::{json, Value};

use super::fixtures::{bundle_with, cleanup, find, live, scratch, write_of};
use crate::settings::bundle::{plan, ImportOptions};

#[test]
fn missing_device_non_tty_falls_back_safe() {
    // §5.3 step 4 non-TTY: backend→SystemDefault, device→null, intent flags→false,
    // all reported in adapted; no device_pick and never hangs.
    let p = scratch("nontty");
    let bundle = bundle_with(json!({
        "audio": {
            "output_device": "hw:9,9",
            "backend_type": "Jack",
            "exclusive_mode": true,
            "dac_passthrough": true
        }
    }));
    let opts = ImportOptions { non_tty: true, ..Default::default() };

    let plan = plan(&bundle, &p, &opts, &live()).expect("plan");

    assert!(plan.device_pick.is_none(), "non-tty must not request a pick");

    let dev = find(&plan.adapted, "audio.output_device").expect("device adapted");
    assert_eq!(dev.old.as_deref(), Some("hw:9,9"));
    assert_eq!(write_of(&plan, "audio.output_device"), Some(&Value::Null));

    let backend = find(&plan.adapted, "audio.backend_type").expect("backend adapted");
    assert_eq!(write_of(&plan, "audio.backend_type"), Some(&json!("SystemDefault")));
    assert!(backend.new.contains("SystemDefault"));

    for flag in ["audio.exclusive_mode", "audio.dac_passthrough"] {
        let l = find(&plan.adapted, flag).unwrap_or_else(|| panic!("{flag} must adapt"));
        assert_eq!(l.new, "false", "{flag} must reset to false");
        assert_eq!(write_of(&plan, flag), Some(&Value::Bool(false)));
    }
    cleanup(&p);
}

#[test]
fn found_device_applies_verbatim() {
    // A machine field that validates cleanly lands in APPLIED (§5.4).
    let p = scratch("found");
    let bundle = bundle_with(json!({
        "audio": {
            "output_device": "hw:1,0",
            "backend_type": "Alsa",
            "exclusive_mode": true
        }
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    assert!(plan.device_pick.is_none());
    assert_eq!(find(&plan.applied, "audio.output_device").unwrap().new, "hw:1,0");
    assert_eq!(find(&plan.applied, "audio.backend_type").unwrap().new, "Alsa");
    assert_eq!(find(&plan.applied, "audio.exclusive_mode").unwrap().new, "true");
    assert!(plan.adapted.is_empty(), "clean validation must not adapt: {:?}", plan.adapted);
    cleanup(&p);
}

#[test]
fn absent_fields_leave_target_untouched() {
    // §7: only present fields have effects.
    let p = scratch("absent");
    let bundle = bundle_with(json!({ "playback": { "autoplay_mode": "track_only" } }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    // Exactly one write (the single present field); nothing audio/etc.
    assert_eq!(plan.writes.len(), 1);
    assert_eq!(plan.writes[0].0, "playback.autoplay_mode");
    assert!(plan.writes.iter().all(|(k, _)| !k.starts_with("audio.")));
    cleanup(&p);
}

#[test]
fn machine_caches_always_skipped() {
    // §2.2/§3: source-machine device caches are meaningless on the target.
    let p = scratch("caches");
    let bundle = bundle_with(json!({
        "audio": {
            "device_max_sample_rate": 768000,
            "device_sample_rate_limits": { "hw:4,0": 768000 }
        }
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    for key in ["audio.device_max_sample_rate", "audio.device_sample_rate_limits"] {
        let l = find(&plan.skipped, key).unwrap_or_else(|| panic!("{key} must skip"));
        assert!(l.why.contains("device cache"), "{}", l.why);
    }
    assert!(plan
        .writes
        .iter()
        .all(|(k, _)| !k.contains("device_max_sample_rate") && !k.contains("device_sample_rate_limits")));
    cleanup(&p);
}
