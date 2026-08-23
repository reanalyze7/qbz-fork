mod machine;

use serde_json::Value;

use super::{adapted_line, applied_line, skip_line, CACHE_SKIP_WHY, UNKNOWN_WHY, VOLUME_SKIP_WHY};
use crate::settings::bundle::readers::read_audio_settings;
use crate::settings::bundle::plan_types::{DeviceChoice, ImportPlan};
use crate::settings::bundle::types::{ImportOptions, LiveSystem, ProfilePaths};

use machine::plan_audio_machine;

// ---- audio — the interdependent machine block (§2.2, §3, §5.3 step 4) ----
const AUDIO_PORTABLE: &[&str] = &[
    "stream_first_track",
    "stream_buffer_seconds",
    "streaming_only",
    "normalization_enabled",
    "normalization_target_lufs",
    "gapless_enabled",
    "crossfade_seconds",
    "allow_quality_fallback",
    "sync_audio_on_startup",
    "limit_quality_to_device",
    "preferred_sample_rate",
];
pub(super) const AUDIO_INTENT_FLAGS: &[&str] = &[
    "exclusive_mode",
    "dac_passthrough",
    "pw_force_bitperfect",
    "skip_sink_switch",
    "reserve_dac_while_running",
];
const AUDIO_NEVER_CACHES: &[&str] = &["device_max_sample_rate", "device_sample_rate_limits"];

pub(in crate::settings::bundle) fn plan_audio(
    value: &Value,
    target: &ProfilePaths,
    opts: &ImportOptions,
    live: &LiveSystem,
    forced_device: &Option<DeviceChoice>,
    plan: &mut ImportPlan,
) {
    let Some(map) = value.as_object() else {
        return;
    };
    let current = read_audio_settings(&target.data_root).unwrap_or_default();

    // First pass: the simple classifications.
    for (k, v) in map {
        if k.eq_ignore_ascii_case("volume") {
            plan.skipped.push(skip_line(&format!("audio.{k}"), VOLUME_SKIP_WHY));
        } else if AUDIO_NEVER_CACHES.contains(&k.as_str()) {
            plan.skipped.push(skip_line(&format!("audio.{k}"), CACHE_SKIP_WHY));
        } else if AUDIO_PORTABLE.contains(&k.as_str()) {
            applied_line(plan, &format!("audio.{k}"), v, "");
        } else if k == "quality_fallback_behavior" {
            plan_quality_fallback(v, plan);
        }
    }

    // Second pass: the machine block (backend + device + intent + alsa + dsd).
    plan_audio_machine(map, &current, opts, live, forced_device, plan);

    // Any genuinely-unknown audio keys.
    for (k, _) in map {
        let known = AUDIO_PORTABLE.contains(&k.as_str())
            || AUDIO_INTENT_FLAGS.contains(&k.as_str())
            || AUDIO_NEVER_CACHES.contains(&k.as_str())
            || matches!(
                k.as_str(),
                "quality_fallback_behavior"
                    | "backend_type"
                    | "output_device"
                    | "alsa_plugin"
                    | "alsa_hardware_volume"
                    | "dsd_mode"
            )
            || k.eq_ignore_ascii_case("volume");
        if !known {
            plan.skipped.push(skip_line(&format!("audio.{k}"), UNKNOWN_WHY));
        }
    }
}

fn plan_quality_fallback(v: &Value, plan: &mut ImportPlan) {
    if v.as_str() == Some("ask") {
        // §5.5: never a silent skip on a daemon.
        adapted_line(
            plan,
            "audio.quality_fallback_behavior",
            v,
            &Value::String("always_fallback".into()),
            "no one to ask on a daemon",
        );
    } else {
        applied_line(plan, "audio.quality_fallback_behavior", v, "");
    }
}
