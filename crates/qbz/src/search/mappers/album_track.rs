use qbz_models::{Album, Track};

use crate::search::pure::{mmss, quality_label, tier, year_of};
use crate::search::rows::{AlbumRow, TrackRowData};

pub fn map_album(album: Album) -> AlbumRow {
    AlbumRow {
        id: album.id,
        title: crate::album_map::format_album_title(&album.title, album.version.as_deref()),
        artist: album.artist.name,
        artist_id: album.artist.id.to_string(),
        genre: album
            .genre
            .map(|g| g.name)
            .filter(|n| !n.is_empty())
            .unwrap_or_default(),
        year: year_of(album.release_date_original.as_deref()),
        quality_tier: tier(album.maximum_bit_depth).to_string(),
        quality_label: quality_label(album.maximum_bit_depth, album.maximum_sampling_rate),
        artwork_url: album.image.best().cloned().unwrap_or_default(),
    }
}

pub fn map_track(track: Track) -> TrackRowData {
    let mut title = track.title;
    if let Some(version) = track.version.as_ref().filter(|v| !v.is_empty()) {
        title = format!("{title} ({version})");
    }
    let artwork_url = track
        .album
        .as_ref()
        .and_then(|a| a.image.best().cloned())
        .unwrap_or_default();
    let album_id = track.album.as_ref().map(|a| a.id.clone()).unwrap_or_default();
    let (artist, artist_id) = track
        .performer
        .map(|p| (p.name, p.id.to_string()))
        .unwrap_or_default();
    TrackRowData {
        id: track.id.to_string(),
        title,
        artist,
        artist_id,
        album_id,
        duration: mmss(track.duration),
        quality_tier: tier(track.maximum_bit_depth).to_string(),
        quality_label: quality_label(track.maximum_bit_depth, track.maximum_sampling_rate),
        quality_detail: crate::quality::detail(
            track.maximum_bit_depth,
            track.maximum_sampling_rate,
        ),
        explicit: track.parental_warning,
        artwork_url,
    }
}
