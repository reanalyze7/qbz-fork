//! [`DsdPcmConverter`]: whole-file streaming demuxer → per-channel dsd2pcm →
//! half-band chain → interleaved f32 blocks at [`super::OUTPUT_RATE`].

use super::downmix::fold_to_stereo;
use super::halfband::HalfBand;
use crate::demux::{DsdDemuxer, DsdError};
use crate::dsd2pcm::Dsd2Pcm;

use super::OUTPUT_RATE;

/// DSD bytes requested from the demuxer per conversion block, per channel.
/// 64 KiB ≈ 0.19 s of DSD64 — small enough to stream, big enough to be cheap.
const BLOCK_BYTES_PER_CH: usize = 64 * 1024;

pub struct DsdPcmConverter {
    demux: Box<dyn DsdDemuxer>,
    channels: usize,
    lsb_first: bool,
    dsd2pcm: Vec<Dsd2Pcm>,
    stages: Vec<Vec<HalfBand>>, // stages[stage][channel]
    gain: f32,
    total_frames: u64,
    frames_emitted: u64,
    finished: bool,
}

impl DsdPcmConverter {
    pub fn new(demux: Box<dyn DsdDemuxer>, gain_db: f32) -> Result<Self, DsdError> {
        let info = demux.info().clone();
        let ratio = info.dsd_rate / OUTPUT_RATE; // 32 / 64 / 128 / 256
        if info.dsd_rate % OUTPUT_RATE != 0 || !(ratio / 8).is_power_of_two() || ratio < 16 {
            return Err(DsdError::UnsupportedRate(info.dsd_rate));
        }
        let n_stages = (ratio / 8).trailing_zeros() as usize; // 2..=5
        let channels = info.channels as usize;
        let stages = (0..n_stages)
            .map(|_| (0..channels).map(|_| HalfBand::new()).collect())
            .collect();
        Ok(Self {
            demux,
            channels,
            lsb_first: info.lsb_first,
            dsd2pcm: (0..channels).map(|_| Dsd2Pcm::new()).collect(),
            stages,
            gain: 10f32.powf(gain_db / 20.0),
            total_frames: info.sample_count / ratio as u64,
            frames_emitted: 0,
            finished: false,
        })
    }

    pub fn output_rate(&self) -> u32 {
        OUTPUT_RATE
    }
    /// PCM output channels: ALWAYS stereo. Mono sources are duplicated;
    /// multichannel (up to 5.1) sources are downmixed (ITU-R BS.775
    /// coefficients, LFE discarded, normalized against clipping).
    pub fn channels(&self) -> u16 {
        2
    }
    /// Exact number of interleaved PCM frames this converter will emit in
    /// total (used to size the WAV header up front).
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    /// Produce the next interleaved f32 block, `None` when done. The overall
    /// emitted frame count is exactly [`Self::total_frames`]: the final block
    /// is silence-padded or truncated as needed so the container size always
    /// matches the header.
    pub fn next_block(&mut self) -> Result<Option<Vec<f32>>, DsdError> {
        if self.finished {
            return Ok(None);
        }
        let mut planar: Vec<Vec<u8>> = (0..self.channels).map(|_| Vec::new()).collect();
        let got = self.demux.read_planar(&mut planar, BLOCK_BYTES_PER_CH)?;
        if got == 0 {
            // EOF: pad with silence if the filter latency left us short.
            self.finished = true;
            let missing = self.total_frames - self.frames_emitted;
            if missing == 0 {
                return Ok(None);
            }
            self.frames_emitted = self.total_frames;
            return Ok(Some(vec![0.0; (missing as usize) * 2]));
        }

        let mut per_ch: Vec<Vec<f32>> = Vec::with_capacity(self.channels);
        for ch in 0..self.channels {
            let mut buf = Vec::new();
            self.dsd2pcm[ch].translate(&planar[ch], self.lsb_first, &mut buf);
            for stage in self.stages.iter_mut() {
                let mut down = Vec::with_capacity(buf.len() / 2 + 1);
                stage[ch].process(&buf, &mut down);
                buf = down;
            }
            per_ch.push(buf);
        }

        let frames = per_ch.iter().map(|c| c.len()).min().unwrap_or(0) as u64;
        let frames = frames.min(self.total_frames - self.frames_emitted) as usize;
        if frames == 0 {
            // Nothing usable this round (filters still priming) — recurse to
            // pull more input; bounded by file size.
            return self.next_block();
        }
        let mut out = Vec::with_capacity(frames * 2);
        for f in 0..frames {
            let (l, r) = fold_to_stereo(self.channels, &per_ch, f);
            out.push(l * self.gain);
            out.push(r * self.gain);
        }
        self.frames_emitted += frames as u64;
        if self.frames_emitted >= self.total_frames {
            self.finished = true;
        }
        Ok(Some(out))
    }
}
