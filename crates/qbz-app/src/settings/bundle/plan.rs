use serde_json::Value;

use super::classify::{
    plan_audio, plan_auth, plan_integrations, plan_library_folders, plan_playback, plan_prefs,
    skip_line, UNKNOWN_WHY, VOLUME_SKIP_WHY,
};
use super::error::BundleError;
use super::readers::read_last_user_id;
use super::plan_types::{DeviceChoice, ImportPlan};
use super::types::{Bundle, ImportOptions, LiveSystem, ProfilePaths, SCHEMA_VERSION};

/// Classify every present field against the local system, producing the three
/// display buckets + the typed write list. Steps 2–4 of §5.3 (read+parse is the
/// caller's `Bundle::parse`, step 1). Non-interactive: when a device needs a
/// pick, TTY callers get `device_pick` set and should call
/// [`replan_with_device`] after prompting.
pub fn plan(
    bundle: &Bundle,
    target: &ProfilePaths,
    opts: &ImportOptions,
    live: &LiveSystem,
) -> Result<ImportPlan, BundleError> {
    build_plan(bundle, target, opts, live, None)
}

/// Re-run [`plan`] with the operator's device choice resolved (TTY re-pick,
/// §5.4). The returned plan has `device_pick == None`.
pub fn replan_with_device(
    bundle: &Bundle,
    target: &ProfilePaths,
    opts: &ImportOptions,
    live: &LiveSystem,
    chosen: DeviceChoice,
) -> Result<ImportPlan, BundleError> {
    build_plan(bundle, target, opts, live, Some(chosen))
}

fn build_plan(
    bundle: &Bundle,
    target: &ProfilePaths,
    opts: &ImportOptions,
    live: &LiveSystem,
    forced_device: Option<DeviceChoice>,
) -> Result<ImportPlan, BundleError> {
    // Step 2: version gate.
    if bundle.schema_version < 1 {
        return Err(BundleError::VersionMalformed);
    }
    if bundle.schema_version > SCHEMA_VERSION {
        return Err(BundleError::VersionTooNew {
            bundle: bundle.schema_version,
            supported: SCHEMA_VERSION,
        });
    }

    let mut plan = ImportPlan::default();

    // Does a uid exist (or will one after auth validation)? Drives whether
    // per-user domains apply (§5.7).
    let auth_present = bundle
        .domains
        .get("auth")
        .and_then(Value::as_object)
        .map(|a| a.contains_key("user_auth_token"))
        .unwrap_or(false);
    let uid_will_exist =
        (opts.include_auth && auth_present) || read_last_user_id(&target.data_root).is_some();

    // Step 3+4: classify each present domain.
    for (domain, value) in &bundle.domains {
        match domain.as_str() {
            "playback" => plan_playback(value, &mut plan),
            "audio" => plan_audio(value, target, opts, live, &forced_device, &mut plan),
            "prefs" => plan_prefs(value, &mut plan),
            "integrations" => plan_integrations(value, opts, uid_will_exist, &mut plan),
            "library_folders" => plan_library_folders(value, &mut plan),
            "auth" => plan_auth(value, opts, &mut plan),
            // §1 corollary: a top-level `volume` domain is NEVER-class, always.
            v if v.eq_ignore_ascii_case("volume") => {
                plan.skipped.push(skip_line(domain, VOLUME_SKIP_WHY));
            }
            _ => {
                plan.skipped.push(skip_line(domain, UNKNOWN_WHY));
            }
        }
    }

    Ok(plan)
}
