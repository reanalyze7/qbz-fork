use super::advance::render_advance;
use super::format::fmt_khz;
use super::mute::mute_body;
use super::now::render_now;
use super::seek::{parse_seek_arg, seek_body, SeekArg};
use super::volume::{fraction_to_pct, parse_volume_arg, pct_to_fraction, volume_body, VolumeArg};

#[test]
fn seek_arg_parses_absolute_relative_and_mmss() {
    assert_eq!(parse_seek_arg("90"), Ok(SeekArg::Absolute(90)));
    assert_eq!(parse_seek_arg("+30"), Ok(SeekArg::Delta(30)));
    assert_eq!(parse_seek_arg("-10"), Ok(SeekArg::Delta(-10)));
    assert_eq!(parse_seek_arg("1:23"), Ok(SeekArg::Absolute(83)));
    assert!(parse_seek_arg("1:99").is_err());
    assert!(parse_seek_arg("nonsense").is_err());
}

#[test]
fn seek_body_maps_to_legacy_position_or_additive_delta() {
    assert_eq!(seek_body(SeekArg::Absolute(90)), serde_json::json!({"position": 90}));
    assert_eq!(seek_body(SeekArg::Delta(-10)), serde_json::json!({"delta": -10}));
}

#[test]
fn volume_arg_parses_absolute_and_relative() {
    assert_eq!(parse_volume_arg("80"), Ok(VolumeArg::Absolute(80)));
    assert_eq!(parse_volume_arg("+5"), Ok(VolumeArg::Delta(5)));
    assert_eq!(parse_volume_arg("-5"), Ok(VolumeArg::Delta(-5)));
    assert!(parse_volume_arg("101").is_err());
    assert!(parse_volume_arg("nonsense").is_err());
}

#[test]
fn cli_percent_and_api_fraction_convert_both_ways() {
    // 02 §2.2 — "CLI speaks 0-100; the API speaks 0.0-1.0".
    assert_eq!(pct_to_fraction(80), 0.8);
    assert_eq!(fraction_to_pct(0.8), 80);
    assert_eq!(fraction_to_pct(0.75), 75);
    assert_eq!(pct_to_fraction(0), 0.0);
    assert_eq!(pct_to_fraction(100), 1.0);
}

#[test]
fn volume_body_converts_absolute_and_delta_percent_to_fraction() {
    assert_eq!(volume_body(VolumeArg::Absolute(80)), serde_json::json!({"volume": 0.8}));
    assert_eq!(volume_body(VolumeArg::Delta(5)), serde_json::json!({"delta": 0.05}));
    assert_eq!(volume_body(VolumeArg::Delta(-5)), serde_json::json!({"delta": -0.05}));
}

#[test]
fn mute_body_maps_bare_on_off_to_the_three_states() {
    assert_eq!(mute_body(None).unwrap(), serde_json::json!({"mute": "toggle"}));
    assert_eq!(mute_body(Some("on")).unwrap(), serde_json::json!({"mute": "on"}));
    assert_eq!(mute_body(Some("off")).unwrap(), serde_json::json!({"mute": "off"}));
    assert!(mute_body(Some("bogus")).is_err());
}

#[test]
fn render_now_matches_the_documented_playing_example() {
    // 02 §2.2 `now --json` example, human line: "playing · Chick Corea –
    // Spain · 3:12/9:41 · 96kHz/24bit · vol 80%".
    let v = serde_json::json!({
        "playback": {
            "is_playing": true, "position": 192, "duration": 581,
            "volume": 0.8, "muted": false, "sample_rate": 96000, "bit_depth": 24,
            "queue_len": 14
        },
        "track": {"id": 176544871, "title": "Spain", "artist": "Chick Corea"}
    });
    assert_eq!(
        render_now(&v),
        "playing · Chick Corea – Spain · 3:12/9:41 · 96kHz/24bit · vol 80%"
    );
}

#[test]
fn render_now_stopped_state_shows_queue_count_and_no_track() {
    let v = serde_json::json!({
        "playback": {"is_playing": false, "position": 0, "duration": 0,
                     "volume": 0.8, "muted": false, "queue_len": 14},
        "track": null
    });
    assert_eq!(render_now(&v), "stopped · queue 14 tracks");
}

#[test]
fn render_advance_shows_landing_track_or_queue_finished() {
    let landing = serde_json::json!({"artist": "Chick Corea", "title": "500 Miles High"});
    assert_eq!(render_advance(&landing), "-> Chick Corea – 500 Miles High");
    assert_eq!(render_advance(&serde_json::Value::Null), "queue finished");
}

#[test]
fn render_advance_shows_spawn_and_ack_queued() {
    let ack = serde_json::json!({"queued": true, "direction": "next"});
    assert_eq!(render_advance(&ack), "-> queued (next)");
    let ack_prev = serde_json::json!({"queued": true, "direction": "previous"});
    assert_eq!(render_advance(&ack_prev), "-> queued (previous)");
}

#[test]
fn fmt_khz_rounds_only_when_not_exact() {
    assert_eq!(fmt_khz(96000), "96kHz");
    assert_eq!(fmt_khz(192000), "192kHz");
    assert_eq!(fmt_khz(44100), "44.1kHz");
}
