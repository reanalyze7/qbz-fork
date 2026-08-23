use serde_json::json;

use qbz_audio::settings::AudioSettingsStore;
use qbz_audio::AudioBackendType;

use crate::settings::daemon_prefs;
use crate::settings::playback::PlaybackPreferencesStore;

use super::fixtures::{bundle_with, cleanup, find, find_contains, live, scratch};
use crate::settings::bundle::{apply, export, plan, ExportOptions, ExportSource, ImportOptions, ProfilePaths};

#[test]
fn roundtrip_same_box_is_noop() {
    // §7 acceptance invariant: export(daemon) → plan(same box) ⇒ adapted EMPTY,
    // applied values == current values, skipped == the always-skip caches only.
    let p = scratch("roundtrip");
    std::fs::create_dir_all(&p.data_root).unwrap();

    // Configure a realistic daemon (never "ask", never "remember_last", a real
    // device that this box's LiveSystem enumerates).
    {
        let audio = AudioSettingsStore::new_at(&p.data_root).unwrap();
        audio.set_backend_type(Some(AudioBackendType::Alsa)).unwrap();
        audio.set_output_device(Some("hw:1,0")).unwrap();
        audio.set_exclusive_mode(true).unwrap();
        audio.set_dsd_mode("dop").unwrap(); // a working DSD daemon
        audio.set_quality_fallback_behavior("always_fallback").unwrap();
        audio.set_gapless_enabled(true).unwrap();

        let pb = PlaybackPreferencesStore::new_at(&p.data_root).unwrap();
        pb.set_persist_session(true).unwrap();
    }

    let src = ExportSource::Daemon(ProfilePaths {
        config_root: p.config_root.clone(),
        data_root: p.data_root.clone(),
    });
    let bundle = export(src, &ExportOptions::default()).expect("export");

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    assert!(
        plan.adapted.is_empty(),
        "roundtrip must not adapt anything: {:?}",
        plan.adapted
    );
    assert!(plan.device_pick.is_none());
    // dsd dop survived without --trust-dsd (no-change short-circuit).
    assert_eq!(find(&plan.applied, "audio.dsd_mode").unwrap().new, "dop");
    assert_eq!(find(&plan.applied, "audio.output_device").unwrap().new, "hw:1,0");

    // Every skipped line must be one of the always-skip caches.
    for l in &plan.skipped {
        assert!(
            l.key.contains("device_max_sample_rate") || l.key.contains("device_sample_rate_limits"),
            "unexpected skip in roundtrip: {} ({})",
            l.key,
            l.why
        );
    }
    cleanup(&p);
}

#[test]
fn library_folders_skipped_on_daemon() {
    // §2.6: the P0 daemon has no local library.
    let p = scratch("folders");
    let bundle = bundle_with(json!({
        "library_folders": [ { "path": "/mnt/music", "network_fs": false } ]
    }));

    let plan = plan(&bundle, &p, &ImportOptions::default(), &live()).expect("plan");

    let line = find_contains(&plan.skipped, "library_folders").expect("skipped");
    assert!(line.why.contains("no local library"), "{}", line.why);
    cleanup(&p);
}

#[test]
fn apply_writes_are_idempotent_and_persist() {
    // §5.3 step 6: pure setter writes; a second apply is safe and lands the same
    // values. Exercises the whole applied/adapted write path end-to-end.
    let p = scratch("apply");
    let bundle = bundle_with(json!({
        "playback": { "autoplay_mode": "track_only", "persist_session": false },
        "audio": {
            "output_device": "hw:1,0",
            "backend_type": "Alsa",
            "gapless_enabled": true,
            "dsd_mode": "dop"
        },
        "prefs": { "streaming_quality": "cd" }
    }));
    let opts = ImportOptions { trust_dsd: true, ..Default::default() };

    let plan = plan(&bundle, &p, &opts, &live()).expect("plan");
    apply(&plan, &p, None).expect("apply once");
    apply(&plan, &p, None).expect("apply twice (idempotent)");

    let audio = AudioSettingsStore::new_at(&p.data_root).unwrap().get_settings().unwrap();
    assert_eq!(audio.output_device.as_deref(), Some("hw:1,0"));
    assert_eq!(audio.backend_type, Some(AudioBackendType::Alsa));
    assert!(audio.gapless_enabled);
    assert_eq!(audio.dsd_mode, "dop");

    let pb = PlaybackPreferencesStore::new_at(&p.data_root).unwrap().get_preferences().unwrap();
    assert!(!pb.persist_session);

    assert_eq!(daemon_prefs::load_at(&p.data_root).streaming_quality, "cd");
    cleanup(&p);
}
