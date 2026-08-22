//! On-disk naming/layout for v2 CMAF bundles — pure path logic, no I/O.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(super) const SUBDIR: &str = "tracks-cmaf";
pub(super) const INIT_FILENAME: &str = "init.mp4";
pub(super) const SEGMENTS_FILENAME: &str = "segments.bin";
pub(super) const MANIFEST_FILENAME: &str = "manifest.json";

/// Lightweight sidecar manifest saved next to the bundle. If the SQLite
/// index is ever lost or an integrity check fails, this is enough to
/// reconstruct the per-segment slicing so the decrypt path can still
/// iterate the concatenated `segments.bin`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub version: u8,
    pub track_id: u64,
    pub format_id: u32,
    pub n_segments: u32,
    /// Offset in bytes of each encrypted segment inside `segments.bin`.
    /// `segment_offsets[i]` = start of segment `i+1`; `segment_offsets[n]`
    /// is the total size. Length = `n_segments + 1`.
    pub segment_offsets: Vec<u64>,
}

/// Where the v2 bundle for a given track id lives on disk. Callers use
/// this to build DB rows and to locate existing bundles at playback.
#[derive(Debug, Clone)]
pub struct BundleLayout {
    pub track_dir: PathBuf,
    pub init_path: PathBuf,
    pub segments_path: PathBuf,
    pub manifest_path: PathBuf,
}

impl BundleLayout {
    pub fn new(offline_root: &Path, track_id: u64) -> Self {
        let track_dir = offline_root.join(SUBDIR).join(track_id.to_string());
        Self {
            init_path: track_dir.join(INIT_FILENAME),
            segments_path: track_dir.join(SEGMENTS_FILENAME),
            manifest_path: track_dir.join(MANIFEST_FILENAME),
            track_dir,
        }
    }
}
