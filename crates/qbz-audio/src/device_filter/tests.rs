use super::*;

fn run(rows: &[(&str, &str)]) -> Vec<(String, String)> {
    let owned: Vec<(String, String)> = rows
        .iter()
        .map(|(id, d)| (id.to_string(), d.to_string()))
        .collect();
    retain_real_outputs(owned, |r| r.0.as_str(), |r| r.1.as_str())
}

#[test]
fn discard_sink_sorted_to_end() {
    assert!(is_discard_sink(
        "Discard all samples (playback) or generate zero samples (capture)"
    ));
    // null appears FIRST in the raw list but must be emitted last.
    let out = run(&[
        ("null", "Discard all samples (playback) or generate zero samples (capture)"),
        ("default", "Default ALSA Output (currently PipeWire Media Server)"),
        ("front:CARD=PCH,DEV=0", "HDA Intel PCH, ALC3254 Analog"),
    ]);
    let ids: Vec<&str> = out.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(ids, vec!["default", "front:CARD=PCH,DEV=0", "null"]);
}

#[test]
fn collapses_plugin_wrappers_to_one_per_output() {
    // The exact shape of the user's listota: one analog output exposed
    // via many plugin ids that all share a description.
    let out = run(&[
        ("front:CARD=PCH,DEV=0", "HDA Intel PCH, ALC3254 Analog"),
        ("surround51:CARD=PCH,DEV=0", "HDA Intel PCH, ALC3254 Analog"),
        ("hw:CARD=PCH,DEV=0", "HDA Intel PCH, ALC3254 Analog"),
        ("plughw:CARD=PCH,DEV=0", "HDA Intel PCH, ALC3254 Analog"),
    ]);
    assert_eq!(out.len(), 1);
    // front: outranks surround/hw/plughw.
    assert_eq!(out[0].0, "front:CARD=PCH,DEV=0");
}

#[test]
fn keeps_genuinely_distinct_outputs() {
    let out = run(&[
        ("default", "Default ALSA Output (currently PipeWire Media Server)"),
        ("front:CARD=PCH,DEV=0", "HDA Intel PCH, ALC3254 Analog"),
        ("iec958:CARD=PCH,DEV=1", "HDA Intel PCH, ALC3254 Digital"),
        ("hdmi:CARD=PCH,DEV=3", "HDA Intel PCH, HDMI 0"),
        ("front:CARD=C20,DEV=0", "Cambridge Audio USB Audio 2.0, USB Audio"),
        ("surround40:CARD=C20,DEV=0", "Cambridge Audio USB Audio 2.0, USB Audio"),
    ]);
    let ids: Vec<&str> = out.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "default",
            "front:CARD=PCH,DEV=0",
            "iec958:CARD=PCH,DEV=1",
            "hdmi:CARD=PCH,DEV=3",
            "front:CARD=C20,DEV=0",
        ]
    );
}

#[test]
fn passes_pipewire_node_names_through() {
    let out = run(&[
        ("alsa_output.usb-Cambridge", "alsa_output.usb-Cambridge"),
        ("alsa_output.pci-0000_00_1f.3", "alsa_output.pci-0000_00_1f.3"),
    ]);
    assert_eq!(out.len(), 2);
}

#[test]
fn first_seen_order_is_preserved() {
    let out = run(&[
        ("hw:CARD=C20,DEV=0", "Cambridge Audio USB Audio 2.0, USB Audio"),
        ("default", "Default ALSA Output"),
        // Better-ranked id for Cambridge appears later; it wins the group
        // but the group keeps its first-seen position (before Default).
        ("front:CARD=C20,DEV=0", "Cambridge Audio USB Audio 2.0, USB Audio"),
    ]);
    let ids: Vec<&str> = out.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(ids, vec!["front:CARD=C20,DEV=0", "default"]);
}

#[test]
fn drops_blank_displays() {
    let out = run(&[("weird", "   "), ("default", "Default ALSA Output")]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].0, "default");
}
