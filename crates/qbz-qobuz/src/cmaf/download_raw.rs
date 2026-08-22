use qbz_models::Quality;

use crate::client::QobuzClient;

use super::fetch::{build_cdn_client, fetch_all_segments};
use super::{CmafProgressCallback, CmafRawBundle};

/// Download a track's complete CMAF stream and return it as a raw (still
/// encrypted) bundle suitable for offline storage.
///
/// The caller is responsible for:
/// 1. Persisting `init_bytes` + `segments` to disk as bit-identical blobs
/// 2. Wrapping `content_key` with a device-bound key before storing it
/// 3. Storing `infos` (either wrapped or as plaintext — it's only a salt,
///    useless without `CMAF_SEED` + `content_key`)
///
/// At playback time, the caller feeds `init_bytes` through
/// [`qbz_cmaf::parse_init_segment`] to recover the FLAC header + segment
/// table, then decrypts each segment with the unwrapped content key.
pub async fn download_raw(
    client: &QobuzClient,
    track_id: u64,
    quality: Quality,
) -> std::result::Result<CmafRawBundle, String> {
    download_raw_with_progress(client, track_id, quality, None).await
}

/// Same as [`download_raw`] but with a progress callback fired once per
/// completed audio segment. The init segment doesn't count toward progress
/// — it's downloaded up front and is typically tiny (<1% of total bytes).
pub async fn download_raw_with_progress(
    client: &QobuzClient,
    track_id: u64,
    quality: Quality,
    on_progress: Option<CmafProgressCallback>,
) -> std::result::Result<CmafRawBundle, String> {
    let file_url = client
        .get_file_url(track_id, quality)
        .await
        .map_err(|e| format!("get_file_url failed: {}", e))?;

    let url_template = file_url
        .url_template
        .as_ref()
        .ok_or("No url_template in file/url response")?
        .clone();
    let key_str = file_url
        .key
        .as_ref()
        .ok_or("No key in file/url response")?;

    let (_session_id, infos) = client
        .ensure_cmaf_session()
        .await
        .map_err(|e| format!("ensure_cmaf_session failed: {}", e))?;

    let session_key = qbz_cmaf::derive_session_key(crate::auth::CMAF_SEED, &infos)
        .map_err(|e| format!("Session key derivation failed: {}", e))?;
    let content_key = qbz_cmaf::unwrap_content_key(&session_key, key_str)
        .map_err(|e| format!("Content key unwrap failed: {}", e))?;

    let http = build_cdn_client()?;

    // Init segment — used for FLAC header + segment table at playback
    let init_url = url_template.replace("$SEGMENT$", "0");
    log::info!("[CMAF-RAW] Fetching init for track {}", track_id);
    let init_bytes = http
        .get(&init_url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch init segment: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read init segment: {}", e))?
        .to_vec();

    // Audio segments — encrypted, stored as-is
    let segments = fetch_all_segments(
        &http,
        &url_template,
        file_url.n_segments,
        "CMAF-RAW",
        on_progress,
    )
    .await?;

    log::info!(
        "[CMAF-RAW] Track {} bundle: init={}B, {} encrypted segments, total raw size={} bytes",
        track_id,
        init_bytes.len(),
        segments.len(),
        init_bytes.len() + segments.iter().map(|s| s.len()).sum::<usize>(),
    );

    Ok(CmafRawBundle {
        init_bytes,
        segments,
        content_key,
        infos,
        format_id: file_url.format_id.unwrap_or(quality.id()),
        sampling_rate: file_url.sampling_rate,
        bit_depth: file_url.bits_depth.or(file_url.bit_depth),
        n_segments: file_url.n_segments,
    })
}
