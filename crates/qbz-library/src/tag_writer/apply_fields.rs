use super::types::{AlbumTagWrite, TrackTagWrite};

/// Sets title/album/artist/track/disc/album-artist/year/genre/catalog-number
/// on one `Tag` per `album`/`track`. A blank/`None` field removes the tag.
pub(super) fn apply_tag_fields(
    tag: &mut lofty::tag::Tag,
    album: &AlbumTagWrite,
    track: &TrackTagWrite,
) {
    use lofty::{prelude::*, tag::ItemKey};

    tag.set_title(track.title.trim().to_string());
    tag.set_album(album.album_title.trim().to_string());
    tag.set_artist(album.album_artist.trim().to_string());

    if let Some(no) = track.track_number {
        tag.set_track(no);
    }
    if let Some(disc) = track.disc_number {
        tag.set_disk(disc);
    }

    // Album artist (not part of the Accessor trait).
    if album.album_artist.trim().is_empty() {
        tag.remove_key(ItemKey::AlbumArtist);
    } else {
        tag.insert_text(ItemKey::AlbumArtist, album.album_artist.trim().to_string());
    }

    // Year.
    if let Some(year) = album.year {
        tag.set_date(lofty::tag::items::Timestamp {
            year: year as u16,
            ..Default::default()
        });
    } else {
        tag.remove_date();
    }

    // Genre.
    if let Some(g) = album
        .genre
        .as_ref()
        .map(|g| g.trim())
        .filter(|g| !g.is_empty())
    {
        tag.set_genre(g.to_string());
    } else {
        tag.remove_genre();
    }

    // Catalog number.
    if let Some(c) = album
        .catalog_number
        .as_ref()
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
    {
        tag.insert_text(ItemKey::CatalogNumber, c.to_string());
    } else {
        tag.remove_key(ItemKey::CatalogNumber);
    }
}
