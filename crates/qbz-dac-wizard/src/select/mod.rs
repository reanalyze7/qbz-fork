//! Slice 7: select-dacs (auto-detect + manual escape hatch).

mod pure;

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use qbz_ui::{AppWindow, DacCandidateRow, DacWizardState};

use pure::{detect_dac_type, format_rates, validate_node_name};

/// Plain, `Send` candidate produced on the worker thread.
pub struct DacCandidateData {
    id: String,
    description: String,
    bus: String,
    is_default: bool,
    looks_like_dac: bool,
    rates_label: String,
}

/// Immediate UI feedback before the (blocking) enumeration runs.
pub fn begin_detect(window: &AppWindow) {
    window.global::<DacWizardState>().set_detecting(true);
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

/// Apply enumerated candidates to the state. Pre-selects the likely DACs; if
/// nothing enumerated, flips `has-enumeration` off so the manual escape hatch
/// shows.
pub fn apply_candidates(window: &AppWindow, data: Vec<DacCandidateData>) {
    let st = window.global::<DacWizardState>();
    let rows: Vec<DacCandidateRow> = data
        .iter()
        .map(|d| DacCandidateRow {
            id: d.id.clone().into(),
            description: d.description.clone().into(),
            bus: d.bus.clone().into(),
            is_default: d.is_default,
            looks_like_dac: d.looks_like_dac,
            checked: d.looks_like_dac,
            rates_label: d.rates_label.clone().into(),
        })
        .collect();
    let any = rows.iter().any(|r| r.checked);
    st.set_has_enumeration(!data.is_empty());
    st.set_any_dac_selected(any);
    st.set_candidates(ModelRc::new(VecModel::from(rows)));
    st.set_detecting(false);
}

/// Flip one candidate's checkbox + recompute the Next gate.
pub fn toggle_dac(window: &AppWindow, index: i32) {
    let st = window.global::<DacWizardState>();
    let model = st.get_candidates();
    if let Some(vm) = model
        .as_any()
        .downcast_ref::<VecModel<DacCandidateRow>>()
    {
        if let Some(mut row) = vm.row_data(index.max(0) as usize) {
            row.checked = !row.checked;
            vm.set_row_data(index.max(0) as usize, row);
        }
    }
    let any = (0..model.row_count()).any(|i| model.row_data(i).map(|r| r.checked).unwrap_or(false));
    st.set_any_dac_selected(any);
}

/// Validate a manually-pasted node.name (escape hatch). 1:1 with the Tauri
/// `validateNodeName` / `detectDacType`.
pub fn validate_manual(window: &AppWindow, text: &str) {
    let st = window.global::<DacWizardState>();
    st.set_manual_node_name(text.into());
    st.set_manual_valid(validate_node_name(text));
    st.set_manual_dac_type(detect_dac_type(text).into());
}
