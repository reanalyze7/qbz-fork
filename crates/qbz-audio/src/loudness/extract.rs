//! Symphonia-facing ReplayGain extraction (probe bytes or a live FormatReader).

use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

use super::source_adapter::CursorMediaSource;
use super::tags::extract_from_tags;
use super::ReplayGainData;

/// Extract ReplayGain metadata from raw audio file bytes.
///
/// Searches for ReplayGain track gain and peak values in:
/// - Vorbis comments (FLAC, Ogg): `REPLAYGAIN_TRACK_GAIN`, `REPLAYGAIN_TRACK_PEAK`
/// - ID3v2 TXXX frames: `replaygain_track_gain`
/// - Standard tag keys mapped by Symphonia
///
/// Returns `None` if no ReplayGain metadata is found.
pub fn extract_replaygain(data: &[u8]) -> Option<ReplayGainData> {
    let source = Box::new(CursorMediaSource::new(data.to_vec())) as Box<dyn MediaSource>;
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    // Help Symphonia detect isomp4/m4a
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        hint.with_extension("m4a");
    }

    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts: MetadataOptions = Default::default();

    let mut probed = match get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Loudness: probe failed for ReplayGain extraction: {}", e);
            return None;
        }
    };

    // Collect all tags from both the probe metadata and the format reader metadata
    let mut gain_db: Option<f32> = None;
    let mut peak: Option<f32> = None;

    // Check probe-level metadata (container-level tags)
    if let Some(metadata) = probed.metadata.get() {
        if let Some(rev) = metadata.current() {
            extract_from_tags(rev.tags(), &mut gain_db, &mut peak);
        }
    }

    // Check format-level metadata (in-stream tags, e.g., Vorbis comments in FLAC)
    if gain_db.is_none() {
        let fmt_metadata = probed.format.metadata();
        if let Some(rev) = fmt_metadata.current() {
            extract_from_tags(rev.tags(), &mut gain_db, &mut peak);
        }
    }

    gain_db.map(|db| {
        log::info!("Loudness: found ReplayGain: {:.2} dB, peak: {:?}", db, peak);
        ReplayGainData { gain_db: db, peak }
    })
}

/// Extract ReplayGain data from a Symphonia FormatReader (for streaming sources).
///
/// This is used when we already have a probed format reader and don't want
/// to re-probe the data.
pub fn extract_replaygain_from_reader(format: &mut dyn FormatReader) -> Option<ReplayGainData> {
    let mut gain_db: Option<f32> = None;
    let mut peak: Option<f32> = None;

    let metadata = format.metadata();
    if let Some(rev) = metadata.current() {
        extract_from_tags(rev.tags(), &mut gain_db, &mut peak);
    }

    gain_db.map(|db| {
        log::info!(
            "Loudness: found ReplayGain (streaming): {:.2} dB, peak: {:?}",
            db,
            peak
        );
        ReplayGainData { gain_db: db, peak }
    })
}
