//! DSD (.dsf/.dff) audible playback, split out of `files.rs` to keep both
//! files under the line budget.

use super::super::loading::clear_loading;
use super::super::Runtime;
use crate::AppWindow;

/// Play a DSD file, converted to PCM on the fly by the player (qbz-dsd
/// Phase 1) — streamed from disk, never slurped through play_data. Errors
/// here are expected user-facing cases (DST-compressed DFF, >2ch) → toast
/// + stop, the queue stays usable. Returns after handling regardless of
/// outcome — the caller has nothing left to do for a DSD path.
pub(super) async fn play_dsd(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    path: String,
    row_id: u64,
) {
    let exists_path = path.clone();
    let exists = tokio::task::spawn_blocking(move || std::path::Path::new(&exists_path).exists())
        .await
        .unwrap_or(false);
    if !exists {
        log::error!("[qbz-slint] local play: DSD file not available at {path}");
        crate::toast::show_weak(
            weak,
            qbz_i18n::t("File not available — is the drive mounted?"),
            crate::ToastKind::Warning,
        );
        clear_loading(weak, row_id);
        return;
    }
    if let Err(e) = runtime.core().player().play_dsd_file(std::path::PathBuf::from(&path), row_id) {
        log::error!("[qbz-slint] local play: play_dsd_file {row_id} failed: {e}");
        crate::toast::show_weak(
            weak,
            format!("{}: {e}", qbz_i18n::t("Cannot play DSD file")),
            crate::ToastKind::Warning,
        );
        clear_loading(weak, row_id);
    }
}
