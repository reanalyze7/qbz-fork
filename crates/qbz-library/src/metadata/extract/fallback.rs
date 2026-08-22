//! Track building for the "no tag found" branch of `extract_with_roots`:
//! everything falls back to filename/folder-derived defaults.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::LocalTrack;

use super::super::MetadataExtractor;
use super::TrackContext;

pub(super) fn build_track_fallback(
    file_path: &Path,
    ctx: &TrackContext,
    filename: String,
    fallback_artist: Option<String>,
    fallback_album: Option<String>,
    inferred_disc: Option<u32>,
) -> LocalTrack {
    let album_title = fallback_album.unwrap_or_else(|| "Unknown Album".to_string());
    let (album_group_key, album_group_title) =
        MetadataExtractor::album_group_info(file_path, Some(album_title.as_str()));

    LocalTrack {
        id: 0,
        file_path: file_path.to_string_lossy().to_string(),
        title: filename,
        artist: fallback_artist.unwrap_or_else(|| "Unknown Artist".to_string()),
        album: album_title,
        album_artist: None,
        album_group_key,
        album_group_title,
        track_number: MetadataExtractor::infer_track_number_from_filename(file_path),
        disc_number: inferred_disc,
        year: None,
        genre: None,
        catalog_number: None,
        duration_secs: ctx.duration_secs,
        format: ctx.format.clone(),
        bit_depth: ctx.bit_depth,
        sample_rate: ctx.sample_rate,
        channels: ctx.channels,
        file_size_bytes: ctx.file_size_bytes,
        cue_file_path: None,
        cue_start_secs: None,
        cue_end_secs: None,
        artwork_path: None,
        last_modified: ctx.last_modified,
        indexed_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        source: None,
        qobuz_track_id: None,
        is_network_mount: false,
    }
}
