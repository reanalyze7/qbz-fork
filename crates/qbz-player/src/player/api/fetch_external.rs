use super::*;

impl Player {
    /// Resolve a fully-materialized audio asset for an EXTERNAL renderer
    /// (Chromecast / DLNA), carrying the bytes verbatim plus the MIME and the
    /// quality. Used by the Cast path through the local media server.
    ///
    /// Cache-first (P1, matches the fast Tauri cast path + consumes the gapless
    /// prefetch): L1 in-memory -> L2 on-disk playback cache (both decrypted
    /// FLAC) -> network. A prefetched/replayed track is served instantly; only a
    /// cold track pays the CMAF download. On a cache hit the delivered quality is
    /// not known here (no metadata stored with the bytes) — the caller derives
    /// the quality label from the track's catalog metadata; the network path
    /// returns the precise resolved tier.
    pub async fn fetch_for_external_stream(
        &self,
        client: &QobuzClient,
        track_id: u64,
        quality: Quality,
    ) -> Option<ExternalStreamAsset> {
        // L1: in-memory cache (warmed by the gapless prefetch / a prior play).
        if let Some(cached) = self.audio_cache.get(track_id) {
            log::info!(
                "[CAST-FETCH] Track {track_id} from MEMORY cache ({} bytes)",
                cached.size_bytes
            );
            return Some(ExternalStreamAsset {
                bytes: cached.data,
                content_type: "audio/flac".to_string(),
                quality: StreamQualityInfo::from_raw(0, None, None),
                duration_secs: None,
                origin: AssetOrigin::Cache,
            });
        }
        // L2: on-disk plain-FLAC playback cache; warm L1 on the way out.
        if let Some(playback_cache) = self.audio_cache.get_playback_cache() {
            if let Some(audio_data) = playback_cache.get(track_id) {
                log::info!(
                    "[CAST-FETCH] Track {track_id} from DISK cache ({} bytes)",
                    audio_data.len()
                );
                self.audio_cache.insert(track_id, audio_data.clone());
                return Some(ExternalStreamAsset {
                    bytes: audio_data,
                    content_type: "audio/flac".to_string(),
                    quality: StreamQualityInfo::from_raw(0, None, None),
                    duration_secs: None,
                    origin: AssetOrigin::Cache,
                });
            }
        }

        // Cold: CMAF full download (Akamai CDN) -> decrypted FLAC.
        match qbz_qobuz::cmaf::download_full_with_quality(client, track_id, quality).await {
            Ok((bytes, q)) => {
                log::info!(
                    "[CAST-FETCH] Track {track_id} via CMAF: {} bytes, format_id={}, {:?} kHz/{:?}-bit",
                    bytes.len(),
                    q.format_id,
                    q.sampling_rate_khz,
                    q.bit_depth
                );
                // Warm L1 so a subsequent local replay skips the network.
                self.audio_cache.insert(track_id, bytes.clone());
                return Some(ExternalStreamAsset {
                    bytes,
                    content_type: "audio/flac".to_string(),
                    quality: q,
                    duration_secs: None,
                    origin: AssetOrigin::Network,
                });
            }
            Err(e) => {
                log::warn!("[CAST-FETCH] CMAF failed for track {track_id}: {e}, trying legacy");
            }
        }

        // Fallback: legacy stream URL + plain HTTP download. Quality and MIME
        // come from the resolved StreamUrl (which carries the granted tier).
        match client.get_stream_url_with_fallback(track_id, quality).await {
            Ok(stream_url) => {
                let content_type =
                    external_content_type(&stream_url.mime_type, stream_url.format_id);
                let q = StreamQualityInfo::from_raw(
                    stream_url.format_id,
                    Some(stream_url.sampling_rate),
                    stream_url.bit_depth,
                );
                match self.download_audio(&stream_url.url).await {
                    Ok(bytes) => {
                        log::info!(
                            "[CAST-FETCH] Track {track_id} via legacy: {} bytes, format_id={}, ct={}",
                            bytes.len(),
                            q.format_id,
                            content_type
                        );
                        self.audio_cache.insert(track_id, bytes.clone());
                        Some(ExternalStreamAsset {
                            bytes,
                            content_type,
                            quality: q,
                            duration_secs: None,
                            origin: AssetOrigin::Network,
                        })
                    }
                    Err(e) => {
                        log::warn!("[CAST-FETCH] Legacy download failed for {track_id}: {e}");
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("[CAST-FETCH] No stream URL for {track_id}: {e}");
                None
            }
        }
    }
}
