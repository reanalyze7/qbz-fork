/// Downsample the interleaved stereo `samples` buffer into 256 L + 256 R
/// points for the oscilloscope. The Box allocation is load-bearing: ownership
/// moves through the sink into the UI's single-slot cell, so it cannot be
/// reused across frames.
pub(super) fn build_waveform(samples: &[f32], fft_size: usize) -> Box<[f32; 512]> {
    const WAVEFORM_POINTS: usize = 256;

    // samples[] is interleaved: L0, R0, L1, R1, ...
    // 1024 samples = 512 stereo pairs → downsample to 256 per channel
    let stereo_pairs = fft_size / 2; // 512
    let step = stereo_pairs / WAVEFORM_POINTS; // 512/256 = 2
    let mut wave = Box::new([0.0f32; 512]);
    for i in 0..WAVEFORM_POINTS {
        let base = i * step * 2; // index into interleaved buffer
        wave[i] = samples[base]; // L
        wave[WAVEFORM_POINTS + i] = samples[base + 1]; // R
    }
    wave
}
