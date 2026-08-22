//! `InMemorySource` — a Symphonia-backed decoder with native seek for
//! fully-in-memory audio bytes. Struct, constructor, and `seek_to` here;
//! `decode_more` and the `Source`/`Iterator` impls live in
//! `in_memory_decode.rs`. Its `Cursor`-backed `MediaSource`
//! (`InMemoryMediaSource`) lives in `in_memory_media_source.rs`.

use std::collections::VecDeque;
use std::time::Duration;

use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

use super::in_memory_media_source::InMemoryMediaSource;

/// Symphonia-backed decoder for fully-in-memory audio bytes, with native
/// seek support.
///
/// Exists because rodio's `skip_duration` decodes every sample from the
/// start of the track when seeking, which on FLAC Hi-Res costs several
/// seconds of CPU for long jumps and stalls the audio thread. This
/// source uses `FormatReader::seek(Accurate, SeekTo::Time)` — FLAC seek
/// table, MP3 Xing/VBRI TOC — to jump straight to the target sample, so
/// the post-seek decode window is ~O(seek point density) instead of
/// O(position).
///
/// Non-Symphonia formats (notably rodio's native MP4/AAC path) aren't
/// supported here; callers must fall back to `decode_with_fallback` +
/// `skip_duration` when `new` returns `Err`.
pub struct InMemorySource {
    pub(super) sample_rate: u32,
    pub(super) channels: u16,
    pub(super) sample_queue: VecDeque<f32>,
    pub(super) format: Box<dyn FormatReader>,
    pub(super) decoder: Box<dyn Decoder>,
    pub(super) track_id: u32,
    pub(super) finished: bool,
}

impl InMemorySource {
    pub fn new(data: Vec<u8>) -> Result<Self, String> {
        let source = Box::new(InMemoryMediaSource::new(data)) as Box<dyn MediaSource>;
        let mss = MediaSourceStream::new(source, Default::default());

        let hint = Hint::new();

        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let metadata_opts: MetadataOptions = Default::default();

        let probed = get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|err| format!("Symphonia probe failed for in-memory source: {}", err))?;

        let track = probed
            .format
            .default_track()
            .ok_or_else(|| "Symphonia: no supported audio tracks".to_string())?;

        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        let sample_rate = codec_params
            .sample_rate
            .ok_or_else(|| "No sample rate in codec params".to_string())?;
        let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

        let decoder = get_codecs()
            .make(&codec_params, &DecoderOptions::default())
            .map_err(|err| format!("Symphonia decoder init failed: {}", err))?;

        Ok(Self {
            sample_rate,
            channels,
            sample_queue: VecDeque::with_capacity(sample_rate as usize * channels as usize),
            format: probed.format,
            decoder,
            track_id,
            finished: false,
        })
    }

    pub fn seek_to(&mut self, time: Duration) -> Result<(), String> {
        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: time.into(),
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| format!("Symphonia in-memory seek failed: {}", e))?;
        self.decoder.reset();
        self.sample_queue.clear();
        self.finished = false;
        Ok(())
    }
}
