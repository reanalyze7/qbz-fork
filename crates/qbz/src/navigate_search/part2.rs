use crate::*;

/// Apply a `nav::NavEntry` — the single generic re-entry point for the
/// history stack (back/forward), the startup-page restore, and deep links.
/// Split into `apply_entry_a` / `apply_entry_b` (this dir's
/// `apply_entry_a.rs` / `apply_entry_b.rs`), each handling half the
/// `NavEntry` variants; `_a` hands any variant outside its subset back
/// unconsumed for `_b` to try.
pub(crate) fn apply_entry(
    entry: nav::NavEntry,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    if let Some(entry) = apply_entry_a(entry, runtime, weak, handle, image_cache) {
        apply_entry_b(entry, runtime, weak, handle, image_cache);
    }
}
