use std::path::Path;

use crate::LibraryError;

use super::apply_fields::apply_tag_fields;
use super::types::{AlbumTagWrite, TrackTagWrite};

/// Write embedded tags to each file. Dedups by `file_path` keeping the FIRST
/// occurrence (order preserved). `on_progress(current, total)` is called
/// BEFORE each file write (1-based; total = deduped count). Partial-failure
/// unsafe by design: returns `Err` on the first failing file with prior files
/// already modified. Does NOT touch the DB or the sidecar.
pub fn write_album_tags_to_files(
    album: &AlbumTagWrite,
    tracks: &[TrackTagWrite],
    mut on_progress: impl FnMut(usize, usize),
) -> Result<(), LibraryError> {
    use lofty::config::WriteOptions;
    use lofty::prelude::*;
    use lofty::tag::Tag;

    // Dedup by file_path, first wins, original order preserved.
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<&TrackTagWrite> = tracks
        .iter()
        .filter(|t| seen.insert(t.file_path.clone()))
        .collect();
    let total = unique.len();

    for (i, track) in unique.iter().enumerate() {
        on_progress(i + 1, total);

        let path = Path::new(&track.file_path);
        if !path.is_file() {
            return Err(LibraryError::Metadata(
                "One or more audio files were not found on disk.".to_string(),
            ));
        }

        let mut tagged_file = lofty::read_from_path(path)
            .map_err(|_| LibraryError::Metadata("Failed to read audio file tags.".to_string()))?;

        let primary_type = tagged_file.primary_tag_type();
        if tagged_file.primary_tag_mut().is_none() && tagged_file.first_tag_mut().is_none() {
            tagged_file.insert_tag(Tag::new(primary_type));
        }

        {
            let tag = if let Some(tag) = tagged_file.primary_tag_mut() {
                tag
            } else if let Some(tag) = tagged_file.first_tag_mut() {
                tag
            } else {
                return Err(LibraryError::Metadata(
                    "Failed to access audio file tags.".to_string(),
                ));
            };

            apply_tag_fields(tag, album, track);
        }

        tagged_file
            .save_to_path(path, WriteOptions::default())
            .map_err(|_| {
                LibraryError::Metadata(
                    "Failed to write tags to audio files. Check that the album folder is mounted \
                     read-write and you have permissions."
                        .to_string(),
                )
            })?;
    }

    Ok(())
}
