// crates/qbzd/src/tui/app/worker_import.rs — applying a planned T12
// settings-bundle import, and exporting one (planning the import itself
// lives in `worker_import_plan.rs`).

use qbz_app::settings::bundle::{self, Bundle, ExportOptions, ExportSource, ImportOptions, LiveSystem, ProfilePaths};

use crate::login;
use crate::paths::ProfileRoots;
use crate::tui::strings as s;

use super::messages_worker::Msg;
use super::worker_fns::do_reload;
use super::worker_fns_ext::{desktop_profile_present, expand_tilde};

pub(super) async fn apply_import(
    roots: &ProfileRoots,
    bundle: Bundle,
    target: ProfilePaths,
    live: LiveSystem,
    opts: ImportOptions,
    choice: Option<bundle::DeviceChoice>,
) -> Msg {
    let plan = match &choice {
        Some(c) => bundle::replan_with_device(&bundle, &target, &opts, &live, c.clone()),
        None => bundle::plan(&bundle, &target, &opts, &live),
    };
    let plan = match plan {
        Ok(p) => p,
        Err(e) => {
            return Msg::ImportApplied {
                lines: vec![e.to_string()],
                status: None,
                reachable: false,
            }
        }
    };

    // Validate the auth token BEFORE any write (§3.6 step 5).
    let mut uid = None;
    if let Some(token) = plan.auth_token.clone() {
        match login::validate_token(&token).await {
            Ok(session) => uid = Some(session.user_id),
            Err(_) => {
                return Msg::ImportApplied {
                    lines: vec!["the Qobuz token in this bundle was rejected".to_string()],
                    status: None,
                    reachable: false,
                }
            }
        }
    }

    if let Err(e) = bundle::apply(&plan, &target, uid) {
        return Msg::ImportApplied {
            lines: vec![format!("import only partially applied: {e}")],
            status: None,
            reachable: false,
        };
    }

    let (mut lines, status, reachable) = do_reload(roots, false, plan.routing_critical_changed).await;
    let mut out = vec![s::b_import_done(
        plan.applied.len(),
        plan.adapted.len(),
        plan.skipped.len(),
    )];
    out.append(&mut lines);
    if uid.is_some() {
        out.push("logged in with the bundled account".to_string());
    }
    Msg::ImportApplied { lines: out, status, reachable }
}

pub(super) fn export_bundle(roots: &ProfileRoots, dest: &str, include_auth: bool) -> Result<Vec<String>, String> {
    let source = ExportSource::Daemon(ProfilePaths {
        config_root: roots.config.clone(),
        data_root: roots.data.clone(),
    });
    let b = bundle::export(source, &ExportOptions { include_auth }).map_err(|e| e.to_string())?;
    let path = expand_tilde(dest);
    bundle::write_bundle_file(&path, &b).map_err(|e| e.to_string())?;

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(dest)
        .to_string();
    let mut lines = vec![s::b_export_success(&name)];
    if b.contains_secrets() {
        lines.push("this file contains your Qobuz token — 0600, move it privately, delete after import".to_string());
    }
    if desktop_profile_present() {
        for l in s::B_DESKTOP_HINT.lines() {
            lines.push(l.to_string());
        }
    }
    Ok(lines)
}
