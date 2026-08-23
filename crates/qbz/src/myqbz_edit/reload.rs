//! Reload the open detail view after a mutation.

use crate::artwork::ImageCache;
use crate::AppWindow;

/// Reload the open detail view for `id` (Tauri's "-> reload") so the hero +
/// toolbar reflect the mutation. Re-runs the detail navigator's
/// load/apply/artwork path; the inner `set_view` is harmless (already there).
pub(super) fn reload(
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &ImageCache,
    id: String,
) {
    let Some(runtime) = crate::myqbz_detail::global_runtime() else {
        return;
    };
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let _ = weak.upgrade_in_event_loop(move |w| {
        crate::myqbz_detail::navigate(runtime, w.as_weak(), handle.clone(), image_cache.clone(), id);
    });
}
