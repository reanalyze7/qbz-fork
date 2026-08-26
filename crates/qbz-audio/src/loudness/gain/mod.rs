//! Pure dB <-> linear gain math.

#[cfg(test)]
mod tests;

use super::ReplayGainData;

// ─── Bornes de normalisation ──────────────────────────────────────────────
//
// Une mesure EBU R128 faite sur un fragment (intro calme, fin en fondu, ou
// pire : la fin du morceau PRECEDENT) donne un LUFS absurdement bas et donc
// un boost gigantesque. Ces bornes existent pour qu'une mesure douteuse ne
// puisse jamais s'entendre, et pour qu'on refuse de la mettre en cache.

/// Boost maximum applique (prevention du clipping, conservateur).
pub const MAX_GAIN_DB: f32 = 6.0;

/// Attenuation maximum appliquee.
pub const MIN_GAIN_DB: f32 = -20.0;

/// En dessous, la mesure porte sur du quasi-silence : elle n'est pas
/// representative du morceau et ne doit ni etre appliquee ni etre cachee.
pub const MIN_PLAUSIBLE_LUFS: f32 = -40.0;

/// Au dessus, la mesure est impossible pour un master reel.
pub const MAX_PLAUSIBLE_LUFS: f32 = 0.0;

/// Une mesure exploitable : finie, et dans la plage des masters reels.
pub fn is_plausible_lufs(lufs: f32) -> bool {
    lufs.is_finite() && (MIN_PLAUSIBLE_LUFS..=MAX_PLAUSIBLE_LUFS).contains(&lufs)
}

/// Ecart en dB a appliquer pour amener `measured_lufs` sur `target_lufs`,
/// borne aux valeurs sures.
pub fn gain_db_for(measured_lufs: f32, target_lufs: f32) -> f32 {
    (target_lufs - measured_lufs).clamp(MIN_GAIN_DB, MAX_GAIN_DB)
}

/// Idem, en facteur lineaire directement applicable aux echantillons.
pub fn gain_factor_for(measured_lufs: f32, target_lufs: f32) -> f32 {
    db_to_linear(gain_db_for(measured_lufs, target_lufs))
}


/// Convert a gain in dB to a linear amplitude factor.
///
/// gain_db = 0.0  → factor = 1.0 (no change)
/// gain_db = -6.0 → factor ≈ 0.501 (half amplitude)
/// gain_db = +6.0 → factor ≈ 1.995 (double amplitude)
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    10_f32.powf(db / 20.0)
}

/// Calculate the normalization gain factor for a track.
///
/// Takes ReplayGain metadata and a target LUFS level, returns the linear
/// gain factor to apply to samples. Includes clipping prevention using
/// peak data when available.
///
/// The ReplayGain standard targets -18 LUFS (83 dB SPL). If the user's
/// target differs, we adjust accordingly.
///
/// # Arguments
/// * `rg` - ReplayGain metadata extracted from the track
/// * `target_lufs` - User's target loudness (e.g., -14.0, -18.0, -23.0)
///
/// # Returns
/// Linear gain factor to multiply samples by
pub fn calculate_gain_factor(rg: &ReplayGainData, target_lufs: f32) -> f32 {
    // ReplayGain reference level is -18 LUFS (EBU R128 / ReplayGain 2.0)
    const REPLAYGAIN_REFERENCE_LUFS: f32 = -18.0;

    // Adjust gain for the user's target level
    // If target is -14 LUFS (louder than reference), we need to add +4 dB
    // If target is -23 LUFS (quieter), we need to subtract -5 dB
    let target_adjustment = target_lufs - REPLAYGAIN_REFERENCE_LUFS;
    let adjusted_gain_db = rg.gain_db + target_adjustment;

    let mut gain = db_to_linear(adjusted_gain_db);

    // Clipping prevention: if we have peak data, cap the gain so
    // the loudest sample doesn't exceed 1.0
    if let Some(peak) = rg.peak {
        if peak > 0.0 {
            let max_safe_gain = 1.0 / peak;
            if gain > max_safe_gain {
                log::debug!(
                    "Loudness: capping gain from {:.3} to {:.3} (peak: {:.4})",
                    gain,
                    max_safe_gain,
                    peak
                );
                gain = max_safe_gain;
            }
        }
    }

    // Without peak data, cap at +6 dB maximum (conservative)
    if rg.peak.is_none() {
        let max_gain = db_to_linear(6.0);
        if gain > max_gain {
            log::debug!("Loudness: capping gain to +6 dB (no peak data)");
            gain = max_gain;
        }
    }

    log::debug!(
        "Loudness: gain_db={:.2}, target={:.1} LUFS, adjusted={:.2} dB, factor={:.4}",
        rg.gain_db,
        target_lufs,
        adjusted_gain_db,
        gain
    );

    gain
}
