//! Offline/network tier-walk playback resolution: prefer an
//! offline-cached copy before falling through to the player's own
//! L1/L2 -> network path.

use qbz_models::{AssetOrigin, ExternalStreamAsset, FrontendAdapter, Quality, StreamQualityInfo};

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Play `track_id` preferring an offline-cached copy (the offline tier of
    /// the shared playback tier-walk) before the player's own L1/L2 → network
    /// path. `offline` is the frontend's open `OfflineCacheState` (None = no
    /// offline tier); `sink` optionally drives the unlock animation.
    ///
    /// The player is untouched: an offline hit is handed to `play_data` (which
    /// warms L1), a miss falls through to `Player::play_track`.
    pub async fn play_track_resolved(
        &self,
        track_id: u64,
        quality: Quality,
        offline: Option<&qbz_offline_cache::OfflineCacheState>,
        sink: Option<&qbz_offline_cache::CacheEventSink>,
        start_position_secs: u64,
    ) -> Result<(), String> {
        if let Some(off) = offline {
            if let Some(bytes) =
                crate::offline_resolve::resolve_offline_bytes(track_id, off, sink).await
            {
                log::info!("[Core] track {} served from OFFLINE cache", track_id);
                let r = self.player.play_data(bytes, track_id);
                // Offline-cached bytes play from memory; honor a session-resume
                // position with a best-effort seek (0 = from the start).
                if r.is_ok() && start_position_secs > 0 {
                    let _ = self.player.seek(start_position_secs);
                }
                return r;
            }
        }
        let guard = self.client.read().await;
        let client = guard
            .as_ref()
            .ok_or_else(|| "No Qobuz client available".to_string())?;
        self.player
            .play_track(client, track_id, quality, start_position_secs)
            .await
    }

    /// Resolve the bytes for a GAPLESS successor. Tier order L1/L2 → offline →
    /// network: the offline tier is checked only when the track is NOT already
    /// in the player's cache (the CMAF decrypt is slow, ~5-7 s, so a cached
    /// copy wins). Returns bytes to hand to `Player::play_next`, or None.
    pub async fn fetch_for_gapless_resolved(
        &self,
        track_id: u64,
        quality: Quality,
        offline: Option<&qbz_offline_cache::OfflineCacheState>,
        sink: Option<&qbz_offline_cache::CacheEventSink>,
    ) -> Option<Vec<u8>> {
        if !self.player.is_track_cached(track_id) {
            if let Some(off) = offline {
                if let Some(bytes) =
                    crate::offline_resolve::resolve_offline_bytes(track_id, off, sink).await
                {
                    return Some(bytes);
                }
            }
        }
        let guard = self.client.read().await;
        let client = guard.as_ref()?;
        self.player.fetch_for_gapless(client, track_id, quality).await
    }

    /// Resolve a fully-materialized audio asset (bytes + MIME + quality) for an
    /// EXTERNAL renderer (Chromecast / DLNA). Tier order mirrors
    /// `fetch_for_gapless_resolved`: L1/L2 player cache -> OFFLINE (local CMAF
    /// decrypt, no network) -> network. The offline tier is what makes a
    /// downloaded track cast fast with no connection (the same "local segments,
    /// decrypt on demand" path the offline cache uses for playback). On a cache
    /// or offline hit the precise delivered quality isn't known here — the Cast
    /// service derives the quality label from the track's catalog metadata.
    pub async fn fetch_for_external_stream_resolved(
        &self,
        track_id: u64,
        quality: Quality,
        offline: Option<&qbz_offline_cache::OfflineCacheState>,
        sink: Option<&qbz_offline_cache::CacheEventSink>,
    ) -> Option<ExternalStreamAsset> {
        // L1/L2 player cache (decrypted FLAC) is handled inside
        // fetch_for_external_stream; only reach for the offline tier when the
        // track is not already cached (the CMAF decrypt is slow).
        if !self.player.is_track_cached(track_id) {
            if let Some(off) = offline {
                if let Some(bytes) =
                    crate::offline_resolve::resolve_offline_bytes(track_id, off, sink).await
                {
                    log::info!("[CAST-FETCH] track {track_id} served from OFFLINE cache");
                    return Some(ExternalStreamAsset {
                        bytes,
                        content_type: "audio/flac".to_string(),
                        quality: StreamQualityInfo::from_raw(0, None, None),
                        duration_secs: None,
                        origin: AssetOrigin::Offline,
                    });
                }
            }
        }
        let guard = self.client.read().await;
        let client = guard.as_ref()?;
        self.player
            .fetch_for_external_stream(client, track_id, quality)
            .await
    }
}
