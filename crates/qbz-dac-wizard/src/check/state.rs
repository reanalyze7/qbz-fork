//! Slice 6 (check step) — UI-state-mutating half: runs the frontend-agnostic
//! audio-stack probes (`qbz_audio::health`) on open, and re-renders remediations
//! when the user overrides the distro/init. Read-only — nothing here writes a
//! system file or opens a stream.

use std::sync::Mutex;

use qbz_audio::{AudioStackHealth, Distro, InitSystem};
use slint::{ComponentHandle, ModelRc, VecModel};

use qbz_ui::{AppWindow, DacWizardState, RemediationRow};

use super::install::reference_commands;
use super::remediation::remediations;

// Last probe result, so a distro override recomputes commands without
// re-shelling on every dropdown change.
static LAST_HEALTH: Mutex<Option<AudioStackHealth>> = Mutex::new(None);

/// Synchronous part of opening the wizard: reset, fill the distro dropdown
/// (auto-detected, always overridable), and show a "checking…" state until the
/// async probe lands.
pub fn open_immediate(window: &AppWindow) {
    let st = window.global::<DacWizardState>();
    st.set_open(true);
    st.set_step(0);
    st.set_welcome_confirmed(false);

    let distro_opts: Vec<slint::SharedString> =
        Distro::ALL.iter().map(|d| d.label().into()).collect();
    st.set_distro_options(ModelRc::new(VecModel::from(distro_opts)));
    st.set_distro_index(qbz_audio::detect_distro().index() as i32);

    let init_opts: Vec<slint::SharedString> =
        InitSystem::ALL.iter().map(|i| i.label().into()).collect();
    st.set_init_options(ModelRc::new(VecModel::from(init_opts)));
    st.set_init_index(qbz_audio::detect_init().index() as i32);

    let sandbox = qbz_audio::detect_sandbox();
    st.set_sandboxed(sandbox != qbz_audio::Sandbox::None);
    st.set_sandbox_name(
        match sandbox {
            qbz_audio::Sandbox::Flatpak => "Flatpak",
            qbz_audio::Sandbox::Snap => "Snap",
            qbz_audio::Sandbox::None => "",
        }
        .into(),
    );

    st.set_health_ok(false);
    st.set_health_summary(qbz_i18n::t("Checking your audio stack…").into());
    st.set_remediations(ModelRc::new(VecModel::from(Vec::<RemediationRow>::new())));
}

/// Apply a completed health probe: cache it and re-render from the current
/// distro/init selections.
pub fn apply_health(window: &AppWindow, health: AudioStackHealth) {
    *LAST_HEALTH.lock().unwrap() = Some(health);
    recompute(window);
}

/// User overrode the distro (package manager) — recompute.
pub fn set_distro(window: &AppWindow, index: i32) {
    window.global::<DacWizardState>().set_distro_index(index);
    recompute(window);
}

/// User overrode the init system (service commands) — recompute.
pub fn set_init(window: &AppWindow, index: i32) {
    window.global::<DacWizardState>().set_init_index(index);
    recompute(window);
}

/// Rebuild the remediations from the cached probe + the current distro/init
/// dropdown selections (either of which the user can override).
pub(crate) fn recompute(window: &AppWindow) {
    let st = window.global::<DacWizardState>();
    let health = LAST_HEALTH
        .lock()
        .unwrap()
        .unwrap_or_else(qbz_audio::audio_stack_health);
    let distro = Distro::ALL
        .get(st.get_distro_index().max(0) as usize)
        .copied()
        .unwrap_or(Distro::Other);
    let init = InitSystem::ALL
        .get(st.get_init_index().max(0) as usize)
        .copied()
        .unwrap_or(InitSystem::Unknown);

    // In a sandbox the host probes are blind, so don't render a health verdict —
    // show reference setup commands for the chosen distro/init (Tauri-style,
    // which never probed either). The UI shows a sandbox info banner instead.
    let rows = if st.get_sandboxed() {
        st.set_health_ok(false);
        st.set_health_summary("".into());
        reference_commands(distro, init)
    } else {
        let r = remediations(health, distro, init);
        st.set_health_ok(health.is_ready());
        st.set_health_summary(if health.is_ready() {
            qbz_i18n::t("Your audio stack is ready for bit-perfect playback.").into()
        } else {
            let n = r.len();
            qbz_i18n::tf(
                "{} item needs attention before bit-perfect playback will work.",
                "{} items need attention before bit-perfect playback will work.",
                n as i64,
                &[&n.to_string()],
            )
            .into()
        });
        r
    };
    let model: Vec<RemediationRow> = rows
        .into_iter()
        .map(|(caption, command)| RemediationRow {
            caption: caption.into(),
            command: command.into(),
        })
        .collect();
    st.set_remediations(ModelRc::new(VecModel::from(model)));
}
