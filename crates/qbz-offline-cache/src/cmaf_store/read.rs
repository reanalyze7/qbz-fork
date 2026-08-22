//! Read + decrypt path: loading a persisted CMAF bundle back for playback.

use std::fs;

use super::layout::{BundleLayout, BundleManifest};

/// A freshly-loaded bundle ready to be decrypted + played. Owned buffers
/// so the caller can feed the player without holding any file locks.
pub struct LoadedBundle {
    pub init_bytes: Vec<u8>,
    pub segments: Vec<Vec<u8>>,
    pub manifest: BundleManifest,
}

impl LoadedBundle {
    /// Decrypt the bundle into a complete, playable FLAC byte stream.
    ///
    /// The layout is `flac_header || decrypted_frames` — identical to
    /// what the streaming playback path produces, so the player can
    /// consume it via `play_data` exactly like a cached plain FLAC.
    pub fn decrypt_to_flac(&self, content_key: &[u8; 16]) -> Result<Vec<u8>, String> {
        let init_info = qbz_cmaf::parse_init_segment(&self.init_bytes)
            .map_err(|e| format!("Failed to parse init segment: {}", e))?;

        let total = init_info.flac_header.len()
            + init_info
                .segment_table
                .iter()
                .map(|s| s.byte_len as usize)
                .sum::<usize>();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&init_info.flac_header);
        qbz_qobuz::cmaf::decrypt_segments_into(&self.segments, content_key, &mut out)?;
        Ok(out)
    }
}

/// Load a bundle back from disk for playback. Returns the init bytes and
/// the per-segment slices of `segments.bin` in order.
pub fn read_bundle(layout: &BundleLayout) -> Result<LoadedBundle, String> {
    let init_bytes = fs::read(&layout.init_path)
        .map_err(|e| format!("Failed to read init {:?}: {}", layout.init_path, e))?;
    let segments_blob = fs::read(&layout.segments_path)
        .map_err(|e| format!("Failed to read segments {:?}: {}", layout.segments_path, e))?;
    let manifest_bytes = fs::read(&layout.manifest_path)
        .map_err(|e| format!("Failed to read manifest {:?}: {}", layout.manifest_path, e))?;
    let manifest: BundleManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    let n = manifest.n_segments as usize;
    if manifest.segment_offsets.len() != n + 1 {
        return Err(format!(
            "Manifest offsets length {} doesn't match n_segments {}+1",
            manifest.segment_offsets.len(),
            n
        ));
    }
    let mut segments: Vec<Vec<u8>> = Vec::with_capacity(n);
    for i in 0..n {
        let start = manifest.segment_offsets[i] as usize;
        let end = manifest.segment_offsets[i + 1] as usize;
        if end > segments_blob.len() {
            return Err(format!(
                "Segment {} offset {}..{} past blob size {}",
                i + 1,
                start,
                end,
                segments_blob.len()
            ));
        }
        segments.push(segments_blob[start..end].to_vec());
    }

    Ok(LoadedBundle {
        init_bytes,
        segments,
        manifest,
    })
}

/// Remove a bundle from disk (called by eviction / re-download).
pub fn remove_bundle(layout: &BundleLayout) {
    if layout.track_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&layout.track_dir) {
            log::warn!(
                "[OfflineCache/CMAF] Failed to remove bundle dir {:?}: {}",
                layout.track_dir,
                e
            );
        }
    }
}
