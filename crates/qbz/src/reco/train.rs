//! Training (W5).

use qbz_app::settings::reco_store::TrainParams;

use super::RECO;

/// Recompute reco scores off-thread, fire-and-forget (mirrors Tauri's
/// non-awaited `trainScores` after login). Never blocks the caller; uses the
/// engine's default decay/weight params. No-op when reco is disabled.
pub fn train_async() {
    tokio::task::spawn_blocking(|| {
        if let Ok(mut guard) = RECO.lock() {
            if let Some(store) = guard.as_mut() {
                if let Err(e) = store.train(TrainParams::default()) {
                    log::warn!("[reco] train failed: {e}");
                }
            }
        }
    });
}
