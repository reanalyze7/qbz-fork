// ============================ classification helpers ============================
//
// Shared by every `plan_*` domain function. The load-bearing invariant (04 §1)
// is CLASSIFICATION LIVES IN THE IMPORTER, NEVER IN THE BUNDLE — these
// `plan_*` functions (and their helpers below) ARE that classification layer.

pub(super) mod audio;
pub(super) mod auth;
pub(super) mod integrations;
pub(super) mod library_folders;
pub(super) mod playback;
pub(super) mod prefs;

pub(super) use audio::plan_audio;
pub(super) use auth::plan_auth;
pub(super) use integrations::plan_integrations;
pub(super) use library_folders::plan_library_folders;
pub(super) use playback::plan_playback;
pub(super) use prefs::plan_prefs;

use serde_json::Value;

use super::plan_types::{ImportPlan, PlanLine};

pub(super) const UNKNOWN_WHY: &str = "unknown field (bundle from a newer QBZ?)";
pub(super) const VOLUME_SKIP_WHY: &str =
    "never imported (volume hazard — a daemon may drive a power amp)";
pub(super) const CACHE_SKIP_WHY: &str = "never imported (source-machine device cache)";

pub(super) fn skip_line(key: &str, why: &str) -> PlanLine {
    PlanLine {
        key: key.to_string(),
        old: None,
        new: String::new(),
        why: why.to_string(),
    }
}

pub(super) fn applied_line(plan: &mut ImportPlan, key: &str, value: &Value, why: &str) {
    plan.applied.push(PlanLine {
        key: key.to_string(),
        old: None,
        new: render_value(value),
        why: why.to_string(),
    });
    plan.writes.push((key.to_string(), value.clone()));
}

/// SECRET-class applied line (§5.4): the VALUE COLUMN IS MASKED — the real
/// value goes only into the write list, never into the rendered summary
/// (terminal scrollback and CI logs are not a place for bearer tokens).
/// Non-empty → `(secret, applied)`; empty → `(empty)`, matching the §5.4 example.
pub(super) fn applied_secret_line(plan: &mut ImportPlan, key: &str, value: &Value) {
    let masked = match value.as_str() {
        Some(s) if !s.is_empty() => "(secret, applied)",
        _ => "(empty)",
    };
    plan.applied.push(PlanLine {
        key: key.to_string(),
        old: None,
        new: masked.to_string(),
        why: String::new(),
    });
    plan.writes.push((key.to_string(), value.clone()));
}

pub(super) fn adapted_line(plan: &mut ImportPlan, key: &str, old: &Value, new: &Value, why: &str) {
    plan.adapted.push(PlanLine {
        key: key.to_string(),
        old: Some(render_value(old)),
        new: render_value(new),
        why: why.to_string(),
    });
    plan.writes.push((key.to_string(), new.clone()));
}

/// Render a JSON value for the human summary (§5.4): null → `(auto)`, empty
/// string → `(empty)`, bools/numbers/strings verbatim.
pub(super) fn render_value(v: &Value) -> String {
    match v {
        Value::Null => "(auto)".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) if s.is_empty() => "(empty)".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
