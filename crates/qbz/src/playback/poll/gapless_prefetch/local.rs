//! LOCAL-track gapless pre-queue (DSD plan Phase 2): resolve the file path
//! and hand it to the engine's gapless queue.

use super::super::super::Runtime;

/// Spawn the resolve-and-hand-off task for `next_id`, the upcoming LOCAL
/// queue track. DSD goes through `play_next_dsd` (DoP append when a DoP
/// stream is live, else an in-memory converted WAV); other local formats
/// feed their raw bytes to the normal `play_next`. CUE virtual tracks are
/// skipped (they share one album image file).
pub(super) fn spawn_fetch(runtime: &Runtime, next_id: u64) {
    let runtime = runtime.clone();
    tokio::spawn(async move {
        let info = if crate::ephemeral::is_ephemeral_id(next_id as i64) {
            crate::ephemeral::get_track(next_id as i64).map(|t| (t.file_path, t.cue_start_secs))
        } else {
            tokio::task::spawn_blocking(move || {
                crate::library_db::with_db(|db| db.get_track(next_id as i64))
            })
            .await
            .ok()
            .flatten()
            .flatten()
            .map(|t| (t.file_path, t.cue_start_secs))
        };
        let Some((path, cue)) = info else { return };
        if cue.is_some() {
            return;
        }
        let rt2 = runtime.clone();
        let res = tokio::task::spawn_blocking(move || {
            let p = std::path::PathBuf::from(&path);
            let player = rt2.core().player();
            if qbz_dsd::is_dsd_path(&p) {
                player.play_next_dsd(p, next_id)
            } else {
                let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
                player.play_next(bytes, next_id)
            }
        })
        .await;
        match res {
            Ok(Ok(())) => log::info!("[qbz-slint] [GAPLESS] queued local track {next_id} for gapless"),
            Ok(Err(e)) => log::info!("[qbz-slint] [GAPLESS] local pre-queue {next_id} skipped: {e}"),
            Err(e) => log::warn!("[qbz-slint] [GAPLESS] local pre-queue task failed: {e}"),
        }
    });
}
