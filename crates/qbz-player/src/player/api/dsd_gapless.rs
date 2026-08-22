use super::*;

impl Player {
    /// Whole-file DSD→PCM conversion into an in-memory WAV, for the gapless
    /// prefetch path: the result feeds `play_next` like any cached track, so
    /// consecutive converted-DSD tracks hand off seamlessly. CPU-bound
    /// (~10-30x realtime) — call from a blocking context.
    pub fn prepare_dsd_gapless_wav(path: &std::path::Path) -> Result<Vec<u8>, String> {
        let demux = qbz_dsd::open_dsd(path).map_err(|e| e.to_string())?;
        let mut conv = qbz_dsd::DsdPcmConverter::new(demux, qbz_dsd::DEFAULT_GAIN_DB)
            .map_err(|e| e.to_string())?;
        let channels = conv.channels();
        let rate = conv.output_rate();
        let total = conv.total_frames();
        let mut out = qbz_dsd::wav_header(total, channels, rate);
        out.reserve(total as usize * channels as usize * 3);
        while let Some(frames) = conv.next_block().map_err(|e| e.to_string())? {
            qbz_dsd::frames_to_pcm24(&frames, &mut out);
        }
        Ok(out)
    }

    /// Queue the next DSD track for a gapless transition: appends to the DoP
    /// engine when one is active (seamless native DSD), otherwise converts to
    /// an in-memory WAV and rides the normal `play_next` gapless path.
    pub fn play_next_dsd(&self, path: std::path::PathBuf, track_id: u64) -> Result<(), String> {
        if self.state.is_dsd_direct() {
            return self
                .tx
                .send(AudioCommand::PlayNextDsdDop { path, track_id })
                .map_err(|e| format!("Failed to send DoP gapless command: {}", e));
        }
        let wav = Self::prepare_dsd_gapless_wav(&path)?;
        self.play_next(wav, track_id)
    }

    /// True while a DoP stream is active (volume fixed, seek unsupported).
    pub fn is_dsd_direct_active(&self) -> bool {
        self.state.is_dsd_direct()
    }
}
