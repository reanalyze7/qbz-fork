use super::super::*;

/// Fetch, decrypt, and push one CMAF segment's audio frames to the
/// streaming buffer. Accumulates into `cache_data` (unless `skip_cache`)
/// and `total_written` for the caller's progress logging / final cache
/// insert.
#[allow(clippy::too_many_arguments)]
pub(super) async fn fetch_and_push_segment(
    client: &reqwest::Client,
    url_template: &str,
    seg_idx: u8,
    content_key: [u8; 16],
    writer: &BufferWriter,
    skip_cache: bool,
    cache_data: &mut Vec<u8>,
    total_written: &mut u64,
) -> Result<(), String> {
    let seg_url = url_template.replace("$SEGMENT$", &seg_idx.to_string());
    let seg_data = client
        .get(&seg_url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("CMAF segment {} fetch: {}", seg_idx, e))?
        .bytes()
        .await
        .map_err(|e| format!("CMAF segment {} read: {}", seg_idx, e))?;

    let crypto = qbz_cmaf::parse_segment_crypto(&seg_data)
        .map_err(|e| format!("CMAF segment {} parse: {}", seg_idx, e))?;

    let mut data_pos = crypto.data_offset;
    for entry in &crypto.entries {
        let frame_end = data_pos + entry.size as usize;
        if frame_end > seg_data.len() {
            let _ = writer.error(format!("CMAF segment {} frame overflow", seg_idx));
            return Err(format!("CMAF segment {} frame overflow", seg_idx));
        }
        let mut frame = seg_data[data_pos..frame_end].to_vec();
        if entry.flags != 0 {
            qbz_cmaf::decrypt_frame(&content_key, &entry.iv, &mut frame);
        }
        if let Err(e) = writer.push_chunk(&frame) {
            let msg = format!("Failed to push frame: {e}");
            let _ = writer.error(msg.clone());
            return Err(msg);
        }
        if !skip_cache {
            cache_data.extend_from_slice(&frame);
        }
        *total_written += frame.len() as u64;
        data_pos = frame_end;
    }

    // Trailing unencrypted data after all frame entries.
    if data_pos < crypto.mdat_end && crypto.mdat_end <= seg_data.len() {
        let trailing = &seg_data[data_pos..crypto.mdat_end];
        if let Err(e) = writer.push_chunk(trailing) {
            let msg = format!("Failed to push trailing data: {e}");
            let _ = writer.error(msg.clone());
            return Err(msg);
        }
        if !skip_cache {
            cache_data.extend_from_slice(trailing);
        }
        *total_written += trailing.len() as u64;
    }

    Ok(())
}
