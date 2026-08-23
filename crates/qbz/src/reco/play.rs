//! Play events (W2).

use super::RECO;

/// CRITICAL source gate: only Qobuz-catalog plays may enter reco. `None`
/// defaults to `"qobuz"` (the queue's own normalization in
/// `playback::record_recent`); only `local` / `ephemeral` carry
/// non-catalog ids that don't resolve against Qobuz and would poison the home
/// seeds. A `qobuz_download` (a purchased Qobuz track) keeps a resolvable
/// Qobuz id, so it counts. Same exclusion the mix seeder uses (`mix.rs`).
pub fn is_qobuz_source(source: Option<&str>) -> bool {
    !matches!(source.unwrap_or("qobuz"), "local" | "ephemeral")
}

/// Log a Qobuz play event. Blocking SQLite — call from `spawn_blocking`.
/// Returns whether it was logged (`false` = gated out as non-Qobuz, or reco
/// disabled). `genre_id` is `None`: a `QueueTrack` carries no genre, exactly
/// as in Tauri (genre is supplied later via the album-meta write-back).
pub fn log_play_gated(
    track_id: u64,
    album_id: Option<String>,
    artist_id: Option<u64>,
    source: Option<&str>,
) -> bool {
    if !is_qobuz_source(source) {
        return false;
    }
    if let Ok(guard) = RECO.lock() {
        if let Some(store) = guard.as_ref() {
            if let Err(e) = store.log_play_event(track_id, album_id, artist_id, None) {
                log::warn!("[reco] log_play failed: {e}");
            }
            return true;
        }
    }
    false
}
