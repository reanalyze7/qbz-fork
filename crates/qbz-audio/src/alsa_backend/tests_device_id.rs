use super::device_id::*;

#[test]
fn is_card_present_in_proc_short_circuits_on_unparseable_ids() {
    // For inputs without a CARD= component, the helper must short-
    // circuit to false without ever touching /proc — so this stays
    // safe regardless of host audio configuration.
    assert!(!is_card_present_in_proc("default"));
    assert!(!is_card_present_in_proc("pulse"));
    assert!(!is_card_present_in_proc("null"));
    assert!(!is_card_present_in_proc(""));
}

#[test]
fn is_known_pcm_id_keeps_only_lookup_targets() {
    // Positive: every shape downstream code actually queries.
    assert!(is_known_pcm_id("default"));
    assert!(is_known_pcm_id("sysdefault:CARD=PCH"));
    assert!(is_known_pcm_id("front:CARD=Generic,DEV=0"));
    assert!(is_known_pcm_id("hdmi:CARD=HDMI,DEV=3"));
    assert!(is_known_pcm_id("iec958:CARD=sndrpihifiberry,DEV=0"));

    // Negative: virtual PCMs that only emit noise when probed.
    assert!(!is_known_pcm_id("dmix:CARD=PCH,DEV=0"));
    assert!(!is_known_pcm_id("dsnoop:CARD=PCH,DEV=0"));
    assert!(!is_known_pcm_id("route:CARD=PCH"));
    assert!(!is_known_pcm_id("surround51:CARD=PCH"));
    assert!(!is_known_pcm_id("pulse"));
    assert!(!is_known_pcm_id("null"));
    assert!(!is_known_pcm_id("hw:0,0"));
    assert!(!is_known_pcm_id("plughw:0,0"));
}

#[test]
fn raw_open_ids_derives_raw_pair_from_front_alias() {
    // Discussion #641 — snd-aloop declares `front` for DEV=0 only, so a
    // saved `front:CARD=Loopback,DEV=1` fails to open with ENOENT. The
    // derived raw ids take CARD=/DEV= straight to the kernel PCM.
    assert_eq!(
        raw_open_ids("front:CARD=Loopback,DEV=1"),
        Some((
            "hw:CARD=Loopback,DEV=1".to_string(),
            "plughw:CARD=Loopback,DEV=1".to_string()
        ))
    );
    assert_eq!(
        raw_open_ids("front:CARD=ZH3,DEV=0"),
        Some((
            "hw:CARD=ZH3,DEV=0".to_string(),
            "plughw:CARD=ZH3,DEV=0".to_string()
        ))
    );
}

#[test]
fn raw_open_ids_defaults_dev_when_missing() {
    assert_eq!(
        raw_open_ids("front:CARD=Loopback"),
        Some((
            "hw:CARD=Loopback,DEV=0".to_string(),
            "plughw:CARD=Loopback,DEV=0".to_string()
        ))
    );
}

#[test]
fn raw_open_ids_passes_through_non_alias_ids() {
    // Raw and virtual ids keep the caller's pre-existing handling.
    assert_eq!(raw_open_ids("hw:0,0"), None);
    assert_eq!(raw_open_ids("plughw:1,0"), None);
    assert_eq!(raw_open_ids("default"), None);
    assert_eq!(raw_open_ids("pulse"), None);
}
