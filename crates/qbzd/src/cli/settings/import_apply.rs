// crates/qbzd/src/cli/settings/import_apply.rs — `settings import`'s steps
// 5-7: validate secrets BEFORE any write, dry-run short-circuit, apply, and
// the reload-nudge + three-bucket summary.

use qbz_app::settings::bundle::{self, Bundle, ImportPlan, ProfilePaths};

use crate::paths::ProfileRoots;

use super::nudge::nudge_outcome;
use super::reload::reload_disposition;
use super::summary::print_buckets;

/// Continues `import()` once the plan (post device-repick) is settled.
pub(super) async fn finish(
    roots: &ProfileRoots,
    bundle: Bundle,
    target: ProfilePaths,
    plan: ImportPlan,
    dry_run: bool,
) -> i32 {
    // Step 5: validate secrets BEFORE any write (rejected → exit 4, nothing done).
    let mut validated_uid: Option<u64> = None;
    let mut auth_note: Option<String> = None;
    if let Some(token) = plan.auth_token.clone() {
        match crate::login::validate_token(&token).await {
            Ok(session) => {
                validated_uid = Some(session.user_id);
                let mut note =
                    format!("Qobuz token validated — logged in as user {}", session.user_id);
                if let Some(bid) = plan.bundle_user_id {
                    if bid != session.user_id {
                        note.push_str(&format!(
                            "\n  note: bundle user_id {bid} differs from the validated login {}",
                            session.user_id
                        ));
                    }
                }
                auth_note = Some(note);
            }
            Err(_) => {
                eprintln!("{}", crate::cli::copy::bundle_token_rejected());
                return 4;
            }
        }
    }

    // Dry-run stops after step 5 (04 §5.1): same summary, writes nothing.
    if dry_run {
        print_buckets(&plan, &bundle, auth_note.as_deref(), None);
        println!("\ndry-run: no changes written");
        return 0;
    }

    // Step 6: apply (validate-all-then-apply; re-run is idempotent).
    if let Err(e) = bundle::apply(&plan, &target, validated_uid) {
        print_buckets(&plan, &bundle, auth_note.as_deref(), None);
        eprintln!(
            "\nerror: settings only partially applied: {e}\n  → fix the disk/permissions, then re-run (import is idempotent)"
        );
        return 1;
    }

    // Step 7: reload-nudge a running daemon. Three states (04 §5.3 step 7):
    // reloaded / not running (fine) / up-but-refused (exit 1, restart hint).
    let outcome = nudge_outcome(roots);
    let (done_line, stderr_msg, exit) =
        reload_disposition(outcome, plan.routing_critical_changed);

    print_buckets(&plan, &bundle, auth_note.as_deref(), Some(&done_line));
    if let Some(msg) = stderr_msg {
        eprintln!("\n{msg}");
    }
    exit
}
