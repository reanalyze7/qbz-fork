use super::super::*;
use qbz_audio::LoudnessAnalyzer;

use crate::player::offline_loudness::OfflineLoudness;
use std::sync::mpsc::SyncSender;

/// Max consecutive sink-creation failures before auto-reinit gives up.
pub(crate) const MAX_SINK_FAILURES: u32 = 3;
/// Delay dropping the audio stream after pause to reduce CPU usage.
pub(crate) const PAUSE_SUSPEND_DELAY_MS: u64 = 2000;

/// Bundles everything the audio thread's command handlers need: the
/// long-lived "environment" (shared state, settings, host, analyzer/
/// loudness plumbing) plus the mutable per-track playback locals that used
/// to live as captured closure locals in `Player::new`.
///
/// Extracted from a single `thread::spawn` closure (see refactor plan) so
/// each `AudioCommand` handler can be its own free function taking
/// `&mut ThreadCtx` instead of a dozen individual `&mut` parameters.
pub(crate) struct ThreadCtx {
    pub(crate) state: SharedState,
    pub(crate) settings: Arc<Mutex<AudioSettings>>,
    pub(crate) viz_tap: Option<VisualizerTap>,
    pub(crate) diagnostic: AudioDiagnostic,
    pub(crate) analyzer_tx: SyncSender<AnalyzerMessage>,
    pub(crate) analyzer_enabled: Arc<AtomicBool>,
    pub(crate) loudness_cache: Arc<LoudnessCache>,
    pub(crate) offline_loudness: OfflineLoudness,
    pub(crate) host: rodio::cpal::Host,

    pub(crate) current_device_name: Option<String>,
    pub(crate) stream_opt: Option<StreamType>,
    pub(crate) current_track_sample_rate: Option<u32>,
    pub(crate) current_track_channels: Option<u16>,
    pub(crate) current_engine: Option<PlaybackEngine>,
    pub(crate) current_audio_data: Option<Vec<u8>>,
    pub(crate) current_streaming_source: Option<Arc<BufferedMediaSource>>,
    pub(crate) consecutive_sink_failures: u32,
    pub(crate) pause_suspend_deadline: Option<Instant>,
    pub(crate) last_empty_check: Instant,
    pub(crate) current_normalization_gain: Option<f32>,
    pub(crate) current_gain_atomic: Option<Arc<AtomicU32>>,
    pub(crate) gapless_pending: Option<GaplessPending>,
    pub(crate) gapless_request_armed: bool,
}

impl ThreadCtx {
    pub(crate) fn new(
        device_name: Option<String>,
        settings: Arc<Mutex<AudioSettings>>,
        viz_tap: Option<VisualizerTap>,
        diagnostic: AudioDiagnostic,
        state: SharedState,
    ) -> Self {
        log::info!("Audio thread starting...");

        // Initialize loudness analysis system
        let (analyzer_tx, analyzer_rx) = mpsc::sync_channel::<AnalyzerMessage>(64);
        let loudness_cache = match LoudnessCache::new() {
            Ok(c) => Arc::new(c),
            Err(e) => {
                log::error!(
                    "Failed to create loudness cache: {}. Normalization will work without caching.",
                    e
                );
                panic!("LoudnessCache creation failed: {}", e);
            }
        };
        let _analyzer_handle = LoudnessAnalyzer::spawn(analyzer_rx, loudness_cache.clone());
        let offline_loudness = OfflineLoudness::spawn(loudness_cache.clone());
        let analyzer_enabled = Arc::new(AtomicBool::new(false));
        let host = rodio::cpal::default_host();

        Self {
            state,
            settings,
            viz_tap,
            diagnostic,
            analyzer_tx,
            analyzer_enabled,
            loudness_cache,
            offline_loudness,
            host,
            current_device_name: device_name,
            stream_opt: None,
            current_track_sample_rate: None,
            current_track_channels: None,
            current_engine: None,
            current_audio_data: None,
            current_streaming_source: None,
            consecutive_sink_failures: 0,
            pause_suspend_deadline: None,
            last_empty_check: Instant::now(),
            current_normalization_gain: None,
            current_gain_atomic: None,
            gapless_pending: None,
            gapless_request_armed: false,
        }
    }
}
