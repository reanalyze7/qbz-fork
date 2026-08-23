use serde_json::Value;

use super::skip_line;
use crate::settings::bundle::plan_types::ImportPlan;
use crate::settings::bundle::types::ImportOptions;

// ---- auth — SECRET, double gate (§2.7, §3) ----
pub(in crate::settings::bundle) fn plan_auth(value: &Value, opts: &ImportOptions, plan: &mut ImportPlan) {
    let Some(map) = value.as_object() else {
        return;
    };
    plan.bundle_user_id = map.get("user_id").and_then(Value::as_u64);

    let token = map.get("user_auth_token").and_then(Value::as_str);
    match token {
        Some(t) if !t.is_empty() => {
            if opts.include_auth {
                plan.auth_token = Some(t.to_string());
                // The applied line is added by the CLI after validation
                // (the token is validated BEFORE any write, §5.3 step 5).
            } else {
                plan.skipped.push(skip_line(
                    "auth.user_auth_token",
                    "secrets require --include-auth",
                ));
            }
        }
        _ => {}
    }
}
