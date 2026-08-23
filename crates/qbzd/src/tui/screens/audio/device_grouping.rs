use qbz_audio::{AudioBackendType, AudioDevice};

// ============================ device picker grouping (§3.2.2) ============================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AlsaSection {
    Defaults,
    BitPerfect,
    PluginHw,
    Other,
}

/// 1:1 with desktop `alsa_section` (`crates/qbz/src/settings.rs:286-301`).
pub(super) fn alsa_section(id: &str, is_default: bool, label: &str) -> AlsaSection {
    let id_l = id.to_ascii_lowercase();
    if id.is_empty() || id_l == "default" || is_default {
        AlsaSection::Defaults
    } else if id_l.starts_with("hw:")
        || id_l.starts_with("iec958:")
        || id_l.starts_with("front:card=")
        || label.to_ascii_lowercase().contains("bit-perfect")
    {
        AlsaSection::BitPerfect
    } else if id_l.starts_with("plughw:") {
        AlsaSection::PluginHw
    } else {
        AlsaSection::Other
    }
}

fn alsa_section_label(section: AlsaSection) -> &'static str {
    match section {
        AlsaSection::Defaults => "Defaults",
        AlsaSection::BitPerfect => "Bit-perfect (Hardware / Digital)",
        AlsaSection::PluginHw => "Plugin Hardware",
        AlsaSection::Other => "Other Outputs",
    }
}

/// 1:1 with desktop `device_is_bit_perfect` (`settings.rs:323-333`). The badge
/// uses the REAL `is_default` flag — unlike the grouping call (§3.2.2 edge case).
fn device_is_bit_perfect(backend: AudioBackendType, d: &AudioDevice) -> bool {
    match backend {
        AudioBackendType::Alsa => {
            let label = d.description.as_deref().unwrap_or(&d.name);
            alsa_section(&d.id, d.is_default, label) == AlsaSection::BitPerfect
        }
        AudioBackendType::PipeWire => d.is_hardware,
        AudioBackendType::Pulse | AudioBackendType::SystemDefault | AudioBackendType::Jack => false,
    }
}

/// One grouped, badged device row for the picker.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceEntry {
    pub label: String,
    pub id: String,
    pub bp: bool,
    /// Section header shown ABOVE this row (ALSA only, first row of a section).
    pub header: Option<String>,
}

/// Re-derive the picker rows (§3.2.2): a synthetic "System default" always
/// leads; ALSA regroups into the four sections (grouping passes is_default=false
/// like `group_alsa_devices:415`); non-ALSA stays flat, no headers.
pub fn group_devices(backend: AudioBackendType, devices: Vec<AudioDevice>) -> Vec<DeviceEntry> {
    // Build rows: System default first (empty id, never BP), then devices.
    let mut rows: Vec<DeviceEntry> = vec![DeviceEntry {
        label: "System default".to_string(),
        id: String::new(),
        bp: false,
        header: None,
    }];
    for d in &devices {
        let label = match d.description.as_deref() {
            Some(desc) if !desc.is_empty() => desc.to_string(),
            _ => d.name.clone(),
        };
        rows.push(DeviceEntry {
            label,
            id: d.id.clone(),
            bp: device_is_bit_perfect(backend, d),
            header: None,
        });
    }

    if backend != AudioBackendType::Alsa {
        return rows; // flat, no headers
    }

    // ALSA: stable-sort by section (grouping ALWAYS passes is_default=false,
    // desktop `group_alsa_devices:415`), then assign a header to each section's
    // first row.
    let mut indexed: Vec<(AlsaSection, DeviceEntry)> = rows
        .into_iter()
        .map(|r| (alsa_section(&r.id, false, &r.label), r))
        .collect();
    indexed.sort_by_key(|(section, _)| *section);

    let mut out = Vec::with_capacity(indexed.len());
    let mut prev: Option<AlsaSection> = None;
    for (section, mut row) in indexed {
        if prev != Some(section) {
            prev = Some(section);
            row.header = Some(alsa_section_label(section).to_string());
        }
        out.push(row);
    }
    out
}
