mod backend;
mod device;
mod intent_alsa_dsd;

use serde_json::{Map, Value};

use qbz_audio::settings::AudioSettings;
use qbz_audio::AudioBackendType;

use crate::settings::bundle::plan_types::{DeviceChoice, ImportPlan};
use crate::settings::bundle::types::{ImportOptions, LiveSystem};

pub(in crate::settings::bundle) fn plan_audio_machine(
    map: &Map<String, Value>,
    current: &AudioSettings,
    opts: &ImportOptions,
    live: &LiveSystem,
    forced_device: &Option<DeviceChoice>,
    plan: &mut ImportPlan,
) {
    // Decide + emit the device outcome first: everything downstream (intent
    // flags, ALSA fields) rides on whether the device "survives".
    let outcome = device::decide(map, current, live, opts, forced_device, plan);
    device::emit_line(map, &outcome, plan);

    backend::emit(map, current, live, outcome.fallback, plan);

    intent_alsa_dsd::emit_intent_flags(map, current, outcome.device_survives, plan);
    intent_alsa_dsd::emit_alsa_fields(
        map,
        current,
        outcome.device_survives,
        outcome.fallback,
        forced_device,
        plan,
    );
    intent_alsa_dsd::emit_dsd_mode(map, current, opts, outcome.fallback, plan);
}

/// The backend the device options belong to: the bundle's backend when present,
/// else the target's current, else system default (for the §5.4 prompt line).
pub(super) fn pick_backend_name(map: &Map<String, Value>, current: &AudioSettings) -> String {
    let bundle_backend: Option<AudioBackendType> = map
        .get("backend_type")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    backend_name(
        bundle_backend
            .or(current.backend_type)
            .unwrap_or(AudioBackendType::SystemDefault),
    )
    .to_string()
}

pub(super) fn backend_name(b: AudioBackendType) -> &'static str {
    match b {
        AudioBackendType::PipeWire => "PipeWire",
        AudioBackendType::Alsa => "Alsa",
        AudioBackendType::Pulse => "Pulse",
        AudioBackendType::Jack => "Jack",
        AudioBackendType::SystemDefault => "SystemDefault",
    }
}
