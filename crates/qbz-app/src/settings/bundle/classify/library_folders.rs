use serde_json::Value;

use crate::settings::bundle::plan_types::{ImportPlan, PlanLine};

// ---- library_folders — P0 daemon always skips (§2.6) ----
pub(in crate::settings::bundle) fn plan_library_folders(value: &Value, plan: &mut ImportPlan) {
    let count = value.as_array().map(|a| a.len()).unwrap_or(0);
    plan.skipped.push(PlanLine {
        key: format!("library_folders ({count} folder{})", if count == 1 { "" } else { "s" }),
        old: None,
        new: String::new(),
        why: "no local library on qbzd v1".to_string(),
    });
}
