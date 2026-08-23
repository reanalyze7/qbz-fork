use serde_json::Value;

use super::exit_code::exit_from_state;
use super::render::{render, render_playback};

fn logged_in_payload() -> Value {
    serde_json::json!({
        "version": "2.1.0", "api_version": 1, "uptime_secs": 259_200,
        "data_root": "/home/pi/.local/share/qbzd", "driver_tick_age_ms": 210,
        "auth": {"state": "logged_in", "user_id": 1234567, "subscription": "studio"},
        "audio": {"backend": "alsa", "configured_device": "hw:CARD=D30,DEV=0",
                  "device_present": true, "device_open": true,
                  "bit_perfect": "DirectHardware", "sample_rate": 192000, "bit_depth": 24},
        "playback": {"state": "playing", "track_id": 176544871, "title": "Spain",
                     "artist": "Chick Corea", "position": 192, "duration": 581,
                     "volume": 0.8, "muted": false, "queue_len": 14},
        "network": {"online": true},
        "last_errors": {"stream": null, "auth": null, "transport": null}
    })
}

#[test]
fn healthy_status_exits_zero() {
    assert_eq!(exit_from_state(&logged_in_payload()), 0);
}

#[test]
fn needs_auth_exits_four() {
    let mut p = logged_in_payload();
    p["auth"]["state"] = serde_json::json!("needs_auth");
    assert_eq!(exit_from_state(&p), 4);
}

#[test]
fn configured_but_absent_device_exits_five() {
    let mut p = logged_in_payload();
    p["audio"]["device_present"] = serde_json::json!(false);
    assert_eq!(exit_from_state(&p), 5);
    // system default (no configured device) never trips exit 5.
    let mut sysdef = logged_in_payload();
    sysdef["audio"]["configured_device"] = serde_json::Value::Null;
    sysdef["audio"]["device_present"] = serde_json::json!(false);
    assert_eq!(exit_from_state(&sysdef), 0);
}

#[test]
fn render_covers_the_composite_block() {
    let block = render(&logged_in_payload(), "127.0.0.1:8182");
    assert!(block.contains("qbzd 2.1.0 · api v1 · up 3d 0h · 127.0.0.1:8182"), "{block}");
    assert!(block.contains("auth      : logged in (user 1234567, studio)"), "{block}");
    assert!(block.contains("alsa hw:CARD=D30,DEV=0 · present · bit-perfect: DirectHardware · 192000 Hz / 24-bit"), "{block}");
    assert!(block.contains("playback  : playing · \"Spain\" — Chick Corea · 3:12 / 9:41 · vol 80% · queue 14"), "{block}");
    assert!(block.contains("last error: none"), "{block}");
}

#[test]
fn stopped_playback_renders_queue_only() {
    let mut p = logged_in_payload();
    p["playback"]["state"] = serde_json::json!("stopped");
    let line = render_playback(&p);
    assert_eq!(line, "stopped · queue 14");
}
