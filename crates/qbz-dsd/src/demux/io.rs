//! Shared byte-reading helpers used by both the DSF and DFF readers.

use super::{DsdError, DsdTags};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const VALID_RATES: [u32; 4] = [2_822_400, 5_644_800, 11_289_600, 22_579_200];

pub(super) fn read_u32_le(f: &mut File) -> Result<u32, DsdError> {
    let mut b = [0u8; 4];
    f.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}
pub(super) fn read_u64_le(f: &mut File) -> Result<u64, DsdError> {
    let mut b = [0u8; 8];
    f.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}
pub(super) fn read_u64_be(f: &mut File) -> Result<u64, DsdError> {
    let mut b = [0u8; 8];
    f.read_exact(&mut b)?;
    Ok(u64::from_be_bytes(b))
}
pub(super) fn read_id(f: &mut File) -> Result<[u8; 4], DsdError> {
    let mut b = [0u8; 4];
    f.read_exact(&mut b)?;
    Ok(b)
}

pub(super) fn validate_rate(rate: u32) -> Result<(), DsdError> {
    if VALID_RATES.contains(&rate) {
        Ok(())
    } else {
        Err(DsdError::UnsupportedRate(rate))
    }
}

pub(super) fn read_id3_tags(file: &mut File, offset: u64) -> DsdTags {
    let mut tags = DsdTags::default();
    if file.seek(SeekFrom::Start(offset)).is_err() {
        return tags;
    }
    match id3::Tag::read_from2(&mut *file) {
        Ok(tag) => {
            use id3::TagLike;
            tags.title = tag.title().map(str::to_string);
            tags.artist = tag.artist().map(str::to_string);
            tags.album = tag.album().map(str::to_string);
            tags.album_artist = tag.album_artist().map(str::to_string);
            tags.genre = tag.genre_parsed().map(|g| g.into_owned());
            tags.track_number = tag.track();
            tags.disc_number = tag.disc();
            tags.year = tag.year().or_else(|| tag.date_recorded().map(|d| d.year));
            tags.artwork = tag
                .pictures()
                .find(|p| p.picture_type == id3::frame::PictureType::CoverFront)
                .or_else(|| tag.pictures().next())
                .map(|p| p.data.clone());
        }
        Err(e) => log::debug!("[qbz-dsd] ID3 read failed (non-fatal): {e}"),
    }
    tags
}
