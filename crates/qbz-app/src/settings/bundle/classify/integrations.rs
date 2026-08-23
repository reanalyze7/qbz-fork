use serde_json::Value;

use super::{applied_line, applied_secret_line, skip_line, UNKNOWN_WHY};
use crate::settings::bundle::plan_types::ImportPlan;
use crate::settings::bundle::types::ImportOptions;

// ---- integrations.scrobblers (§2.5) ----
const SCROBBLER_PORTABLE: &[&str] = &[
    "enabled",
    "lastfm_enabled",
    "lastfm_username",
    "listenbrainz_enabled",
    "listenbrainz_username",
];
const SCROBBLER_SECRET: &[&str] = &["lastfm_session_key", "listenbrainz_token"];

pub(in crate::settings::bundle) fn plan_integrations(
    value: &Value,
    opts: &ImportOptions,
    uid_will_exist: bool,
    plan: &mut ImportPlan,
) {
    let Some(scrob) = value.get("scrobblers").and_then(Value::as_object) else {
        // Unknown integration domain.
        if let Some(map) = value.as_object() {
            for k in map.keys() {
                plan.skipped.push(skip_line(&format!("integrations.{k}"), UNKNOWN_WHY));
            }
        }
        return;
    };

    for (k, v) in scrob {
        let full = format!("integrations.scrobblers.{k}");
        if !uid_will_exist {
            plan.skipped.push(skip_line(
                &full,
                "no user on this daemon yet — run qbzd login first, or import with --include-auth",
            ));
            continue;
        }
        if SCROBBLER_PORTABLE.contains(&k.as_str()) {
            applied_line(plan, &full, v, "");
        } else if SCROBBLER_SECRET.contains(&k.as_str()) {
            if opts.include_auth {
                applied_secret_line(plan, &full, v);
            } else {
                plan.skipped
                    .push(skip_line(&full, "secrets require --include-auth"));
            }
        } else if k == "ui_collapsed" {
            plan.skipped
                .push(skip_line(&full, "not imported (desktop UI state)"));
        } else {
            plan.skipped.push(skip_line(&full, UNKNOWN_WHY));
        }
    }
}
