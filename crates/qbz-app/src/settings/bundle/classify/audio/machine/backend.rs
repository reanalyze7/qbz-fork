use serde_json::{Map, Value};

use qbz_audio::settings::AudioSettings;
use qbz_audio::AudioBackendType;

use super::backend_name;
use crate::settings::bundle::classify::{adapted_line, applied_line};
use crate::settings::bundle::plan_types::ImportPlan;
use crate::settings::bundle::types::LiveSystem;

pub(super) fn emit(
    map: &Map<String, Value>,
    current: &AudioSettings,
    live: &LiveSystem,
    fallback: bool,
    plan: &mut ImportPlan,
) {
    if !map.contains_key("backend_type") {
        return;
    }

    let bundle_backend: Option<AudioBackendType> = map
        .get("backend_type")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let bundle_backend_val = map.get("backend_type").cloned().unwrap_or(Value::Null);
    let backend_no_change = bundle_backend == current.backend_type;
    let valid = bundle_backend.is_none()
        || bundle_backend == Some(AudioBackendType::SystemDefault)
        || bundle_backend
            .map(|b| live.backends.iter().any(|x| backend_name(b) == *x))
            .unwrap_or(false);

    if backend_no_change {
        applied_line(plan, "audio.backend_type", &bundle_backend_val, "");
    } else if fallback {
        // non-tty / pre-pick fallback forces system default output.
        let sd = Value::String(backend_name(AudioBackendType::SystemDefault).to_string());
        if bundle_backend == Some(AudioBackendType::SystemDefault) {
            applied_line(plan, "audio.backend_type", &bundle_backend_val, "");
        } else {
            adapted_line(
                plan,
                "audio.backend_type",
                &bundle_backend_val,
                &sd,
                "output device unavailable; falling back to system default",
            );
            plan.routing_critical_changed = true;
        }
    } else if valid {
        applied_line(
            plan,
            "audio.backend_type",
            &bundle_backend_val,
            "available on this machine",
        );
        plan.routing_critical_changed = true;
    } else {
        let sd = Value::String(backend_name(AudioBackendType::SystemDefault).to_string());
        adapted_line(
            plan,
            "audio.backend_type",
            &bundle_backend_val,
            &sd,
            "backend not available on this machine",
        );
        plan.routing_critical_changed = true;
    }
}
