//! Cross-tag reading core.
//!
//! Backs the per-key cross-tag fallback used by [`super::MetadataExtractor`]:
//! old, repeatedly-retagged collections routinely hold the album /
//! album-artist / date only in a SECONDARY tag (ID3v1/APE/Vorbis) while the
//! file-type's primary tag lacks them — reading just the primary tag dropped
//! them (#447 folder-name albums, #507 ignored Album Artist).

use lofty::prelude::*;
use lofty::tag::ItemKey;

use super::MetadataExtractor;

impl MetadataExtractor {
    pub(super) fn normalize_field(value: Option<&str>) -> Option<String> {
        value
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    /// Iterate the file's tags with the primary tag FIRST, then the rest in
    /// file order.
    pub(super) fn tags_primary_first<'a>(
        tagged_file: &'a lofty::file::TaggedFile,
    ) -> impl Iterator<Item = &'a lofty::tag::Tag> + 'a {
        let primary = tagged_file.primary_tag();
        primary.into_iter().chain(
            tagged_file
                .tags()
                .iter()
                .filter(move |t| primary.map_or(true, |p| !std::ptr::eq(*t, p))),
        )
    }

    /// First non-empty string for `key` across all of the file's tags
    /// (primary first). When several tags disagree, the primary tag wins —
    /// deterministic, and matches what other players show.
    pub(super) fn string_across_tags(
        tagged_file: &lofty::file::TaggedFile,
        key: &ItemKey,
    ) -> Option<String> {
        Self::string_from_tags(Self::tags_primary_first(tagged_file), key)
    }

    /// Pure core of [`Self::string_across_tags`]: first non-empty, trimmed
    /// value for `key` yielded by the tag iterator (already in priority
    /// order). The empty check lives INSIDE the find_map so a blank value in
    /// one tag does not shadow a real value in a later tag.
    pub(super) fn string_from_tags<'a>(
        mut tags: impl Iterator<Item = &'a lofty::tag::Tag>,
        key: &ItemKey,
    ) -> Option<String> {
        tags.find_map(|t| {
            t.get_string(key.clone())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
    }

    /// First track number across all tags (primary first).
    pub(super) fn track_across_tags(tagged_file: &lofty::file::TaggedFile) -> Option<u32> {
        Self::track_from_tags(Self::tags_primary_first(tagged_file))
    }

    /// Pure core of [`Self::track_across_tags`].
    pub(super) fn track_from_tags<'a>(
        mut tags: impl Iterator<Item = &'a lofty::tag::Tag>,
    ) -> Option<u32> {
        tags.find_map(|t| t.track())
    }

    /// First disc number across all tags (primary first).
    pub(super) fn disk_across_tags(tagged_file: &lofty::file::TaggedFile) -> Option<u32> {
        Self::disk_from_tags(Self::tags_primary_first(tagged_file))
    }

    /// Pure core of [`Self::disk_across_tags`].
    pub(super) fn disk_from_tags<'a>(
        mut tags: impl Iterator<Item = &'a lofty::tag::Tag>,
    ) -> Option<u32> {
        tags.find_map(|t| t.disk())
    }

    /// First parseable date's year across all tags (primary first);
    /// `Tag::date()` already falls back RecordingDate -> Year within a tag.
    pub(super) fn year_across_tags(tagged_file: &lofty::file::TaggedFile) -> Option<u32> {
        Self::year_from_tags(Self::tags_primary_first(tagged_file))
    }

    /// Pure core of [`Self::year_across_tags`].
    pub(super) fn year_from_tags<'a>(
        mut tags: impl Iterator<Item = &'a lofty::tag::Tag>,
    ) -> Option<u32> {
        tags.find_map(|t| t.date()).map(|ts| ts.year as u32)
    }
}
