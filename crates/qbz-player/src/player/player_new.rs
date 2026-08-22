use super::*;

impl Player {
    /// Create a new player with an optional specific output device and audio
    /// settings. If `device_name` is None, uses the system default device.
    /// `visualizer_tap` is optional — if provided, audio samples are
    /// captured for visualization.
    pub fn new(
        device_name: Option<String>,
        audio_settings: AudioSettings,
        visualizer_tap: Option<VisualizerTap>,
        diagnostic: AudioDiagnostic,
    ) -> Self {
        let state = SharedState::new();
        let thread_state = state.clone();

        let settings = Arc::new(Mutex::new(audio_settings.clone()));
        let thread_settings = settings.clone();

        let thread_viz_tap = visualizer_tap.clone();
        let thread_diagnostic = diagnostic.clone();

        let tx = audio_thread::spawn(
            device_name,
            thread_settings,
            thread_viz_tap,
            thread_diagnostic,
            thread_state,
        );

        // Two-level playback cache: L1 in memory (~400 MB), L2 on disk
        // (~800 MB). A disk-cache failure degrades to L1-only rather than
        // aborting player creation.
        let audio_cache = match qbz_cache::PlaybackCache::new(800 * 1024 * 1024) {
            Ok(pc) => Arc::new(qbz_cache::AudioCache::with_playback_cache(
                400 * 1024 * 1024,
                Arc::new(pc),
            )),
            Err(e) => {
                log::warn!("Playback disk cache unavailable: {e}; memory cache only");
                Arc::new(qbz_cache::AudioCache::new(400 * 1024 * 1024))
            }
        };

        Self {
            tx,
            state,
            audio_settings: settings,
            visualizer_tap,
            diagnostic,
            audio_cache,
        }
    }
}
