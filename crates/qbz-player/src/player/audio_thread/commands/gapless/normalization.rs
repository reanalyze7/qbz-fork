use super::super::super::*;

/// Compute the normalization gain for the next gapless track.
pub(super) fn compute_gain(
    ctx: &mut ThreadCtx,
    data: &[u8],
    track_id: u64,
    sample_rate: u32,
    channels: u16,
) -> (Option<f32>, Option<Arc<AtomicU32>>) {
    let norm_settings = ctx
        .settings
        .lock()
        .ok()
        .filter(|s| s.normalization_enabled)
        .map(|s| s.normalization_target_lufs);

    let Some(target_lufs) = norm_settings else {
        return (None, None);
    };

    let rg_gain = extract_replaygain(data).map(|rg| calculate_gain_factor(&rg, target_lufs));
    let atomic = Arc::new(AtomicU32::new(rg_gain.unwrap_or(1.0).to_bits()));
    if let Some(cached) = ctx.loudness_cache.get(track_id) {
        let cached_gain = db_to_linear(cached.gain_db.min(6.0));
        atomic.store(cached_gain.to_bits(), Ordering::Relaxed);
    }
    let _ = ctx.analyzer_tx.try_send(AnalyzerMessage::NewTrack {
        track_id,
        sample_rate,
        channels,
        target_lufs,
        gain_atomic: atomic.clone(),
    });
    (rg_gain, Some(atomic))
}
