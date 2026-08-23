// crates/qbzd/src/tui/app/worker_import_plan.rs — planning a T12
// settings-bundle import (read + parse + build the initial plan; the device
// re-pick / apply steps live in `worker_import.rs`).

use serde_json::Value;

use qbz_app::settings::bundle::{self, Bundle, ImportOptions, ProfilePaths};

use crate::paths::ProfileRoots;
use crate::tui::screens::bundle::PendingImport;

use super::worker_fns_ext::{build_live, expand_tilde};

pub(super) fn plan_import(roots: &ProfileRoots, path: &str) -> Result<PendingImport, String> {
    let text = std::fs::read_to_string(expand_tilde(path))
        .map_err(|e| format!("cannot read bundle: {e}"))?;
    let bundle = Bundle::parse(&text).map_err(|e| e.to_string())?;
    let (live, backend, devices) = build_live(&bundle);
    let target = ProfilePaths {
        config_root: roots.config.clone(),
        data_root: roots.data.clone(),
    };
    let opts = ImportOptions {
        include_auth: false,
        trust_dsd: false,
        remap: Vec::new(),
        non_tty: false,
    };
    let plan = bundle::plan(&bundle, &target, &opts, &live).map_err(|e| e.to_string())?;
    let has_auth = bundle
        .domains
        .get("auth")
        .and_then(Value::as_object)
        .and_then(|a| a.get("user_auth_token"))
        .and_then(Value::as_str)
        .map(|t| !t.is_empty())
        .unwrap_or(false);
    Ok(PendingImport {
        bundle,
        plan,
        live,
        opts,
        target,
        backend,
        devices,
        device_choice: None,
        has_auth,
        apply_with_auth: false,
    })
}
