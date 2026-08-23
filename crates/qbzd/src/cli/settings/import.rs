// crates/qbzd/src/cli/settings/import.rs — `qbzd settings import` (T12, 04
// §5.3), steps 1-5: read → parse → remap → plan → interactive device
// re-pick. Secret validation + apply + reload-nudge continue in
// `import_apply.rs`'s `finish`.

use std::io::IsTerminal;

use qbz_app::settings::bundle::{self, Bundle, ImportOptions, ProfilePaths};

use crate::paths::ProfileRoots;

use super::import_apply::finish;
use super::reload::{build_live_system, prompt_device};
use super::summary::print_summary_header;

/// `qbzd settings import FILE [--include-auth] [--trust-dsd] [--remap OLD=NEW]...
/// [--dry-run]` (⬇, 04 §5.3). read → version-gate → plan → (TTY device re-pick /
/// non-tty safe defaults) → validate secrets BEFORE any write → apply →
/// reload-nudge → three-bucket summary. Exit: 0 · 1 · 2 · 4.
pub async fn import(
    roots: &ProfileRoots,
    file: &str,
    include_auth: bool,
    trust_dsd: bool,
    remap_raw: &[String],
    dry_run: bool,
) -> i32 {
    // Step 1: read + parse.
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: cannot read bundle: {e}");
            return 1;
        }
    };
    let bundle = match Bundle::parse(&text) {
        Ok(b) => b,
        Err(bundle::BundleError::VersionMalformed) => {
            eprintln!("error: cannot read bundle: missing or non-integer schema_version");
            return 1;
        }
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    // --remap OLD=NEW (parsed + validated even though the P0 daemon skips
    // library_folders, so scripts written for P1 do not break — 04 §5.2).
    let mut remap = Vec::new();
    for r in remap_raw {
        match r.split_once('=') {
            Some((old, new)) => remap.push((old.to_string(), new.to_string())),
            None => {
                eprintln!("error: invalid --remap '{r}' — expected OLD=NEW");
                return 2;
            }
        }
    }

    let target = ProfilePaths {
        config_root: roots.config.clone(),
        data_root: roots.data.clone(),
    };
    let non_tty = !std::io::stdin().is_terminal();
    let opts = ImportOptions {
        include_auth,
        trust_dsd,
        remap,
        non_tty,
    };
    let live = build_live_system(&bundle);

    // Steps 2–4: plan.
    let mut plan = match bundle::plan(&bundle, &target, &opts, &live) {
        Ok(p) => p,
        Err(bundle::BundleError::VersionTooNew { bundle: b, supported }) => {
            eprintln!("{}", crate::cli::copy::bundle_version_too_new(b, supported));
            return 1;
        }
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    print_summary_header(&bundle);

    // Step 4: interactive device re-pick (TTY only; non-tty already fell back).
    if let Some(pick) = plan.device_pick.clone() {
        if !non_tty && !dry_run {
            let choice = prompt_device(&pick);
            plan = match bundle::replan_with_device(&bundle, &target, &opts, &live, choice) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {e}");
                    return 1;
                }
            };
        }
    }

    finish(roots, bundle, target, plan, dry_run).await
}
