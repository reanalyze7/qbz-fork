//! Resolve the legacy (non-CMAF) streaming URL for a track.

pub(super) async fn resolve_legacy_stream_url(
    client: &std::sync::Arc<tokio::sync::RwLock<Option<qbz_qobuz::QobuzClient>>>,
    track_id: u64,
) -> Result<String, String> {
    let client_guard = client.read().await;
    match client_guard.as_ref() {
        Some(qc) => qc
            .get_stream_url_with_fallback(track_id, qbz_models::Quality::UltraHiRes)
            .await
            .map(|s| s.url)
            .map_err(|e| e.to_string()),
        None => Err("QobuzClient not initialized".to_string()),
    }
}
