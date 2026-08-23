use serde_json::Value;

use super::{applied_line, skip_line, UNKNOWN_WHY, VOLUME_SKIP_WHY};
use crate::settings::bundle::plan_types::ImportPlan;

// ---- prefs — streaming_quality PORTABLE; language/volume out of v1 (§2.3) ----
pub(in crate::settings::bundle) fn plan_prefs(value: &Value, plan: &mut ImportPlan) {
    let Some(map) = value.as_object() else {
        return;
    };
    for (k, v) in map {
        match k.as_str() {
            "streaming_quality" => applied_line(plan, "prefs.streaming_quality", v, ""),
            _ if k.eq_ignore_ascii_case("volume") => {
                plan.skipped.push(skip_line(&format!("prefs.{k}"), VOLUME_SKIP_WHY));
            }
            "language" => plan.skipped.push(skip_line(
                "prefs.language",
                "not imported (no TUI i18n in daemon v1)",
            )),
            _ => plan.skipped.push(skip_line(&format!("prefs.{k}"), UNKNOWN_WHY)),
        }
    }
}
