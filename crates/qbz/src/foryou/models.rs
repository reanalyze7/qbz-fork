//! Plain data structs for For You sections, plus the Qobuz -> local mappers.

use qbz_models::{Album, Artist};

pub struct SpotlightData {
    pub artist_id: String,
    pub artist_name: String,
    pub category: String,
    pub image_url: String,
    pub has_top_tracks: bool,
    pub albums: Vec<AlbumCard>,
}

#[derive(Clone)]
pub struct AlbumCard {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub year: String,
    pub quality_tier: String,
    pub quality_label: String,
    pub artwork_url: String,
}

#[derive(Clone)]
pub struct TrackSlim {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub artwork_url: String,
}

#[derive(Clone)]
pub struct ArtistSlim {
    pub id: String,
    pub name: String,
    pub artwork_url: String,
    pub following: bool,
}

pub(super) fn map_album(album: Album) -> AlbumCard {
    let year = album
        .release_date_original
        .as_deref()
        .and_then(|s| s.get(..4).map(|y| y.to_string()))
        .unwrap_or_default();
    let quality_tier = match album.maximum_bit_depth {
        Some(d) if d >= 24 => "hires",
        Some(_) => "cd",
        None => "",
    }
    .to_string();
    let quality_label = match (album.maximum_bit_depth, album.maximum_sampling_rate) {
        (Some(bd), Some(sr)) => format!("{}-bit / {} kHz", bd, sr),
        _ => String::new(),
    };
    AlbumCard {
        id: album.id,
        title: album.title,
        artist: album.artist.name,
        artist_id: album.artist.id.to_string(),
        year,
        quality_tier,
        quality_label,
        artwork_url: album.image.best().cloned().unwrap_or_default(),
    }
}

pub(super) fn map_artist(artist: Artist, following: bool) -> ArtistSlim {
    ArtistSlim {
        id: artist.id.to_string(),
        name: artist.name,
        artwork_url: artist
            .image
            .and_then(|img| img.best().cloned())
            .unwrap_or_default(),
        following,
    }
}
