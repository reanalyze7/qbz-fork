use super::*;

pub struct PlaybackEvent {
    pub is_playing: bool,
    pub position: u64,
    pub duration: u64,
    pub track_id: u64,
    pub volume: f32,
    /// Actual sample rate of the current stream (Hz)
    pub sample_rate: Option<u32>,
    /// Actual bit depth of the current stream
    pub bit_depth: Option<u32>,
    /// Queue shuffle state
    pub shuffle: Option<bool>,
    /// Queue repeat mode ("off", "all", "one")
    pub repeat: Option<String>,
    /// Normalization gain factor being applied (None = normalization not active)
    pub normalization_gain: Option<f32>,
    /// True when backend wants the next track pre-queued for gapless playback
    #[serde(default)]
    pub gapless_ready: bool,
    /// Track ID of the gapless-queued next track (0 = none queued)
    #[serde(default)]
    pub gapless_next_track_id: u64,
    /// Bit-perfect mode of the current stream. None when no stream is active.
    /// Lets the UI show whether playback is direct-hardware bit-perfect, going
    /// through plughw software resample, or running on a shared system path
    /// (pipewire/pulse/cpal) where bit-perfect is not guaranteed.
    #[serde(default)]
    pub bit_perfect_mode: Option<BitPerfectMode>,
    /// Streaming buffer progress (0.0..1.0). `None` when not streaming or
    /// the track is fully buffered — drives the seek-bar cache overlay.
    #[serde(default)]
    pub buffer_progress: Option<f32>,
}

/// Shared state between main thread and audio thread
#[derive(Clone)]
pub struct SharedState {
    /// Is currently playing
    pub(crate) is_playing: Arc<AtomicBool>,
    /// Current position in seconds
    pub(crate) position: Arc<AtomicU64>,
    /// Total duration in seconds
    pub(crate) duration: Arc<AtomicU64>,
    /// Current track ID
    pub(crate) current_track_id: Arc<AtomicU64>,
    /// DSD-direct mode: 0 = none, 1 = DoP, 2 = native DSD_U32_BE,
    /// 3 = native DSD_U32_LE. Non-zero means volume is fixed and seek is
    /// unsupported; the gapless arm uses it to build the matching packing.
    pub(crate) dsd_direct: Arc<std::sync::atomic::AtomicU8>,
    /// True when audio data/source is available for playback or resume
    pub(crate) has_loaded_audio: Arc<AtomicBool>,
    /// Volume (0.0 - 1.0, f32 stored as u32 bits — same idiom as
    /// `normalization_gain`; integer-percent storage quantized the volume
    /// to 1% on every re-apply)
    pub(crate) volume: Arc<AtomicU32>,
    /// Playback start time (Unix timestamp millis when started/resumed)
    pub(crate) playback_start_millis: Arc<AtomicU64>,
    /// Position when playback was started/resumed (in seconds)
    pub(crate) position_at_start: Arc<AtomicU64>,
    /// Current output device name
    pub(crate) current_device: Arc<std::sync::RwLock<Option<String>>>,
    /// Stream error flag (set when ALSA/audio errors are detected)
    pub(crate) stream_error: Arc<AtomicBool>,
    /// Optional user-readable explanation paired with `stream_error`.
    /// Drained by the Tauri polling loop to emit a frontend toast and then
    /// cleared, so the UI fires the notification exactly once per error.
    pub(crate) stream_error_message: Arc<std::sync::RwLock<Option<String>>>,
    /// Actual sample rate of the current stream (Hz)
    pub(crate) sample_rate: Arc<AtomicU32>,
    /// Actual bit depth of the current stream
    pub(crate) bit_depth: Arc<AtomicU32>,
    /// Current normalization gain factor (f32 stored as u32 bits, 0 = not applied)
    pub(crate) normalization_gain: Arc<AtomicU32>,
    /// True when the audio thread wants the next track pre-queued for gapless
    pub(crate) gapless_ready: Arc<AtomicBool>,
    /// Track ID of the gapless-queued next track (0 = none)
    pub(crate) gapless_next_track_id: Arc<AtomicU64>,
    /// Streaming buffer progress (0.0-1.0 stored as f32 bits, 0 = not streaming)
    pub(crate) buffer_progress: Arc<AtomicU32>,
    /// Current bit-perfect mode encoded as u8 (see `bit_perfect_mode_from_u8`).
    /// 0 = Unknown (no stream active yet), 1 = Disabled (CPAL/Rodio / shared
    /// system path), 2 = DirectHardware (ALSA hw:), 3 = PluginFallback (plughw:).
    pub(crate) bit_perfect_mode: Arc<AtomicU8>,
    /// Monotonic play generation (PR #583). Bumped by `Player::begin_play` on
    /// every new play intent. Lives in the shared state so the audio thread
    /// can detect that a queued `PlayStreaming` was superseded by a newer play
    /// and stop waiting on its initial buffer instead of blocking ~60s (#591).
    pub(crate) play_generation: Arc<AtomicU64>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            is_playing: Arc::new(AtomicBool::new(false)),
            position: Arc::new(AtomicU64::new(0)),
            duration: Arc::new(AtomicU64::new(0)),
            current_track_id: Arc::new(AtomicU64::new(0)),
            dsd_direct: Arc::new(std::sync::atomic::AtomicU8::new(0)),
            has_loaded_audio: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(AtomicU32::new(0.75f32.to_bits())),
            playback_start_millis: Arc::new(AtomicU64::new(0)),
            position_at_start: Arc::new(AtomicU64::new(0)),
            current_device: Arc::new(std::sync::RwLock::new(None)),
            stream_error: Arc::new(AtomicBool::new(false)),
            stream_error_message: Arc::new(std::sync::RwLock::new(None)),
            sample_rate: Arc::new(AtomicU32::new(0)),
            bit_depth: Arc::new(AtomicU32::new(0)),
            normalization_gain: Arc::new(AtomicU32::new(0)),
            gapless_ready: Arc::new(AtomicBool::new(false)),
            gapless_next_track_id: Arc::new(AtomicU64::new(0)),
            buffer_progress: Arc::new(AtomicU32::new(0)),
            bit_perfect_mode: Arc::new(AtomicU8::new(0)),
            play_generation: Arc::new(AtomicU64::new(0)),
        }
    }
}
