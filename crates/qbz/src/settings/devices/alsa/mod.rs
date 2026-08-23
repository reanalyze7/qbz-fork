//! ALSA-specific device classification and section grouping.

#[cfg(test)]
mod tests;

use qbz_audio::backend::AudioBackendType;

use super::{DeviceList, DeviceRow};

/// Which ALSA dropdown section a device belongs to. Mirrors the Tauri
/// `DeviceDropdown.svelte` ALSA grouping (`Defaults`, `Bit-perfect
/// (Hardware / Digital)`, `Plugin Hardware`, `Other Outputs`), in that
/// display order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum AlsaSection {
    Defaults,
    BitPerfect,
    PluginHw,
    Other,
}

/// Classify an ALSA device into its dropdown section, matching the Tauri
/// `DeviceDropdown.svelte` ALSA branch:
///  - the "System default" entry (empty id) and `default` / `is_default`
///    devices → Defaults;
///  - `hw:`, `iec958:`, `front:CARD=` ids, or any label containing
///    "bit-perfect" → Bit-perfect (Hardware / Digital);
///  - `plughw:` ids → Plugin Hardware;
///  - everything else (`sysdefault:`, `hdmi:`, ...) → Other Outputs.
fn alsa_section(id: &str, is_default: bool, label: &str) -> AlsaSection {
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

/// The display label for an ALSA section header.
fn alsa_section_label(section: AlsaSection) -> &'static str {
    // Marked at the definition so the extractor registers the English literals;
    // the single `t(...)` happens at the `group_alsa_devices` call site.
    match section {
        AlsaSection::Defaults => qbz_i18n::mark("Defaults"),
        AlsaSection::BitPerfect => qbz_i18n::mark("Bit-perfect (Hardware / Digital)"),
        AlsaSection::PluginHw => qbz_i18n::mark("Plugin Hardware"),
        AlsaSection::Other => qbz_i18n::mark("Other Outputs"),
    }
}

/// Whether a device can deliver bit-perfect playback on `backend` — the
/// rule that drives the "BP" badge. On ALSA this is exactly the
/// Bit-perfect section of the dropdown (Tauri shows the badge on that
/// group only): direct-hardware `hw:` / `front:CARD=` PCMs and the
/// digital `iec958:` outputs. `sysdefault:`, `hdmi:`, `plughw:` and the
/// system default route through converting plugins / mixers and never
/// qualify. PipeWire reports a hardware flag per node; PulseAudio always
/// mixes, so never capable.
pub(super) fn device_is_bit_perfect(backend: AudioBackendType, device: &qbz_audio::AudioDevice) -> bool {
    match backend {
        AudioBackendType::Alsa => {
            let label = device.description.as_deref().unwrap_or(&device.name);
            alsa_section(&device.id, device.is_default, label) == AlsaSection::BitPerfect
        }
        AudioBackendType::PipeWire => device.is_hardware,
        // JACK never bit-perfect (resampled to the graph rate); no per-device concept.
        AudioBackendType::Pulse | AudioBackendType::SystemDefault | AudioBackendType::Jack => false,
    }
}

/// Regroup ALSA device rows into the Tauri dropdown sections and build
/// the parallel `groups` header list. A row's `groups` entry is the
/// section label only when it is the first row of its section; the rest
/// are empty. Rows keep their relative order within a section, so the
/// resulting `ids` stay a faithful index map for `device_index`.
pub(super) fn group_alsa_devices(rows: Vec<DeviceRow>) -> DeviceList {
    // Stable sort by section keeps within-section enumeration order.
    let mut indexed: Vec<(AlsaSection, DeviceRow)> = rows
        .into_iter()
        .map(|r| (alsa_section(&r.id, false, &r.label), r))
        .collect();
    indexed.sort_by_key(|(section, _)| *section);

    let len = indexed.len();
    let mut list = DeviceList {
        labels: Vec::with_capacity(len),
        ids: Vec::with_capacity(len),
        bp: Vec::with_capacity(len),
        groups: Vec::with_capacity(len),
    };
    let mut prev_section: Option<AlsaSection> = None;
    for (section, row) in indexed {
        let header = if prev_section != Some(section) {
            prev_section = Some(section);
            qbz_i18n::t(alsa_section_label(section))
        } else {
            String::new()
        };
        list.labels.push(row.label);
        list.ids.push(row.id);
        list.bp.push(row.bp);
        list.groups.push(header);
    }
    list
}
