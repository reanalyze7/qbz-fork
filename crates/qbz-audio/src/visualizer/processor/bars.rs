use std::sync::Arc;

use super::log_bars::map_to_log_bars;
use super::{VizFrame, VizSink};
use super::super::NUM_BARS;

/// Smoothing factor: 0 = no smoothing, higher = more smoothing
const SMOOTHING: f32 = 0.65;

/// Map the spectrum to the 16 log-spaced bars, apply attack/decay smoothing,
/// and submit `VizFrame::Viz16`.
pub(super) fn process_bars(
    spectrum: &spectrum_analyzer::FrequencySpectrum,
    output: &mut [f32],
    smoothed: &mut [f32],
    sink: &Arc<dyn VizSink>,
) {
    map_to_log_bars(spectrum, output);

    // Apply smoothing for visual continuity
    for i in 0..NUM_BARS {
        let new = output[i];
        // Faster attack, slower decay for punchy visuals
        if new > smoothed[i] {
            smoothed[i] = smoothed[i] * 0.3 + new * 0.7; // Fast attack
        } else {
            smoothed[i] = smoothed[i] * SMOOTHING + new * (1.0 - SMOOTHING);
            // Slow decay
        }
        output[i] = smoothed[i];
    }

    let mut bars = [0.0f32; 16];
    bars.copy_from_slice(output);
    sink.submit(VizFrame::Viz16(bars));
}
