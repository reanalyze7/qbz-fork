use super::parse::parse_pw_dump_sinks;

// Minimal fixture mirroring the real `pw-dump` shape (Cambridge USB DAC +
// internal PCI card + a capture source that must be filtered out).
const FIXTURE: &str = r#"[
  {
    "id": 30,
    "type": "PipeWire:Interface:Metadata",
    "props": { "metadata.name": "default" },
    "metadata": [
      { "subject": 0, "key": "default.audio.sink", "type": "Spa:String:JSON",
        "value": { "name": "alsa_output.usb-Cambridge_Audio-00.analog-stereo" } }
    ]
  },
  {
    "id": 52, "type": "PipeWire:Interface:Device",
    "info": { "props": { "device.bus": "usb", "device.api": "alsa" } }
  },
  {
    "id": 53, "type": "PipeWire:Interface:Node",
    "info": { "props": {
      "media.class": "Audio/Sink",
      "node.name": "alsa_output.usb-Cambridge_Audio-00.analog-stereo",
      "node.description": "DacMagic Plus Analog Stereo",
      "device.id": 52, "device.bus": "usb", "device.api": "alsa",
      "factory.name": "api.alsa.pcm.sink"
    } }
  },
  {
    "id": 60, "type": "PipeWire:Interface:Device",
    "info": { "props": { "device.bus": "pci", "device.api": "alsa" } }
  },
  {
    "id": 61, "type": "PipeWire:Interface:Node",
    "info": { "props": {
      "media.class": "Audio/Sink",
      "node.name": "alsa_output.pci-0000_00_1f.3.analog-stereo",
      "node.description": "Built-in Audio Analog Stereo",
      "device.id": 60, "device.api": "alsa",
      "factory.name": "api.alsa.pcm.sink"
    } }
  },
  {
    "id": 70, "type": "PipeWire:Interface:Node",
    "info": { "props": {
      "media.class": "Audio/Source",
      "node.name": "alsa_input.usb-Cambridge_Audio-00.analog-stereo"
    } }
  }
]"#;

#[test]
fn parses_sinks_only_with_node_names_bus_and_default() {
    let devs = parse_pw_dump_sinks(FIXTURE);
    // The Audio/Source must be excluded.
    assert_eq!(devs.len(), 2, "should parse exactly the two Audio/Sink nodes");

    let usb = devs
        .iter()
        .find(|d| d.id == "alsa_output.usb-Cambridge_Audio-00.analog-stereo")
        .expect("usb sink present");
    assert_eq!(usb.name, "DacMagic Plus Analog Stereo");
    assert_eq!(usb.device_bus.as_deref(), Some("usb")); // read from node props
    assert!(usb.is_hardware);
    assert!(usb.is_default, "usb sink is the default per Metadata");
    assert!(usb.max_sample_rate.is_none(), "rate comes from the capability probe, not pw-dump");

    let pci = devs
        .iter()
        .find(|d| d.id == "alsa_output.pci-0000_00_1f.3.analog-stereo")
        .expect("pci sink present");
    // Bus absent on the node -> cross-referenced via device.id 60.
    assert_eq!(pci.device_bus.as_deref(), Some("pci"));
    assert!(pci.is_hardware);
    assert!(!pci.is_default);
}

#[test]
fn empty_or_garbage_json_yields_no_devices() {
    assert!(parse_pw_dump_sinks("not json").is_empty());
    assert!(parse_pw_dump_sinks("[]").is_empty());
    assert!(parse_pw_dump_sinks("{}").is_empty());
}
