//! Rebuilding and re-pushing the full settings snapshot, used after a
//! cross-setting cascade.

use std::sync::Arc;

use crate::settings::snapshot::{apply_snapshot, load_snapshot};
use crate::settings::store::SettingsCtx;
use crate::AppWindow;

/// Rebuild the full snapshot off the UI thread and push it onto
/// `SettingsState`. Used after a cross-setting cascade so the UI reflects
/// every forced change (and the conditional flags) in one shot.
pub(in crate::settings) async fn rebuild_and_push(ctx: Arc<SettingsCtx>, weak: slint::Weak<AppWindow>) {
    let snap = match tokio::task::spawn_blocking(move || load_snapshot(&ctx)).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("[qbz-slint] settings cascade rebuild task failed: {e}");
            return;
        }
    };
    let _ = weak.upgrade_in_event_loop(move |w| {
        apply_snapshot(&w, snap);
    });
}
