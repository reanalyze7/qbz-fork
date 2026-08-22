//! `IncrementalStreamingSource` — a rodio `Source` that decodes on-demand
//! from a `BufferedMediaSource`, so playback can start before the whole
//! file has downloaded. Struct, constructor, and accessors here;
//! `decode_more` and the `Source`/`Iterator` impls live in
//! `incremental_decode.rs`.

use std::collections::VecDeque;
use std::sync::Arc;

use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

use super::buffer::BufferedMediaSource;

/// A rodio Source that decodes audio packets incrementally from a BufferedMediaSource.
///
/// This allows playback to start immediately after the initial buffer is filled,
/// while the rest of the file continues downloading in the background.
///
/// The source maintains an internal queue of decoded samples and decodes more
/// packets on-demand as samples are consumed.
pub struct IncrementalStreamingSource {
    /// Sample rate of the audio
    pub(super) sample_rate: u32,
    /// Number of channels
    pub(super) channels: u16,
    /// Queue of decoded samples ready to play
    pub(super) sample_queue: VecDeque<f32>,
    /// The format reader (demuxer)
    pub(super) format: Box<dyn FormatReader>,
    /// The audio decoder
    pub(super) decoder: Box<dyn Decoder>,
    /// Track ID we're decoding
    pub(super) track_id: u32,
    /// Whether we've reached end of stream
    pub(super) finished: bool,
    /// Number of packets decoded (for stats)
    pub(super) packets_decoded: u64,
    /// True while inside a WouldBlock stall episode (playback caught up
    /// with the download). Set on the first WouldBlock after at least one
    /// decoded packet, cleared on the next successful decode — so each
    /// episode records exactly one underrun with the network throttle.
    pub(super) stalled: bool,
    /// Reference to the buffered source (for cache retrieval after playback)
    pub(super) buffered_source: Arc<BufferedMediaSource>,
}

impl IncrementalStreamingSource {
    /// Create a new incremental streaming source.
    ///
    /// This initializes the symphonia decoder and prepares for incremental decoding.
    /// The BufferedMediaSource should already have its initial buffer filled.
    ///
    /// Returns the source along with detected sample_rate and channels.
    pub fn new(buffered_source: Arc<BufferedMediaSource>) -> Result<Self, String> {
        // Create a reader from the buffered source
        let reader = buffered_source.create_reader();
        let media_source = Box::new(reader) as Box<dyn MediaSource>;
        let mss = MediaSourceStream::new(media_source, Default::default());

        let mut hint = Hint::new();
        hint.with_extension("flac"); // Most Qobuz Hi-Res is FLAC

        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let metadata_opts: MetadataOptions = Default::default();

        let probed = get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|err| format!("Symphonia probe failed for streaming: {}", err))?;

        let track = probed
            .format
            .default_track()
            .ok_or_else(|| "Symphonia: no supported audio tracks in stream".to_string())?;

        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        // Extract sample rate and channels from codec params
        let sample_rate = codec_params
            .sample_rate
            .ok_or_else(|| "No sample rate in codec params".to_string())?;
        let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

        let decoder = get_codecs()
            .make(&codec_params, &DecoderOptions::default())
            .map_err(|err| format!("Symphonia decoder init failed for streaming: {}", err))?;

        log::info!(
            "IncrementalStreamingSource initialized: {}Hz, {} channels",
            sample_rate,
            channels
        );

        Ok(Self {
            sample_rate,
            channels,
            sample_queue: VecDeque::with_capacity(sample_rate as usize * channels as usize), // ~1s buffer
            format: probed.format,
            decoder,
            track_id,
            finished: false,
            packets_decoded: 0,
            stalled: false,
            buffered_source,
        })
    }

}
