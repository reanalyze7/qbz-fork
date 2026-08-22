use super::*;

impl Player {
    /// Fetch a track's audio bytes for a gapless handoff: L1 memory ->
    /// L2 disk -> CMAF `download_full` (legacy full download as fallback).
    /// Does not start playback — the caller passes the bytes to
    /// `play_next`. Returns `None` only when every tier fails.
    ///
    /// Ports the L1/L2/CMAF tiers of Tauri's `v2_play_next_gapless`; the
    /// ephemeral / offline-cache / local-library tiers are intentionally
    /// omitted as they do not exist in the Slint MVP.
    pub async fn fetch_for_gapless(
        &self,
        client: &QobuzClient,
        track_id: u64,
        quality: Quality,
    ) -> Option<Vec<u8>> {
        // L1: in-memory cache.
        if let Some(cached) = self.audio_cache.get(track_id) {
            log::info!(
                "[GAPLESS] Track {track_id} from MEMORY cache ({} bytes)",
                cached.size_bytes
            );
            return Some(cached.data);
        }

        // L2: on-disk plain-FLAC playback cache. Warm L1 on the way out.
        if let Some(playback_cache) = self.audio_cache.get_playback_cache() {
            if let Some(audio_data) = playback_cache.get(track_id) {
                log::info!(
                    "[GAPLESS] Track {track_id} from DISK cache ({} bytes)",
                    audio_data.len()
                );
                self.audio_cache.insert(track_id, audio_data.clone());
                return Some(audio_data);
            }
        }

        // CMAF full download (Akamai CDN), legacy full download as
        // fallback. Warm L1 so a re-gapless / replay skips the network.
        let downloaded = match qbz_qobuz::cmaf::download_full(client, track_id, quality).await {
            Ok(data) => Some(data),
            Err(e) => {
                log::warn!("[GAPLESS] CMAF failed for track {track_id}: {e}, trying legacy");
                match client.get_stream_url_with_fallback(track_id, quality).await {
                    Ok(stream_url) => match self.download_audio(&stream_url.url).await {
                        Ok(data) => Some(data),
                        Err(e) => {
                            log::warn!("[GAPLESS] Legacy download failed for {track_id}: {e}");
                            None
                        }
                    },
                    Err(e) => {
                        log::warn!("[GAPLESS] No stream URL for {track_id}: {e}");
                        None
                    }
                }
            }
        };

        if let Some(ref data) = downloaded {
            log::info!(
                "[GAPLESS] Track {track_id} downloaded for gapless ({} bytes)",
                data.len()
            );
            self.audio_cache.insert(track_id, data.clone());
        } else {
            log::info!("[GAPLESS] Track {track_id} not available, gapless not possible");
        }
        downloaded
    }
}
