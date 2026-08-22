/// Map spectrum data to logarithmically-spaced frequency bars.
///
/// Human hearing is logarithmic, so we use log-spaced bars to match how we
/// perceive frequency. This gives equal visual weight to bass, mids, and treble.
pub(super) fn map_to_log_bars(spectrum: &spectrum_analyzer::FrequencySpectrum, output: &mut [f32]) {
    let num_bars = output.len();

    // Frequency range (Hz)
    const MIN_FREQ: f32 = 20.0;
    const MAX_FREQ: f32 = 20000.0;

    let min_log = MIN_FREQ.ln();
    let max_log = MAX_FREQ.ln();

    // Get spectrum data
    let data = spectrum.data();

    for (i, bar) in output.iter_mut().enumerate() {
        // Calculate logarithmic frequency bounds for this bar
        let t_low = i as f32 / num_bars as f32;
        let t_high = (i + 1) as f32 / num_bars as f32;

        let freq_low = (min_log + (max_log - min_log) * t_low).exp();
        let freq_high = (min_log + (max_log - min_log) * t_high).exp();

        // Find all frequency bins that fall within this bar's range
        let mut sum = 0.0f32;
        let mut count = 0u32;

        for (freq, magnitude) in data.iter() {
            let f = freq.val();
            if f >= freq_low && f < freq_high {
                // Apply perceptual weighting (boost bass slightly)
                let weight = if f < 200.0 {
                    1.5 // Bass boost
                } else if f < 2000.0 {
                    1.0 // Mids
                } else {
                    0.8 // Reduce harsh highs
                };

                sum += magnitude.val() * weight;
                count += 1;
            }
        }

        // Average magnitude for this bar
        let avg = if count > 0 { sum / count as f32 } else { 0.0 };

        // Apply dynamic range compression and normalize.
        // This makes quiet passages more visible while preventing clipping.
        let compressed = (avg * 4.0).powf(0.6);
        *bar = compressed.clamp(0.0, 1.0);
    }
}
