use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{AlbumTagSidecar, LibraryDatabase, MetadataExtractor};

use super::helpers::apply_sidecar_override;

/// Process one already-canonicalized audio file: extract metadata, apply any
/// sidecar tag override, resolve artwork (embedded, falling back to a
/// per-folder cached lookup), and insert the track. `folder_root` is the
/// scanned folder's own normalized path, used for the untagged-artist root
/// clamp. `sidecar_cache` spans the whole scan; `folder_artwork_cache` is
/// reset per folder by the caller.
pub(super) fn process_audio_file(
    db: &LibraryDatabase,
    canonical: &Path,
    folder_root: &PathBuf,
    artwork_cache: &Path,
    sidecar_cache: &mut HashMap<String, Option<AlbumTagSidecar>>,
    folder_artwork_cache: &mut HashMap<PathBuf, Option<String>>,
) -> Result<(), String> {
    let mut track = MetadataExtractor::extract_with_roots(canonical, std::slice::from_ref(folder_root))
        .map_err(|e| e.to_string())?;

    apply_sidecar_override(&mut track, sidecar_cache);

    let mut artwork = MetadataExtractor::extract_artwork(canonical, artwork_cache);
    if artwork.is_none() {
        let album_hint: Option<String> = if !track.album_group_title.is_empty() {
            Some(track.album_group_title.clone())
        } else {
            Some(track.album.clone())
        };
        let folder_dir = canonical
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| canonical.to_path_buf());
        let cached = folder_artwork_cache
            .entry(folder_dir)
            .or_insert_with(|| MetadataExtractor::find_folder_artwork(canonical, album_hint.as_deref()))
            .clone();
        if let Some(folder_art) = cached {
            artwork = MetadataExtractor::cache_artwork_file(Path::new(&folder_art), artwork_cache);
        }
    }
    track.artwork_path = artwork;

    db.insert_track(&track).map_err(|e| e.to_string())?;
    Ok(())
}
