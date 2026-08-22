//! Track building for the "at least one tag present" branch of
//! `extract_with_roots`.

use lofty::tag::ItemKey;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::LocalTrack;

use super::super::MetadataExtractor;
use super::TrackContext;

/// Per-key fallback across ALL of the file's tags (primary first): old,
/// repeatedly-retagged collections routinely carry the album / album-artist
/// / date only in a secondary tag (ID3v1/APE/Vorbis) while the primary tag
/// lacks them. Reading just the primary tag dropped them (folder-name
/// albums, folder-level grouping, MIN(year) across mixed folders = #447;
/// album_artist landed NULL, Various Artists everywhere = #507).
pub(super) fn build_track_tagged(
    file_path: &Path,
    tagged_file: &lofty::file::TaggedFile,
    ctx: &TrackContext,
    filename: String,
    fallback_artist: Option<String>,
    fallback_album: Option<String>,
    inferred_disc: Option<u32>,
) -> LocalTrack {
    let album_tag = MetadataExtractor::string_across_tags(tagged_file, &ItemKey::AlbumTitle);
    if album_tag.is_none() {
        // Diagnostic (#447): the folder backfill fires for this file — the
        // signal when a user reports folder-name albums on "tagged" files.
        log::debug!(
            "[library] no album tag in any tag of {}; using folder-derived name",
            file_path.display()
        );
    }
    let album_title = album_tag
        .or_else(|| fallback_album.clone())
        .unwrap_or_else(|| "Unknown Album".to_string());
    let (album_group_key, album_group_title) =
        MetadataExtractor::album_group_info(file_path, Some(album_title.as_str()));

    LocalTrack {
        id: 0,
        file_path: file_path.to_string_lossy().to_string(),
        title: MetadataExtractor::string_across_tags(tagged_file, &ItemKey::TrackTitle)
            .unwrap_or(filename),
        artist: MetadataExtractor::string_across_tags(tagged_file, &ItemKey::TrackArtist)
            .or(fallback_artist)
            .unwrap_or_else(|| "Unknown Artist".to_string()),
        album: album_title,
        album_artist: MetadataExtractor::string_across_tags(tagged_file, &ItemKey::AlbumArtist),
        album_group_key,
        album_group_title,
        track_number: MetadataExtractor::track_across_tags(tagged_file)
            .or_else(|| MetadataExtractor::infer_track_number_from_filename(file_path)),
        disc_number: MetadataExtractor::disk_across_tags(tagged_file)
            .and_then(|d| if d > 0 { Some(d) } else { None })
            .or(inferred_disc),
        year: MetadataExtractor::year_across_tags(tagged_file),
        genre: MetadataExtractor::string_across_tags(tagged_file, &ItemKey::Genre),
        catalog_number: MetadataExtractor::string_across_tags(tagged_file, &ItemKey::CatalogNumber),
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
