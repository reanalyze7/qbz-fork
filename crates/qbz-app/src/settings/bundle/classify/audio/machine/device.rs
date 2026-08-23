use serde_json::{Map, Value};

use qbz_audio::settings::AudioSettings;

use super::pick_backend_name;
use crate::settings::bundle::classify::{adapted_line, applied_line};
use crate::settings::bundle::plan_types::{DeviceChoice, DevicePick, ImportPlan};
use crate::settings::bundle::types::{ImportOptions, LiveSystem};

/// The resolved device decision, threaded into the backend/intent-flags/ALSA
/// steps that ride on it.
pub(super) struct DeviceOutcome {
    pub(super) fallback: bool,
    pub(super) device_survives: bool,
    pub(super) resolved_device: Option<String>,
    pub(super) bundle_device: Option<String>,
    pub(super) device_present: bool,
    pub(super) device_no_change: bool,
    pub(super) device_is_null: bool,
    pub(super) repick_label: Option<String>,
}

pub(super) fn decide(
    map: &Map<String, Value>,
    current: &AudioSettings,
    live: &LiveSystem,
    opts: &ImportOptions,
    forced_device: &Option<DeviceChoice>,
    plan: &mut ImportPlan,
) -> DeviceOutcome {
    let device_present = map.contains_key("output_device");

    let bundle_device: Option<String> = map
        .get("output_device")
        .and_then(|v| v.as_str().filter(|s| !s.is_empty()).map(str::to_string));
    let device_found = bundle_device
        .as_ref()
        .map(|id| live.devices.iter().any(|(d, _)| d == id))
        .unwrap_or(false);
    let device_no_change = device_present && bundle_device == current.output_device;
    let device_is_null = device_present && bundle_device.is_none();

    let mut fallback = false;
    let device_survives;
    let resolved_device: Option<String>; // None = system default
    let mut repick_label: Option<String> = None;

    match forced_device {
        Some(DeviceChoice::Device { id, label }) => {
            resolved_device = Some(id.clone());
            device_survives = true;
            repick_label = Some(label.clone());
        }
        Some(DeviceChoice::SystemDefault) => {
            resolved_device = None;
            device_survives = false;
        }
        None => {
            if device_no_change || device_is_null || !device_present {
                resolved_device = bundle_device.clone();
                device_survives = true;
            } else if device_found {
                resolved_device = bundle_device.clone();
                device_survives = true;
            } else {
                // present, changed, not found → needs a pick.
                fallback = true;
                resolved_device = None;
                device_survives = false;
                if !opts.non_tty {
                    plan.device_pick = Some(DevicePick {
                        wanted: bundle_device.clone().unwrap_or_default(),
                        backend: pick_backend_name(map, current),
                        options: live.devices.clone(),
                    });
                }
            }
        }
    }

    DeviceOutcome {
        fallback,
        device_survives,
        resolved_device,
        bundle_device,
        device_present,
        device_no_change,
        device_is_null,
        repick_label,
    }
}

pub(super) fn emit_line(map: &Map<String, Value>, outcome: &DeviceOutcome, plan: &mut ImportPlan) {
    let _ = map;
    if !outcome.device_present {
        return;
    }
    let new_val = match &outcome.resolved_device {
        Some(id) => Value::String(id.clone()),
        None => Value::Null,
    };
    if outcome.resolved_device == outcome.bundle_device {
        let why = if outcome.device_no_change {
            ""
        } else if outcome.device_is_null {
            ""
        } else {
            "found on this machine"
        };
        applied_line(plan, "audio.output_device", &new_val, why);
    } else {
        let old_val = match &outcome.bundle_device {
            Some(id) => Value::String(id.clone()),
            None => Value::Null,
        };
        let why = match &outcome.repick_label {
            Some(label) => format!("re-picked: {label}"),
            None => "device not found on this machine".to_string(),
        };
        adapted_line(plan, "audio.output_device", &old_val, &new_val, &why);
        plan.routing_critical_changed = true;
    }
}
