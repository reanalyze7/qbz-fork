//! Pure dB <-> linear gain math.

#[cfg(test)]
mod tests;

use super::ReplayGainData;

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
