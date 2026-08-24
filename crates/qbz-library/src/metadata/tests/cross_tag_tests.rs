// ---- Cross-tag fallback (#447/#507) ----------------------------------
//
// Old, repeatedly-retagged collections routinely carry the album /
// album-artist / date only in a SECONDARY tag (ID3v1/APE/Vorbis) while
// the file-type's primary tag lacks them. These tests build in-memory
// tags (no audio files needed) and exercise the pure fallback cores
// used by `extract`.

use crate::MetadataExtractor;
use lofty::tag::ItemKey;

#[test]
fn cross_tag_album_read_falls_back_to_secondary_tag() {
    // #447: the primary tag (ID3v2) has no album; it lives only in ID3v1.
    let mut primary = lofty::tag::Tag::new(lofty::tag::TagType::Id3v2);
    primary.insert_text(ItemKey::TrackTitle, "Song".to_string());
    let mut secondary = lofty::tag::Tag::new(lofty::tag::TagType::Id3v1);
    secondary.insert_text(ItemKey::AlbumTitle, "ALBUM.".to_string());

    let tags = [&primary, &secondary];
    assert_eq!(
        MetadataExtractor::string_from_tags(tags.iter().copied(), &ItemKey::AlbumTitle)
            .as_deref(),
        Some("ALBUM.")
    );
}

#[test]
fn cross_tag_album_artist_read_falls_back_to_secondary_tag() {
    // #507: the album artist exists only in the APE tag.
    let primary = lofty::tag::Tag::new(lofty::tag::TagType::Id3v2);
    let mut secondary = lofty::tag::Tag::new(lofty::tag::TagType::Ape);
    secondary.insert_text(ItemKey::AlbumArtist, "Curated Artist".to_string());

    let tags = [&primary, &secondary];
    assert_eq!(
        MetadataExtractor::string_from_tags(tags.iter().copied(), &ItemKey::AlbumArtist)
            .as_deref(),
        Some("Curated Artist")
    );
}

#[test]
fn cross_tag_read_prefers_first_tag_on_conflict() {
    // The primary tag comes first in the iterator, so its value wins —
    // deterministic conflict policy (matches other players).
    let mut primary = lofty::tag::Tag::new(lofty::tag::TagType::Id3v2);
    primary.insert_text(ItemKey::AlbumTitle, "Primary Album".to_string());
    let mut secondary = lofty::tag::Tag::new(lofty::tag::TagType::Ape);
    secondary.insert_text(ItemKey::AlbumTitle, "Other Album".to_string());

    let tags = [&primary, &secondary];
    assert_eq!(
        MetadataExtractor::string_from_tags(tags.iter().copied(), &ItemKey::AlbumTitle)
            .as_deref(),
        Some("Primary Album")
    );
}

#[test]
fn cross_tag_read_skips_empty_values() {
    let mut primary = lofty::tag::Tag::new(lofty::tag::TagType::Id3v2);
    primary.insert_text(ItemKey::AlbumTitle, "   ".to_string());
    let mut secondary = lofty::tag::Tag::new(lofty::tag::TagType::Ape);
    secondary.insert_text(ItemKey::AlbumTitle, "Real Album".to_string());

    let tags = [&primary, &secondary];
    assert_eq!(
        MetadataExtractor::string_from_tags(tags.iter().copied(), &ItemKey::AlbumTitle)
            .as_deref(),
        Some("Real Album")
    );
}

#[test]
fn cross_tag_year_read_falls_back_to_secondary_tag() {
    // #447 year: the date exists only in the secondary tag.
    let primary = lofty::tag::Tag::new(lofty::tag::TagType::Id3v2);
    let mut secondary = lofty::tag::Tag::new(lofty::tag::TagType::Ape);
    secondary.insert_text(ItemKey::RecordingDate, "2025".to_string());

    let tags = [&primary, &secondary];
    assert_eq!(
        MetadataExtractor::year_from_tags(tags.iter().copied()),
        Some(2025)
    );
}

#[test]
fn cross_tag_track_and_disc_read_fall_back_to_secondary_tag() {
    let primary = lofty::tag::Tag::new(lofty::tag::TagType::Id3v2);
    let mut secondary = lofty::tag::Tag::new(lofty::tag::TagType::Ape);
    secondary.insert_text(ItemKey::TrackNumber, "7".to_string());
    secondary.insert_text(ItemKey::DiscNumber, "2".to_string());

    let tags = [&primary, &secondary];
    assert_eq!(
        MetadataExtractor::track_from_tags(tags.iter().copied()),
        Some(7)
    );
    assert_eq!(
        MetadataExtractor::disk_from_tags(tags.iter().copied()),
        Some(2)
    );
}
