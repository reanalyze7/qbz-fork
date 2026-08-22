use qbz_models::{Quality, StreamQualityInfo};

use crate::client::QobuzClient;

use super::decrypt::decrypt_segments_into;
use super::fetch::{build_cdn_client, fetch_all_segments};
use super::setup::setup_streaming;
use super::CmafProgressCallback;

/// Download a track's complete CMAF stream and return decrypted FLAC bytes.
///
/// Used by the playback path for in-memory cache writes. Segments are
/// fetched concurrently with a semaphore cap, decrypted, and concatenated.
pub async fn download_full(
    client: &QobuzClient,
    track_id: u64,
    quality: Quality,
) -> std::result::Result<Vec<u8>, String> {
    download_full_with_progress(client, track_id, quality, None).await
}

/// Same as [`download_full`] but with a progress callback fired once per
/// completed segment.
pub async fn download_full_with_progress(
    client: &QobuzClient,
    track_id: u64,
    quality: Quality,
    on_progress: Option<CmafProgressCallback>,
) -> std::result::Result<Vec<u8>, String> {
    download_full_with_quality_progress(client, track_id, quality, on_progress)
        .await
        .map(|(bytes, _quality)| bytes)
}

/// Like [`download_full`] but also returns the quality actually resolved from
/// the CMAF init segment (`format_id` / `sampling_rate` / `bit_depth`). Used
/// by the external-stream (Cast / DLNA) path, which must surface the real
/// delivered quality. The CMAF path always yields decrypted FLAC, so the
/// caller's content type is `audio/flac`.
pub async fn download_full_with_quality(
    client: &QobuzClient,
    track_id: u64,
    quality: Quality,
) -> std::result::Result<(Vec<u8>, StreamQualityInfo), String> {
    download_full_with_quality_progress(client, track_id, quality, None).await
}

/// [`download_full_with_quality`] + a per-segment progress callback.
pub async fn download_full_with_quality_progress(
    client: &QobuzClient,
    track_id: u64,
    quality: Quality,
    on_progress: Option<CmafProgressCallback>,
) -> std::result::Result<(Vec<u8>, StreamQualityInfo), String> {
    let setup = setup_streaming(client, track_id, quality).await?;
    let http = build_cdn_client()?;

    let total_size: usize = setup.flac_header.len()
        + setup
            .segment_table
            .iter()
            .map(|s| s.byte_len as usize)
            .sum::<usize>();

    let segments = fetch_all_segments(
        &http,
        &setup.url_template,
        setup.n_segments,
        "CMAF-FULL",
        on_progress,
    )
    .await?;

    let mut output = Vec::with_capacity(total_size);
    output.extend_from_slice(&setup.flac_header);
    decrypt_segments_into(&segments, &setup.content_key, &mut output)?;

    log::info!(
        "[CMAF-FULL] Track {} complete: {:.2} MB FLAC, expected {:.2} MB",
        track_id,
        output.len() as f64 / (1024.0 * 1024.0),
        total_size as f64 / (1024.0 * 1024.0),
    );

    // `from_raw` normalizes the rate unit (kHz vs Hz) defensively.
    let quality_info = StreamQualityInfo::from_raw(
        setup.format_id,
        setup.sampling_rate.map(|v| v as f64),
        setup.bit_depth,
    );
    Ok((output, quality_info))
}
