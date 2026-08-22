use super::{parse_alsa_card_for_node, parse_hw_params};

// Real shape captured from /proc/asound/card1/pcm0p/sub0/hw_params while a
// 24/192 stream was open on a Cambridge USB DAC.
const ACTIVE: &str = "access: MMAP_INTERLEAVED\n\
format: S32_LE\n\
subformat: STD\n\
channels: 2\n\
rate: 192000 (192000/1)\n\
period_size: 2048\n\
buffer_size: 32768\n";

#[test]
fn parses_active_hw_params() {
    let nr = parse_hw_params(ACTIVE).expect("active stream parses");
    assert_eq!(nr.sample_rate, 192000);
    assert_eq!(nr.format, "S32_LE");
    assert_eq!(nr.channels, 2);
}

#[test]
fn idle_or_empty_yields_none() {
    assert!(parse_hw_params("closed").is_none());
    assert!(parse_hw_params("closed\n").is_none());
    assert!(parse_hw_params("").is_none());
    // No rate line -> None (we can't assert a negotiated rate).
    assert!(parse_hw_params("format: S16_LE\nchannels: 2\n").is_none());
}

#[test]
fn resolves_card_from_node_name() {
    let json = r#"[
      { "id": 53, "type": "PipeWire:Interface:Node",
        "info": { "props": {
          "media.class": "Audio/Sink",
          "node.name": "alsa_output.usb-Cambridge_Audio-00.analog-stereo",
          "api.alsa.pcm.card": 1, "alsa.card": 1 } } }
    ]"#;
    assert_eq!(
        parse_alsa_card_for_node(json, "alsa_output.usb-Cambridge_Audio-00.analog-stereo"),
        Some(1)
    );
    assert_eq!(parse_alsa_card_for_node(json, "alsa_output.unknown"), None);
}

#[test]
fn resolves_card_when_only_string_prop_present() {
    let json = r#"[
      { "id": 7, "type": "PipeWire:Interface:Node",
        "info": { "props": {
          "media.class": "Audio/Sink",
          "node.name": "alsa_output.pci-x.analog-stereo",
          "alsa.card": "0" } } }
    ]"#;
    assert_eq!(
        parse_alsa_card_for_node(json, "alsa_output.pci-x.analog-stereo"),
        Some(0)
    );
}
