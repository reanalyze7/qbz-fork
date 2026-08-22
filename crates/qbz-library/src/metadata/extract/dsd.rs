//! Build a LocalTrack from a DSF/DFF file via qbz-dsd. Tag-read failures
//! degrade to filename-derived metadata — a DSD file must still index.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{AudioFormat, LibraryError, LocalTrack};

use super::super::MetadataExtractor;

pub(super) fn extract_dsd(
    file_path: &Path,
    library_roots: &[PathBuf],
) -> Result<LocalTrack, LibraryError> {
    let demux = qbz_dsd::open_dsd(file_path)
        .map_err(|e| LibraryError::Metadata(format!("Failed to read DSD file: {}", e)))?;
    let info = demux.info().clone();
    drop(demux);

    let file_metadata = fs::metadata(file_path).map_err(LibraryError::Io)?;
    let file_size_bytes = file_metadata.len();
    let last_modified = file_metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let filename = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    let (fallback_artist, fallback_album) =
        MetadataExtractor::infer_artist_album(file_path, library_roots);
    let inferred_disc = MetadataExtractor::infer_disc_number(file_path);

    let tags = &info.tags;
    let album_title = MetadataExtractor::normalize_field(tags.album.as_deref())
        .or(fallback_album)
        .unwrap_or_else(|| "Unknown Album".to_string());
    let (album_group_key, album_group_title) =
        MetadataExtractor::album_group_info(file_path, Some(album_title.as_str()));

    Ok(LocalTrack {
        id: 0,
        file_path: file_path.to_string_lossy().to_string(),
        title: tags.title.clone().unwrap_or(filename),
        artist: MetadataExtractor::normalize_field(tags.artist.as_deref())
            .or(fallback_artist)
            .unwrap_or_else(|| "Unknown Artist".to_string()),
        album: album_title,
        album_artist: tags.album_artist.clone(),
        album_group_key,
        album_group_title,
        track_number: tags
            .track_number
            .or_else(|| MetadataExtractor::infer_track_number_from_filename(file_path)),
        disc_number: tags.disc_number.filter(|d| *d > 0).or(inferred_disc),
        year: tags.year.and_then(|y| u32::try_from(y).ok()),
        genre: tags.genre.clone(),
        catalog_number: None,
        duration_secs: info.duration_secs(),
        format: AudioFormat::Dsd,
        // 1-bit stream; sample_rate carries the DSD bit rate (2 822 400 =
        // DSD64) — the badge layer derives "DSD64/128/256" from it.
        bit_depth: Some(1),
        sample_rate: info.dsd_rate as f64,
        channels: info.channels as u8,
        file_size_bytes,
        cue_file_path: None,
        cue_start_secs: None,
        cue_end_secs: None,
        artwork_path: None,
        last_modified,
        indexed_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        source: None,
        qobuz_track_id: None,
        is_network_mount: false,
    })
}
