//! Write Vorbis-comment tags into a FLAC file via `lofty`.

use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::tag::{ItemKey, Tag};
use std::path::Path;

use super::model::CompleteTrackMetadata;

/// Write metadata tags to a FLAC file
pub fn write_flac_tags(file_path: &str, metadata: &CompleteTrackMetadata) -> Result<(), String> {
    log::info!("Writing FLAC tags to: {}", file_path);

    let path = Path::new(file_path);
    let mut tagged_file =
        lofty::read_from_path(path).map_err(|e| format!("Failed to read FLAC file: {}", e))?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(Tag::new(tag_type));
            tagged_file.primary_tag_mut().unwrap()
        }
    };

    // Clear existing tags
    tag.clear();

    // Write standard Vorbis comments
    tag.set_title(metadata.title.clone());
    tag.set_artist(metadata.artist.clone());
    tag.set_album(metadata.album.clone());

    if let Some(album_artist) = &metadata.album_artist {
        tag.insert_text(ItemKey::AlbumArtist, album_artist.clone());
    }

    if let Some(track_number) = metadata.track_number {
        tag.set_track(track_number);
    }

    if let Some(disc_number) = metadata.disc_number {
        tag.set_disk(disc_number);
    }

    if let Some(year) = metadata.year {
        tag.set_date(lofty::tag::items::Timestamp {
            year: year as u16,
            ..Default::default()
        });
    }

    if let Some(genre) = &metadata.genre {
        tag.set_genre(genre.clone());
    }

    if let Some(isrc) = &metadata.isrc {
        tag.insert_text(ItemKey::Isrc, isrc.clone());
    }

    if let Some(label) = &metadata.label {
        tag.insert_text(ItemKey::Label, label.clone());
    }

    if let Some(copyright) = &metadata.copyright {
        tag.insert_text(ItemKey::CopyrightMessage, copyright.clone());
    }

    if let Some(composer) = &metadata.composer {
        tag.insert_text(ItemKey::Composer, composer.clone());
    }

    // Save tags
    tagged_file
        .save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("Failed to save tags: {}", e))?;

    Ok(())
}
