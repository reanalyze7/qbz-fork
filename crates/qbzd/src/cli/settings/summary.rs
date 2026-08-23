// crates/qbzd/src/cli/settings/summary.rs — the import flow's bundle header
// + three-bucket (applied/adapted/skipped) summary print, EXACTLY in the
// 04 §5.4 format.

use qbz_app::settings::bundle::{Bundle, ImportPlan};

pub(super) fn print_summary_header(bundle: &Bundle) {
    let date = bundle
        .created_at
        .split('T')
        .next()
        .unwrap_or(&bundle.created_at);
    println!(
        "bundle: schema v{} — exported {} from \"{}\" (qbz {}, {} profile)",
        bundle.schema_version,
        date,
        bundle.source.hostname,
        bundle.source.app_version,
        bundle.source.profile
    );
    println!();
}

/// The three-bucket summary EXACTLY in the 04 §5.4 format: applied (`= value`),
/// adapted (`old -> new (why)`), skipped (`+why`), the desktop shared-name
/// advisory, the auth footer, and the `done:` line.
pub(super) fn print_buckets(
    plan: &ImportPlan,
    _bundle: &Bundle,
    auth_note: Option<&str>,
    reload_line: Option<&str>,
) {
    let width = plan
        .applied
        .iter()
        .chain(&plan.adapted)
        .chain(&plan.skipped)
        .map(|l| l.key.len())
        .max()
        .unwrap_or(0)
        .min(44);

    println!("applied ({})", plan.applied.len());
    for l in &plan.applied {
        let note = if l.why.is_empty() {
            String::new()
        } else {
            format!(" ({})", l.why)
        };
        println!("  {:width$} = {}{}", l.key, l.new, note);
    }

    println!("\nadapted ({})", plan.adapted.len());
    for l in &plan.adapted {
        println!(
            "  {:width$} {} -> {} ({})",
            l.key,
            l.old.as_deref().unwrap_or(""),
            l.new,
            l.why
        );
    }

    println!("\nskipped ({})", plan.skipped.len());
    for l in &plan.skipped {
        println!("  {:width$} {}", l.key, l.why);
    }

    if let Some(note) = auth_note {
        println!("\nauth");
        println!("  {note}");
    }

    if let Some(reload) = reload_line {
        println!(
            "\ndone: {} applied, {} adapted, {} skipped — {}",
            plan.applied.len(),
            plan.adapted.len(),
            plan.skipped.len(),
            reload
        );
    }
}
