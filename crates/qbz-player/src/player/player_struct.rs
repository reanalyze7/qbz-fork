use super::*;

/// Audio player that handles streaming playback
/// Uses a dedicated thread for audio output
pub struct Player {
    /// Channel to send commands to the audio thread
    pub(crate) tx: Sender<AudioCommand>,
    /// Shared state accessible from any thread
    pub state: SharedState,
    /// Audio settings (exclusive mode, DAC passthrough, etc.)
    pub(crate) audio_settings: Arc<Mutex<AudioSettings>>,
    /// Visualizer tap for audio sample capture (optional)
    #[allow(dead_code)]
    pub(crate) visualizer_tap: Option<VisualizerTap>,
    /// Bit-depth diagnostic capture (always available, zero-cost when idle)
    pub diagnostic: AudioDiagnostic,
    /// Two-level playback cache (L1 memory + optional L2 disk). A track is
    /// cached after its first play so replays start instantly.
    pub(crate) audio_cache: Arc<qbz_cache::AudioCache>,
}

impl Default for Player {
    fn default() -> Self {
        Self::new(None, AudioSettings::default(), None, AudioDiagnostic::new())
    }
}
