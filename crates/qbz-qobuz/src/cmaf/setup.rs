use qbz_models::Quality;

use crate::client::QobuzClient;

use super::fetch::{build_cdn_client, fetch_bytes_with_retry};
use super::CmafStreamingInfo;

/// Prepare CMAF streaming: fetch init segment only, derive keys, return info.
/// Does NOT download audio segments -- the caller streams those in background.
pub async fn setup_streaming(
    client: &QobuzClient,
    track_id: u64,
    quality: Quality,
) -> std::result::Result<CmafStreamingInfo, String> {
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

    // Fetch only the init segment (s=0) -- typically small, <500ms
    let http = build_cdn_client()?;
    let init_url = url_template.replace("$SEGMENT$", "0");
    let init_start = std::time::Instant::now();

    log::info!("[CMAF] Fetching init segment for track {}", track_id);
    let init_data = fetch_bytes_with_retry(&http, &init_url, "CMAF init")
        .await
        .map_err(|e| format!("Failed to fetch init segment: {}", e))?;

    let init_fetch_ms = init_start.elapsed().as_millis() as u64;

    let init_info = qbz_cmaf::parse_init_segment(&init_data)
        .map_err(|e| format!("Failed to parse init segment: {}", e))?;

    log::info!(
        "[CMAF] Init for track {}: FLAC header {}B, segment_table={} entries, API n_segments={}, fetched in {}ms",
        track_id,
        init_info.flac_header.len(),
        init_info.segment_table.len(),
        file_url.n_segments,
        init_fetch_ms
    );
    if init_info.segment_table.len() != file_url.n_segments as usize {
        log::warn!(
            "[CMAF] MISMATCH for track {}: segment_table has {} entries but API says n_segments={}",
            track_id,
            init_info.segment_table.len(),
            file_url.n_segments
        );
    }

    let format_id = file_url.format_id.unwrap_or(quality.id());

    Ok(CmafStreamingInfo {
        url_template,
        n_segments: file_url.n_segments,
        content_key,
        flac_header: init_info.flac_header,
        segment_table: init_info.segment_table,
        format_id,
        sampling_rate: file_url.sampling_rate,
        bit_depth: file_url.bits_depth.or(file_url.bit_depth),
        init_fetch_ms,
    })
}
