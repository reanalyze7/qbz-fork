use serde_json::Value;

use super::{applied_line, skip_line, UNKNOWN_WHY, VOLUME_SKIP_WHY};
use crate::settings::bundle::plan_types::ImportPlan;

// ---- playback — all PORTABLE, applied verbatim (§2.1) ----
const PLAYBACK_KEYS: &[&str] = &[
    "autoplay_mode",
    "show_context_icon",
    "persist_session",
    "resume_playback_position",
];

pub(in crate::settings::bundle) fn plan_playback(value: &Value, plan: &mut ImportPlan) {
    let Some(map) = value.as_object() else {
        return;
    };
    for (k, v) in map {
        if k.eq_ignore_ascii_case("volume") {
            plan.skipped.push(skip_line(&format!("playback.{k}"), VOLUME_SKIP_WHY));
        } else if PLAYBACK_KEYS.contains(&k.as_str()) {
            applied_line(plan, &format!("playback.{k}"), v, "");
        } else {
            plan.skipped.push(skip_line(&format!("playback.{k}"), UNKNOWN_WHY));
        }
    }
}
