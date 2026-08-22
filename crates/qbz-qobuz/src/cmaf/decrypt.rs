/// Decrypt a sequence of encrypted CMAF segments in order and append the
/// decrypted frames to `output`.
///
/// This is the common decryption logic shared between the full-download
/// path (decrypt-then-return) and the offline playback path (decrypt-from-
/// disk-then-feed-player).
///
/// Hot-path note: the previous implementation allocated a `Vec<u8>` per
/// frame, copied the encrypted bytes into it, decrypted in place, then
/// copied again into `output` via `extend_from_slice`. For a HiRes FLAC
/// this is tens of thousands of small heap allocations + double copies
/// per track. Now we extend `output` with the encrypted bytes directly
/// and decrypt the just-appended slice in place — one copy instead of
/// three, zero per-frame allocations. Combined with AES-NI codegen
/// (enabled via `target-cpu=x86-64-v3` in `.cargo/config.toml`) this
/// is the difference between a 20-second offline-cache gap on track
/// transitions and a sub-second one.
pub fn decrypt_segments_into(
    segments: &[Vec<u8>],
    content_key: &[u8; 16],
    output: &mut Vec<u8>,
) -> std::result::Result<(), String> {
    for (seg_idx, seg_data) in segments.iter().enumerate() {
        // seg_idx is 0-based here but the original segment number is idx+1
        let log_idx = seg_idx + 1;
        let crypto = qbz_cmaf::parse_segment_crypto(seg_data)
            .map_err(|e| format!("CMAF seg {} parse: {}", log_idx, e))?;

        let mut data_pos = crypto.data_offset;
        for entry in &crypto.entries {
            let frame_end = data_pos + entry.size as usize;
            if frame_end > seg_data.len() {
                return Err(format!("CMAF seg {} frame overflow", log_idx));
            }
            let output_start = output.len();
            output.extend_from_slice(&seg_data[data_pos..frame_end]);
            if entry.flags != 0 {
                qbz_cmaf::decrypt_frame(content_key, &entry.iv, &mut output[output_start..]);
            }
            data_pos = frame_end;
        }
        if data_pos < crypto.mdat_end && crypto.mdat_end <= seg_data.len() {
            output.extend_from_slice(&seg_data[data_pos..crypto.mdat_end]);
        }
    }
    Ok(())
}
