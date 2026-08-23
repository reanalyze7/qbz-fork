use qbz_audio::AudioBackendType;

use crate::tui::screens::audio::device_grouping::{alsa_section, group_devices, AlsaSection};

use super::fixtures::dev;

// ---- device grouping §3.2.2 ----

#[test]
fn non_alsa_is_flat_with_system_default_first_no_headers() {
    let devices = vec![dev("pw-node-1", "USB DAC", false, true)];
    let rows = group_devices(AudioBackendType::PipeWire, devices);
    assert_eq!(rows[0].id, "", "System default leads");
    assert!(rows.iter().all(|r| r.header.is_none()), "no headers off ALSA");
    assert!(rows[1].bp, "PipeWire hardware node is BP");
}

#[test]
fn alsa_groups_into_four_sections_in_order() {
    let devices = vec![
        dev("plughw:CARD=D30", "Plug D30", false, false),
        dev("hw:CARD=D30,DEV=0", "Topping D30", false, false),
        dev("sysdefault:CARD=x", "Sys x", false, false),
    ];
    let rows = group_devices(AudioBackendType::Alsa, devices);
    // First section is Defaults (the synthetic system-default row).
    assert_eq!(rows[0].header.as_deref(), Some("Defaults"));
    let headers: Vec<&str> = rows.iter().filter_map(|r| r.header.as_deref()).collect();
    assert_eq!(
        headers,
        vec![
            "Defaults",
            "Bit-perfect (Hardware / Digital)",
            "Plugin Hardware",
            "Other Outputs"
        ]
    );
}

#[test]
fn alsa_hw_device_gets_bp_badge() {
    let devices = vec![dev("hw:CARD=D30,DEV=0", "Topping D30", false, false)];
    let rows = group_devices(AudioBackendType::Alsa, devices);
    let d = rows.iter().find(|r| r.id.starts_with("hw:")).unwrap();
    assert!(d.bp, "hw: ALSA device is bit-perfect");
}

#[test]
fn is_default_hw_lands_in_bitperfect_section_but_gets_no_badge() {
    // §3.2.2 edge case (1:1 desktop): a device with is_default=true and an
    // hw: id → Bit-perfect SECTION (grouping passes is_default=false) but NO
    // badge (badge sees is_default=true → Defaults).
    let devices = vec![dev("hw:CARD=D30,DEV=0", "Default D30", true, false)];
    let rows = group_devices(AudioBackendType::Alsa, devices);
    let d = rows.iter().find(|r| r.id.starts_with("hw:")).unwrap();
    assert!(!d.bp, "badge predicate uses REAL is_default → no [BP]");
    // Its section header (if it's first of its section) is Bit-perfect.
    assert_eq!(
        alsa_section("hw:CARD=D30,DEV=0", false, "Default D30"),
        AlsaSection::BitPerfect,
        "grouping predicate uses is_default=false → Bit-perfect section"
    );
}
