//! Pure `pw-dump` JSON parsing, split out of `mod.rs` for the line-count
//! limit. Unit-testable against a captured fixture (see `tests.rs`).

use crate::backend::AudioDevice;

/// Parse `pw-dump` JSON into our `AudioDevice` list. Pure (no I/O) so it is
/// unit-testable against a captured fixture.
///
/// Selects objects of `type == "PipeWire:Interface:Node"` whose
/// `info.props["media.class"] == "Audio/Sink"`. The `node.name` is the id the
/// HiFi wizard otherwise asks the user to paste by hand. `device.bus` is read
/// from the node props (present in practice) and, when absent, cross-referenced
/// via the node's `device.id` against the `PipeWire:Interface:Device` objects.
/// `max_sample_rate` is intentionally left `None`: `pw-dump`'s `EnumFormat`
/// reports the CURRENTLY negotiated rate, not the device maximum — the
/// capability probe (`/proc/asound/.../stream0`) is the honest source for that.
pub(crate) fn parse_pw_dump_sinks(json: &str) -> Vec<AudioDevice> {
    let root: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("[PipeWire Backend] pw-dump JSON parse failed: {}", e);
            return Vec::new();
        }
    };
    let arr = match root.as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    // Default sink name lives in the "default" Metadata object.
    let mut default_sink: Option<String> = None;
    for obj in arr {
        let is_default_meta = obj.get("type").and_then(|v| v.as_str())
            == Some("PipeWire:Interface:Metadata")
            && obj
                .get("props")
                .and_then(|p| p.get("metadata.name"))
                .and_then(|v| v.as_str())
                == Some("default");
        if !is_default_meta {
            continue;
        }
        if let Some(entries) = obj.get("metadata").and_then(|m| m.as_array()) {
            for entry in entries {
                if entry.get("key").and_then(|v| v.as_str()) == Some("default.audio.sink") {
                    if let Some(name) = entry
                        .get("value")
                        .and_then(|v| v.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        default_sink = Some(name.to_string());
                    }
                }
            }
        }
    }

    // device.id -> device.bus, used only when the bus is not on the node itself.
    let mut device_bus: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    for obj in arr {
        if obj.get("type").and_then(|v| v.as_str()) != Some("PipeWire:Interface:Device") {
            continue;
        }
        if let Some(id) = obj.get("id").and_then(|v| v.as_i64()) {
            if let Some(bus) = obj
                .get("info")
                .and_then(|i| i.get("props"))
                .and_then(|p| p.get("device.bus"))
                .and_then(|v| v.as_str())
            {
                device_bus.insert(id, bus.to_string());
            }
        }
    }

    let mut devices = Vec::new();
    for obj in arr {
        if obj.get("type").and_then(|v| v.as_str()) != Some("PipeWire:Interface:Node") {
            continue;
        }
        let props = match obj.get("info").and_then(|i| i.get("props")) {
            Some(p) => p,
            None => continue,
        };
        if props.get("media.class").and_then(|v| v.as_str()) != Some("Audio/Sink") {
            continue;
        }
        let node_name = match props.get("node.name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let name = props
            .get("node.description")
            .and_then(|v| v.as_str())
            .or_else(|| props.get("node.nick").and_then(|v| v.as_str()))
            .map(|s| s.to_string())
            .unwrap_or_else(|| node_name.clone());
        let bus = props
            .get("device.bus")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                props
                    .get("device.id")
                    .and_then(|v| v.as_i64())
                    .and_then(|did| device_bus.get(&did).cloned())
            });
        // A real ALSA-backed sink = hardware (USB/PCI DAC, internal card).
        let is_hardware = props.get("device.api").and_then(|v| v.as_str()) == Some("alsa")
            || props
                .get("factory.name")
                .and_then(|v| v.as_str())
                .map(|f| f.contains("alsa"))
                .unwrap_or(false);
        let is_default = default_sink.as_deref() == Some(node_name.as_str());

        devices.push(AudioDevice {
            id: node_name,
            name,
            description: None,
            is_default,
            max_sample_rate: None,
            supported_sample_rates: None,
            device_bus: bus,
            is_hardware,
        });
    }
    devices
}
