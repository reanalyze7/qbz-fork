//! Discover album -> `CardData` mapper.

use qbz_models::DiscoverAlbum;

use super::quality::{classify_release_type, quality_detail, quality_label, quality_tier};
use crate::home::CardData;

pub(crate) fn map_album(album: DiscoverAlbum) -> CardData {
    let artist = album
        .artists
        .first()
        .map(|a| a.name.clone())
        .unwrap_or_default();
    let artist_id = album
        .artists
        .first()
        .map(|a| a.id.to_string())
        .unwrap_or_default();
    let genre = album.genre.map(|g| g.name).unwrap_or_default();
    let year = crate::dates::release_label(
        album
            .dates
            .as_ref()
            .and_then(|d| d.original.as_ref().or(d.download.as_ref()).or(d.stream.as_ref()))
            .map(|s| s.as_str()),
    );
    let quality_tier_val = quality_tier(album.audio_info.as_ref()).to_string();
    let quality_label_val = quality_label(album.audio_info.as_ref());
    let quality_detail_val = quality_detail(album.audio_info.as_ref());
    let artwork_url = album
        .image
        .large
        .or(album.image.thumbnail)
        .or(album.image.small)
        .unwrap_or_default();
    // Bare 4-digit year for the list-row YEAR column (the grid uses the
    // localized `year`); plus a track-count display string and a release
    // type heuristic for the list-row TYPE column.
    let plain_year = album
        .dates
        .as_ref()
        .and_then(|d| d.original.as_ref().or(d.download.as_ref()).or(d.stream.as_ref()))
        .and_then(|s| s.get(0..4))
        .unwrap_or_default()
        .to_string();
    let track_count = album.track_count.map(|n| n.to_string()).unwrap_or_default();
    let release_type = qbz_i18n::t(classify_release_type(album.track_count));
    CardData {
        id: album.id,
        title: album.title,
        artist,
        artist_id,
        genre,
        year,
        quality_tier: quality_tier_val,
        quality_label: quality_label_val,
        ribbon: String::new(),
        ribbon_kind: String::new(),
        artwork_url,
        release_type,
        // Discover is always the Qobuz catalog.
        source: "qobuz".to_string(),
        quality_detail: quality_detail_val,
        track_count,
        plain_year,
    }
}
