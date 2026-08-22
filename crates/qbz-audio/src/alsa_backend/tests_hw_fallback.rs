use super::device_id::*;

#[test]
fn build_hw_fallback_id_rewrites_iec958_alias() {
    // The exact case from issue #331 — HifiBerry Digi2 Pro on RPi OS.
    assert_eq!(
        build_hw_fallback_id("iec958:CARD=sndrpihifiberry,DEV=0"),
        Some("hw:CARD=sndrpihifiberry,DEV=0".to_string())
    );
}

#[test]
fn build_hw_fallback_id_handles_every_alias_prefix() {
    assert_eq!(
        build_hw_fallback_id("front:CARD=Generic,DEV=0"),
        Some("hw:CARD=Generic,DEV=0".to_string())
    );
    assert_eq!(
        build_hw_fallback_id("sysdefault:CARD=PCH,DEV=0"),
        Some("hw:CARD=PCH,DEV=0".to_string())
    );
    assert_eq!(
        build_hw_fallback_id("hdmi:CARD=HDMI,DEV=3"),
        Some("hw:CARD=HDMI,DEV=3".to_string())
    );
}

#[test]
fn build_hw_fallback_id_defaults_dev_when_missing() {
    // Some alias forms don't carry DEV=; default to 0 so we still
    // produce a valid hw: id.
    assert_eq!(
        build_hw_fallback_id("iec958:CARD=NameOnly"),
        Some("hw:CARD=NameOnly,DEV=0".to_string())
    );
}

#[test]
fn build_hw_fallback_id_returns_none_for_non_alias_inputs() {
    // Raw hw:/plughw: ids don't need a fallback — they're already
    // the kernel PCM. `default` and unknown shapes don't match any
    // known alias prefix.
    assert_eq!(build_hw_fallback_id("hw:0,0"), None);
    assert_eq!(build_hw_fallback_id("plughw:0,0"), None);
    assert_eq!(build_hw_fallback_id("default"), None);
    assert_eq!(build_hw_fallback_id("pulse"), None);
    assert_eq!(build_hw_fallback_id(""), None);
}

#[test]
fn extract_card_name_from_device_handles_alias_prefixes() {
    // Alias forms — pure string parse, no /proc lookup involved.
    assert_eq!(
        extract_card_name_from_device("iec958:CARD=sndrpihifiberry,DEV=0"),
        Some("sndrpihifiberry".to_string())
    );
    assert_eq!(
        extract_card_name_from_device("front:CARD=Generic,DEV=0"),
        Some("Generic".to_string())
    );
    assert_eq!(
        extract_card_name_from_device("hdmi:CARD=HDMI_C,DEV=3"),
        Some("HDMI_C".to_string())
    );
    assert_eq!(
        extract_card_name_from_device("sysdefault:CARD=PCH,DEV=0"),
        Some("PCH".to_string())
    );
}

#[test]
fn extract_card_name_from_device_rejects_non_card_pcms() {
    // These shapes don't carry a CARD= component and should not
    // resolve to anything in /proc/asound.
    assert_eq!(extract_card_name_from_device("default"), None);
    assert_eq!(extract_card_name_from_device("pulse"), None);
    assert_eq!(extract_card_name_from_device("null"), None);
    assert_eq!(extract_card_name_from_device(""), None);
}
