//! Slice 10 (review-and-apply) — UI-state half: per-DAC config generation
//! orchestration + accordion/backup/restart state.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use qbz_audio::InitSystem;
use qbz_ui::{AppWindow, DacConfigRow, DacWizardState};

use crate::check::remediation::restart_cmd;

use super::conf::{pipewire_conf, pulse_conf, short_name, wireplumber_conf};

/// Plain, `Send` per-DAC generated config (built on a worker thread).
pub struct DacConfigData {
    name: String,
    node_name: String,
    pipewire_conf: String,
    pulse_conf: String,
    wireplumber_conf: String,
}

/// (node_name, display_name) for every checked candidate, or a valid manual one.
pub fn checked_dacs(window: &AppWindow) -> Vec<(String, String)> {
    let st = window.global::<DacWizardState>();
    let model = st.get_candidates();
    let mut out = Vec::new();
    for i in 0..model.row_count() {
        if let Some(r) = model.row_data(i) {
            if r.checked {
                out.push((r.id.to_string(), r.description.to_string()));
            }
        }
    }
    if out.is_empty() {
        let manual = st.get_manual_node_name().to_string();
        if !manual.trim().is_empty() && st.get_manual_valid() {
            out.push((manual.clone(), manual));
        }
    }
    out
}

/// Re-probe rates + build the three config snippets per DAC (blocking).
pub fn gen_configs_blocking(dacs: Vec<(String, String)>) -> Vec<DacConfigData> {
    dacs.into_iter()
        .map(|(node_name, name)| {
            let rates = qbz_audio::query_dac_capabilities(&node_name).sample_rates;
            let short = short_name(&name, &node_name);
            DacConfigData {
                pipewire_conf: pipewire_conf(&short, &rates),
                pulse_conf: pulse_conf(&short),
                wireplumber_conf: wireplumber_conf(&short, &node_name, &rates, &name),
                name,
                node_name,
            }
        })
        .collect()
}

/// Push generated configs + backup/restart/paths to the state.
pub fn apply_configs(window: &AppWindow, data: Vec<DacConfigData>) {
    let st = window.global::<DacWizardState>();
    let single = data.len() == 1;
    let rows: Vec<DacConfigRow> = data
        .iter()
        .map(|d| DacConfigRow {
            name: d.name.clone().into(),
            node_name: d.node_name.clone().into(),
            pipewire_conf: d.pipewire_conf.clone().into(),
            pulse_conf: d.pulse_conf.clone().into(),
            wireplumber_conf: d.wireplumber_conf.clone().into(),
            expanded: single, // one DAC → expanded; multiple → collapsed accordions
        })
        .collect();
    let mut paths: Vec<slint::SharedString> = Vec::new();
    for d in &data {
        let short = short_name(&d.name, &d.node_name);
        paths.push(format!("~/.config/pipewire/pipewire.conf.d/99-qbz-dac-{short}.conf").into());
        paths.push(format!("~/.config/pipewire/client.conf.d/99-qbz-bitperfect-{short}.conf").into());
        paths
            .push(format!("~/.config/wireplumber/wireplumber.conf.d/99-qbz-dac-{short}.conf").into());
    }
    st.set_dac_configs(ModelRc::new(VecModel::from(rows)));
    st.set_created_paths(ModelRc::new(VecModel::from(paths)));
    st.set_backup_cmd(BACKUP_CMD.into());
    let init = InitSystem::ALL
        .get(st.get_init_index().max(0) as usize)
        .copied()
        .unwrap_or(InitSystem::Unknown);
    st.set_restart_cmd(restart_cmd(init).into());
}

/// Collapse/expand one DAC's generated-config accordion.
pub fn toggle_config(window: &AppWindow, index: i32) {
    let model = window.global::<DacWizardState>().get_dac_configs();
    if let Some(vm) = model.as_any().downcast_ref::<VecModel<DacConfigRow>>() {
        if let Some(mut row) = vm.row_data(index.max(0) as usize) {
            row.expanded = !row.expanded;
            vm.set_row_data(index.max(0) as usize, row);
        }
    }
}

const BACKUP_CMD: &str = "BACKUP=~/.config/qbz/backups/pipewire-$(date +%Y%m%d-%H%M%S)\nmkdir -p \"$BACKUP\"\ncp -a ~/.config/pipewire \"$BACKUP/\" 2>/dev/null || true\ncp -a ~/.config/wireplumber \"$BACKUP/\" 2>/dev/null || true\necho \"Backup created at: $BACKUP\"";
