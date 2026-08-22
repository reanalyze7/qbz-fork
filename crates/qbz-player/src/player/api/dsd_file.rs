use super::*;

mod dsd_file_direct;

impl Player {
    /// Play a local DSD file (.dsf/.dff) by converting it on the fly to
    /// 176.4 kHz / 24-bit PCM (qbz-dsd, Phase 1 of the DSD plan — see
    /// qbz-nix-docs/dsd-support/). The converted stream rides the existing
    /// `play_streaming` path as an ordinary finite WAV: a background thread
    /// demuxes + decimates and pushes into the BufferWriter, so the whole
    /// PCM pipeline (engines, bit-perfect ALSA at 176.4 kHz, volume,
    /// normalization, seek-in-buffer) behaves exactly as for any hi-res
    /// track. DST-compressed DFF and >2ch files are rejected with a
    /// readable error before anything starts.
    pub fn play_dsd_file(&self, path: std::path::PathBuf, track_id: u64) -> Result<(), String> {
        let _gen = self.begin_play();
        let demux = qbz_dsd::open_dsd(&path).map_err(|e| e.to_string())?;
        let dsd_rate = demux.info().dsd_rate;

        // DoP resolution (Phase 2): user opt-in + ALSA direct backend +
        // stereo + carrier rate supported by the device. Anything else falls
        // through to the universal DSD->PCM conversion below.
        #[cfg(target_os = "linux")]
        {
            let info = demux.info().clone();
            if let Some(result) = self.try_direct_dsd(&info, &path, track_id) {
                drop(demux);
                return result;
            }
        }

        let mut conv = qbz_dsd::DsdPcmConverter::new(demux, qbz_dsd::DEFAULT_GAIN_DB)
            .map_err(|e| e.to_string())?;
        let channels = conv.channels();
        let rate = conv.output_rate();
        let total_frames = conv.total_frames();
        let duration_secs = total_frames / rate as u64;
        let content_length = qbz_dsd::wav_total_size(total_frames, channels);
        log::info!(
            "Player: DSD track {} — {} ({} Hz) → PCM {} Hz/24-bit, {}s, {} bytes WAV",
            track_id,
            qbz_dsd::dsd_label(dsd_rate),
            dsd_rate,
            rate,
            duration_secs,
            content_length
        );
        self.state.set_stream_quality(rate, 24);

        let writer = self.apply_play_streaming(
            track_id,
            rate,
            channels,
            content_length,
            3,
            duration_secs,
            0,
        )?;

        std::thread::spawn(move || {
            if writer
                .push_chunk(&qbz_dsd::wav_header(total_frames, channels, rate))
                .is_err()
            {
                return;
            }
            let mut pcm = Vec::new();
            loop {
                match conv.next_block() {
                    Ok(Some(frames)) => {
                        pcm.clear();
                        qbz_dsd::frames_to_pcm24(&frames, &mut pcm);
                        if writer.push_chunk(&pcm).is_err() {
                            // Reader gone (track changed/stopped) — just stop.
                            return;
                        }
                    }
                    Ok(None) => {
                        let _ = writer.complete();
                        return;
                    }
                    Err(e) => {
                        log::error!("Player: DSD conversion failed mid-track: {}", e);
                        let _ = writer.error(format!("DSD conversion failed: {}", e));
                        return;
                    }
                }
            }
        });
        Ok(())
    }
}
