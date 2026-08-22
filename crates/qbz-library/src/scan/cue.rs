use std::path::Path;

use crate::{cue_to_tracks, CueParser, LibraryDatabase, MetadataExtractor};

use super::helpers::normalize_path;

/// Parse a CUE sheet into its virtual tracks and insert them. Mirrors Tauri's
/// `library_process_cue_file` (the per-track artwork loop already covers what
/// the deprecated `update_album_group_artwork` shortcut did, so it is omitted).
pub(super) fn process_cue_file(
    db: &LibraryDatabase,
    cue_path: &Path,
    artwork_cache: &Path,
) -> Result<(), String> {
    let mut cue = CueParser::parse(cue_path).map_err(|e| e.to_string())?;
    let audio_path = normalize_path(Path::new(&cue.audio_file));
    if !audio_path.exists() {
        return Err(format!("Audio file not found: {}", cue.audio_file));
    }
    cue.audio_file = audio_path.to_string_lossy().to_string();

    let properties = MetadataExtractor::extract_properties(&audio_path).map_err(|e| e.to_string())?;
    let format = MetadataExtractor::detect_format(&audio_path);
    let mut tracks = cue_to_tracks(&cue, properties.duration_secs, format, &properties);

    if let Some(group_key) = tracks
        .first()
        .map(|t| t.album_group_key.trim().to_string())
        .filter(|k| !k.is_empty())
    {
        let album_dir = Path::new(&group_key);
        if album_dir.is_dir() {
            if let Ok(Some(sidecar)) = crate::tag_sidecar::read_album_sidecar(album_dir) {
                for t in tracks.iter_mut() {
                    crate::tag_sidecar::apply_sidecar_to_track(t, &sidecar);
                }
            }
        }
    }

    let mut artwork = MetadataExtractor::extract_artwork(&audio_path, artwork_cache);
    if artwork.is_none() {
        if let Some(folder_art) =
            MetadataExtractor::find_folder_artwork(&audio_path, cue.title.as_deref())
        {
            artwork =
                MetadataExtractor::cache_artwork_file(Path::new(&folder_art), artwork_cache);
        }
    }
    if let Some(p) = artwork.as_ref() {
        for t in tracks.iter_mut() {
            t.artwork_path = Some(p.clone());
        }
    }

    for track in &tracks {
        db.insert_track(track).map_err(|e| e.to_string())?;
    }
    Ok(())
}
