use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use crate::SpectralAnalyzer;

use super::super::{VisualizerTap, FFT_SIZE, NUM_BARS, TARGET_FPS};
use super::bars::process_bars;
use super::energy::EnergyState;
use super::waveform::build_waveform;
use super::{VizFrame, VizSink, IDLE_POLL, NUM_SPECTRAL_BANDS, SPECTRAL_SMOOTHING, SPECTRAL_UPDATE_RATE_HZ};

/// Main FFT processing loop. Reads samples from the tap's ring buffer, computes
/// all five streams at `TARGET_FPS`, and submits them to the sink. The enabled
/// path (pacing, `sample_rate` reads, `SpectralAnalyzer` cadence, DSP) matches
/// the historical Tauri loop; while DISABLED or PAUSED the thread parks (see
/// [`IDLE_POLL`]) instead of spinning at `TARGET_FPS`.
pub(super) fn run_fft_loop(tap: VisualizerTap, sink: Arc<dyn VizSink>) {
    // Pre-allocate all buffers to avoid allocations in the hot path
    let mut samples = vec![0.0f32; FFT_SIZE];
    let mut windowed = vec![0.0f32; FFT_SIZE];
    let mut output = vec![0.0f32; NUM_BARS];
    let mut smoothed = vec![0.0f32; NUM_BARS];

    let mut energy_state = EnergyState::new();
    let mut spectral_analyzer = SpectralAnalyzer::new(
        tap.sample_rate.load(Ordering::Relaxed),
        FFT_SIZE,
        NUM_SPECTRAL_BANDS,
        SPECTRAL_UPDATE_RATE_HZ,
        SPECTRAL_SMOOTHING,
    );

    let frame_duration = Duration::from_micros(1_000_000 / TARGET_FPS);

    loop {
        if !tap.enabled.load(Ordering::Relaxed) || tap.paused.load(Ordering::Relaxed) {
            // Disabled OR paused: park instead of pacing at TARGET_FPS doing
            // nothing (paused = the ring buffer receives no new samples, so
            // re-FFTing it would burn CPU on identical stale data).
            // Spurious wakeups are fine (the loop just re-checks the atomics);
            // the bounded timeout means no enable/resume path is REQUIRED to
            // unpark. Never blocks shutdown: the thread is detached and always
            // wakes.
            std::thread::park_timeout(IDLE_POLL);
            continue;
        }

        let frame_start = Instant::now();

        {
            let sample_rate = tap.sample_rate.load(Ordering::Relaxed);

            // Get samples from ring buffer
            tap.ring_buffer.snapshot(&mut samples);

            // Compact, progressive spectrogram bands for the Spectral Ribbon,
            // gated on the analyzer's own update cadence.
            if spectral_analyzer.process_audio_frame(&samples, sample_rate) {
                let spectral = spectral_analyzer.get_latest_bands();
                sink.submit(VizFrame::Spectral512(spectral.to_vec()));
            }

            // Apply Hann window to reduce spectral leakage
            let window = hann_window(&samples);
            for (i, (sample, win)) in samples.iter().zip(window.iter()).enumerate() {
                windowed[i] = sample * win;
            }

            // Compute FFT spectrum
            match samples_fft_to_spectrum(
                &windowed,
                sample_rate,
                FrequencyLimit::Range(20.0, 20000.0),
                Some(&divide_by_N_sqrt),
            ) {
                Ok(spectrum) => {
                    process_bars(&spectrum, &mut output, &mut smoothed, &sink);
                    energy_state.process(&spectrum, &sink);
                }
                Err(e) => {
                    log::debug!("FFT error: {:?}", e);
                }
            }

            // Raw waveform data for the oscilloscope (stereo L/R).
            let wave = build_waveform(&samples, FFT_SIZE);
            sink.submit(VizFrame::Wave256x2(wave));
        }

        // Maintain target FPS
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}
