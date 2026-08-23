// ── select-dacs (auto-detect + manual escape hatch) ────────────────────────

/// Plain, `Send` candidate produced on the worker thread.
pub struct DacCandidateData {
    pub id: String,
    pub description: String,
    pub bus: String,
    pub is_default: bool,
    pub looks_like_dac: bool,
    pub rates_label: String,
}

/// Heavy work (enumerate sinks via the pw-dump-robust path + probe rates for
/// the likely DACs). Runs on a blocking thread; returns plain data.
pub fn detect_blocking() -> Vec<DacCandidateData> {
    let devices = qbz_audio::backend::BackendManager::create_backend(
        qbz_audio::backend::AudioBackendType::PipeWire,
    )
    .and_then(|b| b.enumerate_devices())
    .unwrap_or_default();

    let mut out = Vec::new();
    for d in devices {
        let bus = d.device_bus.unwrap_or_default();
        let looks_like_dac = d.is_hardware && (bus == "usb" || bus == "pci");
        // Only probe rates for likely DACs (skip virtual/monitor sinks).
        let rates_label = if looks_like_dac {
            format_rates(&qbz_audio::query_dac_capabilities(&d.id).sample_rates)
        } else {
            String::new()
        };
        let description = if d.name.is_empty() { d.id.clone() } else { d.name };
        out.push(DacCandidateData {
            id: d.id,
            description,
            bus,
            is_default: d.is_default,
            looks_like_dac,
            rates_label,
        });
    }
    out
}

/// Validate a manually-pasted node.name (escape hatch). 1:1 with the Tauri
/// `validateNodeName` / `detectDacType`.
pub fn validate_node_name(name: &str) -> bool {
    let t = name.trim();
    !t.is_empty() && (t.contains("alsa_output") || t.contains("alsa_input"))
}

pub fn detect_dac_type(name: &str) -> &'static str {
    let l = name.to_lowercase();
    if l.contains("usb-") || l.contains(".usb") {
        "usb"
    } else if l.contains("pci-") || l.contains(".pci") {
        "pci"
    } else if l.contains("bluez") || l.contains("bluetooth") {
        "bluetooth"
    } else if l.contains("virtual") || l.contains("null") || l.contains("dummy") {
        "virtual"
    } else {
        "unknown"
    }
}

/// "44.1 / 96 / 192 kHz" from a rate list (kHz, .1 only when non-integer).
pub(super) fn format_rates(rates: &[u32]) -> String {
    if rates.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = rates
        .iter()
        .map(|&r| {
            if r % 1000 == 0 {
                format!("{}", r / 1000)
            } else {
                format!("{:.1}", r as f64 / 1000.0)
            }
        })
        .collect();
    format!("{} kHz", parts.join(" / "))
}
