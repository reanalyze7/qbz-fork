use super::super::super::*;
use crate::player::command::PendingNormalization;
use crate::player::offline_loudness::OfflineJob;
use qbz_audio::loudness::gain::gain_factor_for;

/// Cible de normalisation active, ou `None` si la normalisation est coupee.
pub(super) fn target_lufs(ctx: &ThreadCtx) -> Option<f32> {
    ctx.settings
        .lock()
        .ok()
        .filter(|s| s.normalization_enabled)
        .map(|s| s.normalization_target_lufs)
}

/// Mesure le morceau hors-ligne pour le cache seul.
///
/// Utilise quand le gapless est refuse (changement de frequence, source
/// streaming...) : le titre sera demarre plus tard par un autre chemin, qui
/// trouvera la mesure deja faite et posera le bon volume des la premiere note
/// au lieu d'attendre d'avoir entendu le morceau.
pub(super) fn preanalyze_only(ctx: &ThreadCtx, data: &[u8], track_id: u64) {
    let Some(target) = target_lufs(ctx) else {
        return;
    };
    if ctx.loudness_cache.has(track_id) {
        return;
    }
    ctx.offline_loudness.submit(OfflineJob::cache_only(
        track_id,
        Arc::new(data.to_vec()),
        target,
    ));
}

/// Prepare la normalisation du prochain morceau gapless.
///
/// N'envoie PAS `NewTrack` : l'analyseur ne bascule qu'a la transition reelle
/// (voir `PendingNormalization`). Lance en revanche la pre-analyse hors-ligne,
/// qui a ~10 s pour mesurer le morceau avant qu'il commence.
pub(super) fn prepare(
    ctx: &mut ThreadCtx,
    data: &[u8],
    track_id: u64,
    sample_rate: u32,
    channels: u16,
) -> (
    Option<f32>,
    Option<Arc<AtomicU32>>,
    Option<PendingNormalization>,
) {
    let Some(target_lufs) = target_lufs(ctx) else {
        return (None, None, None);
    };

    let rg_gain = extract_replaygain(data).map(|rg| calculate_gain_factor(&rg, target_lufs));
    let atomic = Arc::new(AtomicU32::new(rg_gain.unwrap_or(1.0).to_bits()));
    let started = Arc::new(AtomicBool::new(false));

    if let Some(cached) = ctx.loudness_cache.get(track_id) {
        let gain = gain_factor_for(cached.measured_lufs, target_lufs);
        atomic.store(gain.to_bits(), Ordering::Relaxed);
        log::info!(
            "Gapless: mesure connue pour la piste {} ({:.1} LUFS), gain {:.4}",
            track_id,
            cached.measured_lufs,
            gain
        );
    } else {
        ctx.offline_loudness.submit(OfflineJob::for_pending(
            track_id,
            Arc::new(data.to_vec()),
            target_lufs,
            atomic.clone(),
            started.clone(),
        ));
    }

    let pending = PendingNormalization {
        sample_rate,
        channels,
        target_lufs,
        gain_atomic: atomic.clone(),
        started,
    };
    (rg_gain, Some(atomic), Some(pending))
}
