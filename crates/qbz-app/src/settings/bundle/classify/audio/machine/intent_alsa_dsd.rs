use serde_json::{Map, Value};

use qbz_audio::settings::AudioSettings;
use qbz_audio::AudioBackendType;

use crate::settings::bundle::classify::audio::AUDIO_INTENT_FLAGS;
use crate::settings::bundle::classify::{adapted_line, applied_line, skip_line};
use crate::settings::bundle::plan_types::{DeviceChoice, ImportPlan};
use crate::settings::bundle::types::ImportOptions;

pub(super) fn emit_intent_flags(
    map: &Map<String, Value>,
    current: &AudioSettings,
    device_survives: bool,
    plan: &mut ImportPlan,
) {
    for flag in AUDIO_INTENT_FLAGS {
        let Some(v) = map.get(*flag) else { continue };
        let bundle_bool = v.as_bool().unwrap_or(false);
        let current_bool = intent_flag_current(current, flag);
        if bundle_bool == current_bool {
            applied_line(plan, &format!("audio.{flag}"), v, "");
        } else if device_survives {
            applied_line(plan, &format!("audio.{flag}"), v, "rides the device");
            plan.routing_critical_changed = true;
        } else {
            adapted_line(
                plan,
                &format!("audio.{flag}"),
                v,
                &Value::Bool(false),
                "reset (no validated device)",
            );
        }
    }
}

pub(super) fn emit_alsa_fields(
    map: &Map<String, Value>,
    current: &AudioSettings,
    device_survives: bool,
    fallback: bool,
    forced_device: &Option<DeviceChoice>,
    plan: &mut ImportPlan,
) {
    let resolved_backend_alsa = resolved_backend_is_alsa(map, current, fallback, forced_device);
    for key in ["alsa_plugin", "alsa_hardware_volume"] {
        let Some(v) = map.get(key) else { continue };
        let no_change = alsa_field_no_change(current, key, v);
        if no_change {
            applied_line(plan, &format!("audio.{key}"), v, "");
        } else if device_survives && resolved_backend_alsa {
            applied_line(plan, &format!("audio.{key}"), v, "rides the ALSA device");
        } else {
            plan.skipped.push(skip_line(
                &format!("audio.{key}"),
                "applies only with a validated ALSA device",
            ));
        }
    }
}

pub(super) fn emit_dsd_mode(
    map: &Map<String, Value>,
    current: &AudioSettings,
    opts: &ImportOptions,
    fallback: bool,
    plan: &mut ImportPlan,
) {
    // dsd_mode — no-change short-circuit, else downgrade unless --trust-dsd (§5.3 step 4).
    if let Some(v) = map.get("dsd_mode") {
        let bundle_dsd = v.as_str().unwrap_or("convert");
        if bundle_dsd == current.dsd_mode {
            applied_line(plan, "audio.dsd_mode", v, "");
        } else if matches!(bundle_dsd, "dop" | "native") && (!opts.trust_dsd || fallback) {
            adapted_line(
                plan,
                "audio.dsd_mode",
                v,
                &Value::String("convert".into()),
                "pass --trust-dsd to keep DoP",
            );
        } else {
            applied_line(plan, "audio.dsd_mode", v, "");
        }
    }
}

fn intent_flag_current(current: &AudioSettings, flag: &str) -> bool {
    match flag {
        "exclusive_mode" => current.exclusive_mode,
        "dac_passthrough" => current.dac_passthrough,
        "pw_force_bitperfect" => current.pw_force_bitperfect,
        "skip_sink_switch" => current.skip_sink_switch,
        "reserve_dac_while_running" => current.reserve_dac_while_running,
        _ => false,
    }
}

fn alsa_field_no_change(current: &AudioSettings, key: &str, v: &Value) -> bool {
    match key {
        "alsa_hardware_volume" => v.as_bool() == Some(current.alsa_hardware_volume),
        "alsa_plugin" => {
            let cur = serde_json::to_value(current.alsa_plugin).unwrap_or(Value::Null);
            *v == cur
        }
        _ => false,
    }
}

fn resolved_backend_is_alsa(
    map: &Map<String, Value>,
    current: &AudioSettings,
    fallback: bool,
    forced_device: &Option<DeviceChoice>,
) -> bool {
    if fallback || matches!(forced_device, Some(DeviceChoice::SystemDefault)) {
        return false;
    }
    let bundle_backend: Option<AudioBackendType> = map
        .get("backend_type")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    match bundle_backend {
        Some(b) => b == AudioBackendType::Alsa,
        None => {
            // backend not in the bundle → the target keeps its current backend.
            current.backend_type == Some(AudioBackendType::Alsa)
        }
    }
}
