use super::*;

pub(crate) fn is_isomp4(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }

    &data[4..8] == b"ftyp"
}

/// Extract audio metadata (sample rate, channels) without full decode.
/// This is much faster than decode_with_symphonia as it only reads headers.
/// Audio metadata extracted from file headers
#[allow(dead_code)]
pub(crate) struct AudioMetadata {
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
    pub(crate) bit_depth: Option<u32>,
}

#[allow(dead_code)]
pub(crate) fn extract_audio_metadata(data: &[u8]) -> Result<(u32, u16), String> {
    let meta = extract_audio_metadata_full(data)?;
    Ok((meta.sample_rate, meta.channels))
}

pub(crate) fn extract_audio_metadata_full(data: &[u8]) -> Result<AudioMetadata, String> {
    // For non-isomp4 files (FLAC, etc.), try symphonia directly to get all metadata
    // Symphonia gives us bits_per_sample which rodio doesn't expose

    // Use symphonia probe for codec params (no decode needed)
    let source = Box::new(CursorMediaSource::new(data.to_vec())) as Box<dyn MediaSource>;
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    if is_isomp4(data) {
        hint.with_extension("m4a");
    }

    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts: MetadataOptions = Default::default();
    let probed = get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|err| format!("Symphonia probe failed: {}", err))?;

    let track = probed
        .format
        .default_track()
        .ok_or_else(|| "Symphonia: no supported audio tracks".to_string())?;

    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| "No sample rate in codec params".to_string())?;

    // ALAC and some other formats don't include channel info in initial codec params
    // Default to stereo (2 channels) which is the most common case
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(2);

    // Get bits per sample for bit depth
    let bit_depth = track.codec_params.bits_per_sample;

    Ok(AudioMetadata {
        sample_rate,
        channels,
        bit_depth,
    })
}

/// True when a cached FLAC is a lower quality than `requested`, so the
/// cache entry should be bypassed and the track re-fetched. Ported from
/// the Tauri `cached_quality_below_requested` helper: an unparseable
/// buffer is assumed compatible.
pub(crate) fn cached_quality_below_requested(data: &[u8], requested: Quality) -> bool {
    let meta = match extract_audio_metadata_full(data) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let sample_rate = meta.sample_rate;
    let bit_depth = meta.bit_depth.unwrap_or(16);
    match requested {
        // Hi-Res+: expect 24-bit AND > 96 kHz.
        Quality::UltraHiRes => bit_depth < 24 || sample_rate <= 96000,
        // Hi-Res: expect 24-bit.
        Quality::HiRes => bit_depth < 24,
        // Lossless / Mp3: any FLAC satisfies the request.
        _ => false,
    }
}

