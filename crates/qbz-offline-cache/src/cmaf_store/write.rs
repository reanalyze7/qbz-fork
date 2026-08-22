//! Write path: persisting a freshly-downloaded CMAF bundle to disk.

use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use qbz_qobuz::cmaf::CmafRawBundle;

use super::layout::{BundleLayout, BundleManifest};

/// Writes a freshly-downloaded [`CmafRawBundle`] to disk under the track
/// directory, returning the layout + total size of the persisted bytes.
///
/// Note: this does NOT write any key material. The caller is responsible
/// for wrapping `bundle.content_key` / `bundle.infos` via `qbz-secrets`
/// and persisting those blobs to the SQLite row.
pub fn persist_bundle(
    offline_root: &Path,
    track_id: u64,
    bundle: &CmafRawBundle,
) -> Result<(BundleLayout, u64), String> {
    let layout = BundleLayout::new(offline_root, track_id);
    fs::create_dir_all(&layout.track_dir)
        .map_err(|e| format!("Failed to create bundle dir {:?}: {}", layout.track_dir, e))?;

    // Init segment (unencrypted, small)
    write_atomic(&layout.init_path, &bundle.init_bytes)
        .map_err(|e| format!("Failed to write init: {}", e))?;

    // Audio segments: concatenated into a single file, with offsets tracked
    // for the manifest so playback can slice them back apart.
    let segments_tmp = layout.segments_path.with_extension("tmp");
    let file = fs::File::create(&segments_tmp)
        .map_err(|e| format!("Failed to create segments file: {}", e))?;
    let mut writer = BufWriter::new(file);
    let mut offsets: Vec<u64> = Vec::with_capacity(bundle.segments.len() + 1);
    let mut cursor: u64 = 0;
    offsets.push(cursor);
    for seg in &bundle.segments {
        writer
            .write_all(seg)
            .map_err(|e| format!("Failed to write segment: {}", e))?;
        cursor += seg.len() as u64;
        offsets.push(cursor);
    }
    writer
        .flush()
        .map_err(|e| format!("Failed to flush segments: {}", e))?;
    let file = writer
        .into_inner()
        .map_err(|e| format!("Failed to finalize writer: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to fsync segments: {}", e))?;
    drop(file);
    fs::rename(&segments_tmp, &layout.segments_path)
        .map_err(|e| format!("Failed to rename segments: {}", e))?;

    let manifest = BundleManifest {
        version: 1,
        track_id,
        format_id: bundle.format_id,
        n_segments: bundle.n_segments as u32,
        segment_offsets: offsets.clone(),
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    write_atomic(&layout.manifest_path, &manifest_json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    let total_bytes = bundle.init_bytes.len() as u64 + cursor + manifest_json.len() as u64;
    log::info!(
        "[OfflineCache/CMAF] Persisted bundle for track {}: init={}B, segments={}B ({} files), manifest={}B, total={:.2} MB",
        track_id,
        bundle.init_bytes.len(),
        cursor,
        bundle.segments.len(),
        manifest_json.len(),
        total_bytes as f64 / (1024.0 * 1024.0),
    );
    Ok((layout, total_bytes))
}

fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}
