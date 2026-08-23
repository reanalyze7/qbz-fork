//! Device/backend enumeration and labeling — pure "what devices/backends
//! are there and how do we label/group them" logic. Blocking I/O only via
//! `BackendManager::create_backend`.

mod alsa;
mod labels;

pub(in crate::settings) use labels::output_labels;

use alsa::group_alsa_devices;
use qbz_audio::backend::{AudioBackendType, BackendManager};

/// Devices enumerated for one backend: parallel label / id / bit-perfect
/// / section-header lists. `bp[i]` flags a device able to deliver
/// bit-perfect output; `groups[i]` is the section-header label shown
/// above row `i` (empty = no header — the row continues the previous
/// section). The four lists stay index-aligned with each other and with
/// `SettingsMaps.devices`, so `device_index` keeps resolving correctly
/// after the rows are regrouped.
pub(in crate::settings) struct DeviceList {
    pub(in crate::settings) labels: Vec<String>,
    pub(in crate::settings) ids: Vec<String>,
    pub(in crate::settings) bp: Vec<bool>,
    pub(in crate::settings) groups: Vec<String>,
}

/// One enumerated output device before grouping.
pub(super) struct DeviceRow {
    pub(super) label: String,
    pub(super) id: String,
    pub(super) bp: bool,
}

pub(in crate::settings) fn backend_label(t: AudioBackendType) -> String {
    match t {
        // Brand/product names stay literal; only the prose entry is translated.
        AudioBackendType::PipeWire => "PipeWire".to_string(),
        AudioBackendType::Alsa => "ALSA".to_string(),
        AudioBackendType::Pulse => "PulseAudio".to_string(),
        AudioBackendType::SystemDefault => qbz_i18n::t("System default"),
        AudioBackendType::Jack => "JACK".to_string(),
    }
}

/// Enumerate output devices for a backend. Always leads with a "System
/// default" entry (empty id). Blocking — call off the UI thread.
///
/// For the ALSA backend the rows are regrouped into the Tauri dropdown
/// sections (Defaults / Bit-perfect / Plugin Hardware / Other Outputs)
/// and a parallel `groups` list carries the section header shown above
/// each section's first row. Other backends keep a flat list with no
/// headers (`groups` all empty).
pub(in crate::settings) fn enumerate_devices(backend: AudioBackendType) -> DeviceList {
    // The synthetic "System default" entry (empty id) always leads.
    let mut rows = vec![DeviceRow {
        label: qbz_i18n::t("System default"),
        id: String::new(),
        bp: false,
    }];
    match BackendManager::create_backend(backend).and_then(|b| b.enumerate_devices()) {
        Ok(devices) => {
            for d in devices {
                let label = match d.description.as_deref() {
                    Some(desc) if !desc.is_empty() => desc.to_string(),
                    _ => d.name.clone(),
                };
                let bp = alsa::device_is_bit_perfect(backend, &d);
                rows.push(DeviceRow {
                    label,
                    id: d.id,
                    bp,
                });
            }
        }
        Err(e) => log::warn!("[qbz-slint] device enumeration failed: {e}"),
    }

    if backend == AudioBackendType::Alsa {
        group_alsa_devices(rows)
    } else {
        // Non-ALSA backends: flat list, no section headers.
        let len = rows.len();
        let mut list = DeviceList {
            labels: Vec::with_capacity(len),
            ids: Vec::with_capacity(len),
            bp: Vec::with_capacity(len),
            groups: vec![String::new(); len],
        };
        for r in rows {
            list.labels.push(r.label);
            list.ids.push(r.id);
            list.bp.push(r.bp);
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_labels_are_distinct() {
        let labels: Vec<_> = [
            AudioBackendType::PipeWire,
            AudioBackendType::Alsa,
            AudioBackendType::Pulse,
            AudioBackendType::SystemDefault,
        ]
        .iter()
        .map(|t| backend_label(*t))
        .collect();
        let mut deduped = labels.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(labels.len(), deduped.len());
    }
}
