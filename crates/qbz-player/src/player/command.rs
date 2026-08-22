use super::*;

pub(crate) enum AudioCommand {
    /// Play audio data with track ID, duration, and audio specs
    Play {
        data: Vec<u8>,
        track_id: u64,
        duration_secs: u64,
        sample_rate: u32,
        channels: u16,
    },
    /// Play from streaming source (BufferedMediaSource)
    /// The download task should already be running and pushing to the source
    PlayStreaming {
        source: Arc<BufferedMediaSource>,
        track_id: u64,
        sample_rate: u32,
        channels: u16,
        duration_secs: u64,
        /// Resume offset in seconds (#315). When > 0, the audio thread
        /// waits for enough buffer to cover the offset and pre-skips
        /// decoder output up to that point before engaging audio.
        start_position_secs: u64,
        /// Total content length in bytes. Combined with `duration_secs`
        /// to estimate bytes-per-second when sizing the resume buffer.
        content_length: u64,
        /// Play generation this command belongs to (PR #583 counter,
        /// snapshotted at send time). The audio thread stops waiting for the
        /// initial buffer as soon as a newer play intent bumps the counter,
        /// instead of blocking up to 60s on a superseded download (#591).
        play_gen: u64,
    },
    /// Pause playback
    Pause,
    /// Resume playback
    Resume,
    /// Stop playback
    Stop,
    /// Set volume (0.0 - 1.0)
    SetVolume(f32),
    /// Seek to position in seconds
    Seek(u64),
    /// Reinitialize audio device (releases and re-acquires)
    ReinitDevice { device_name: Option<String> },
    /// Release the output device WITHOUT reopening it: drops the active
    /// stream (freeing an exclusive ALSA `hw:` grab + its D-Bus reservation)
    /// and un-suspends / un-forces anything QBZ parked, so PipeWire can
    /// reclaim a device QBZ was holding. User-triggered from settings.
    ReleaseDevice,
    /// Append next track to current engine for gapless playback (Rodio only)
    PlayNext {
        data: Vec<u8>,
        track_id: u64,
        sample_rate: u32,
        channels: u16,
    },
    /// Play a local DSD file via DoP (DSD over PCM) on ALSA direct (DSD plan
    /// Phase 2). The audio thread opens the demuxer + an S32 stream at the
    /// DoP carrier rate and feeds pre-packed words through the DoP engine.
    PlayDsdDop { path: std::path::PathBuf, track_id: u64 },
    /// Play a local DSD file NATIVELY (ALSA DSD_U32, DSD plan Phase 3) —
    /// requires the kernel to grant the device a DSD format (quirk table).
    PlayDsdNative { path: std::path::PathBuf, track_id: u64 },
    /// Queue the next DSD track on the ACTIVE DoP engine (gapless DSD).
    /// Ignored (with gapless_ready reset) when the engine isn't DoP or the
    /// carrier rate differs — the normal track-end advance then handles it.
    PlayNextDsdDop { path: std::path::PathBuf, track_id: u64 },
}

/// Pending gapless track data (queued for seamless transition)
pub(crate) struct GaplessPending {
    pub(crate) track_id: u64,
    pub(crate) duration_secs: u64,
    pub(crate) data: Vec<u8>,
    pub(crate) normalization_gain: Option<f32>,
}
