// ============================ Import / Export (§3.6) ============================

pub const BUNDLE_TITLE: &str = "Import / Export";
pub const BUNDLE_IMPORT_HEADER: &str = "IMPORT";
pub const BUNDLE_EXPORT_HEADER: &str = "EXPORT";

pub const B_IMPORT_PATH: &str = "Bundle file";
pub const B_IMPORT_PATH_HINT: &str = "path to a .qbzb (scp it to ~ first)";
pub const B_IMPORT_ACTION: &str = "Review import";
pub const B_EXPORT_DEST: &str = "Destination";
pub const B_EXPORT_INCLUDE_AUTH: &str = "Include Qobuz login";
pub const B_EXPORT_ACTION: &str = "Export";

pub const B_BUCKET_APPLIED: &str = "applies verbatim";
pub const B_BUCKET_ADAPTED: &str = "needs your confirmation";
pub const B_BUCKET_SKIPPED: &str = "skipped";

/// Import-side auth gate (§3.6 step 5) — dedicated, default-OFF.
pub const B_IMPORT_AUTH_TITLE: &str = "Bundle carries a Qobuz login";
pub const B_IMPORT_AUTH_BODY: &str =
    "Also log in with the bundled account? The token is validated with Qobuz\nbefore anything is stored.";
pub const B_IMPORT_AUTH_HINT: &str = "y log in · Esc skip auth";

/// Export include-auth warning (§3.6, shown while the toggle is on).
pub const B_EXPORT_AUTH_WARNING: &str = "embeds your decrypted Qobuz token — anyone with this file can use your\naccount. File is written 0600; move it privately (scp), delete after import.";

pub fn b_export_success(path: &str) -> String {
    format!("saved. on the daemon box: qbzd settings import {path}")
}
/// Success-panel hint when a desktop profile is detected (§3.6): desktop export
/// is the CLI's job.
pub const B_DESKTOP_HINT: &str =
    "a desktop QBZ profile was found on this box — to export IT instead:\n  qbzd settings export --from desktop";

pub fn b_import_done(applied: usize, adapted: usize, skipped: usize) -> String {
    format!("imported: {applied} applied, {adapted} adapted, {skipped} skipped")
}
