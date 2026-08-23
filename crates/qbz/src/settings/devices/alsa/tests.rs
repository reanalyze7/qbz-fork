use super::*;

#[test]
fn alsa_section_classification_matches_tauri() {
    // Empty id = synthetic "System default" -> Defaults.
    assert_eq!(alsa_section("", false, "System default"), AlsaSection::Defaults);
    // The qbz-audio `default` device -> Defaults.
    assert_eq!(alsa_section("default", true, "default"), AlsaSection::Defaults);
    // Direct hardware / digital PCMs -> Bit-perfect.
    assert_eq!(
        alsa_section("front:CARD=C20,DEV=0", false, "Cambridge"),
        AlsaSection::BitPerfect
    );
    assert_eq!(
        alsa_section("iec958:CARD=PCH,DEV=0", false, "S/PDIF"),
        AlsaSection::BitPerfect
    );
    assert_eq!(alsa_section("hw:0,0", false, "raw"), AlsaSection::BitPerfect);
    // Plugin hardware -> Plugin Hardware.
    assert_eq!(
        alsa_section("plughw:0,0", false, "converted"),
        AlsaSection::PluginHw
    );
    // sysdefault: and hdmi: route through plugins / are not in the
    // Tauri ALSA bit-perfect rule -> Other Outputs.
    assert_eq!(
        alsa_section("sysdefault:CARD=PCH", false, "onboard"),
        AlsaSection::Other
    );
    assert_eq!(
        alsa_section("hdmi:CARD=HDMI,DEV=0", false, "HDMI"),
        AlsaSection::Other
    );
}

#[test]
fn group_alsa_devices_orders_sections_and_places_headers() {
    // Deliberately scrambled input order.
    let rows = vec![
        DeviceRow {
            label: "HDMI out".into(),
            id: "hdmi:CARD=HDMI,DEV=0".into(),
            bp: false,
        },
        DeviceRow {
            label: "System default".into(),
            id: String::new(),
            bp: false,
        },
        DeviceRow {
            label: "Cambridge S/PDIF".into(),
            id: "iec958:CARD=C20,DEV=0".into(),
            bp: true,
        },
        DeviceRow {
            label: "Onboard".into(),
            id: "sysdefault:CARD=PCH".into(),
            bp: false,
        },
        DeviceRow {
            label: "Cambridge front".into(),
            id: "front:CARD=C20,DEV=0".into(),
            bp: true,
        },
    ];
    let list = group_alsa_devices(rows);
    // Section order: Defaults, Bit-perfect, Other.
    assert_eq!(
        list.ids,
        vec![
            "",
            "iec958:CARD=C20,DEV=0",
            "front:CARD=C20,DEV=0",
            "hdmi:CARD=HDMI,DEV=0",
            "sysdefault:CARD=PCH",
        ]
    );
    // Header appears on the first row of each section, empty otherwise.
    assert_eq!(
        list.groups,
        vec![
            "Defaults".to_string(),
            "Bit-perfect (Hardware / Digital)".to_string(),
            String::new(),
            "Other Outputs".to_string(),
            String::new(),
        ]
    );
    // BP badge only on the bit-perfect section.
    assert_eq!(list.bp, vec![false, true, true, false, false]);
    // All parallel lists stay index-aligned.
    assert_eq!(list.labels.len(), list.ids.len());
    assert_eq!(list.ids.len(), list.bp.len());
    assert_eq!(list.bp.len(), list.groups.len());
}
