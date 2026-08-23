use serde_json::Value;

use super::apply_writes::{
    apply_audio_writes, apply_playback_writes, apply_prefs_quality, apply_scrobbler_writes,
    persist_auth,
};
use super::error::BundleError;
use super::readers::read_last_user_id;
use super::plan_types::{ImportPlan, ImportReport};
use super::types::ProfilePaths;

/// Execute a plan against the daemon-root stores. Reached only after every
/// check passed (validate-all-then-apply). Pure setter writes — re-running is
/// safe and idempotent (§5.3 step 6). `validated_uid` is the authoritative uid
/// from the validated login (§5.7); when absent, the daemon's own
/// `last_user_id` is consulted for per-user writes.
pub fn apply(
    plan: &ImportPlan,
    target: &ProfilePaths,
    validated_uid: Option<u64>,
) -> Result<ImportReport, BundleError> {
    let mut report = ImportReport {
        applied: plan.applied.len(),
        adapted: plan.adapted.len(),
        skipped: plan.skipped.len(),
        per_domain: Vec::new(),
    };

    let uid = validated_uid.or_else(|| read_last_user_id(&target.data_root));

    // auth first: persist token + last_user_id + ensure users/<uid>/ (§5.7).
    if let (Some(token), Some(uid)) = (&plan.auth_token, validated_uid) {
        match persist_auth(target, token, uid) {
            Ok(()) => report.per_domain.push(("auth".into(), Ok(()))),
            Err(e) => {
                report.per_domain.push(("auth".into(), Err(e.clone())));
                return Err(BundleError::Io(e));
            }
        }
    }

    // group writes by store so each store opens once.
    let mut audio_writes: Vec<(&str, &Value)> = Vec::new();
    let mut playback_writes: Vec<(&str, &Value)> = Vec::new();
    let mut prefs_quality: Option<&Value> = None;
    let mut scrobbler_writes: Vec<(&str, &Value)> = Vec::new();

    for (key, value) in &plan.writes {
        if let Some(rest) = key.strip_prefix("audio.") {
            audio_writes.push((rest, value));
        } else if let Some(rest) = key.strip_prefix("playback.") {
            playback_writes.push((rest, value));
        } else if key == "prefs.streaming_quality" {
            prefs_quality = Some(value);
        } else if let Some(rest) = key.strip_prefix("integrations.scrobblers.") {
            scrobbler_writes.push((rest, value));
        }
    }

    if !audio_writes.is_empty() {
        let r = apply_audio_writes(&target.data_root, &audio_writes);
        let failed = r.is_err();
        report.per_domain.push(("audio".into(), r.clone()));
        if failed {
            return Err(BundleError::Io(r.unwrap_err()));
        }
    }
    if !playback_writes.is_empty() {
        let r = apply_playback_writes(&target.data_root, &playback_writes);
        let failed = r.is_err();
        report.per_domain.push(("playback".into(), r.clone()));
        if failed {
            return Err(BundleError::Io(r.unwrap_err()));
        }
    }
    if let Some(q) = prefs_quality {
        let r = apply_prefs_quality(&target.data_root, q);
        let failed = r.is_err();
        report.per_domain.push(("prefs".into(), r.clone()));
        if failed {
            return Err(BundleError::Io(r.unwrap_err()));
        }
    }
    if !scrobbler_writes.is_empty() {
        match uid {
            Some(uid) => {
                let r = apply_scrobbler_writes(&target.data_root, uid, &scrobbler_writes);
                let failed = r.is_err();
                report.per_domain.push(("integrations".into(), r.clone()));
                if failed {
                    return Err(BundleError::Io(r.unwrap_err()));
                }
            }
            None => report.per_domain.push((
                "integrations".into(),
                Err("no user on this daemon — skipped".into()),
            )),
        }
    }

    Ok(report)
}
