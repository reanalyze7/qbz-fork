//! Test fixtures shared by the filter test modules, plus the module
//! declarations. Album and Track do not derive Default in qbz-models, so
//! fixtures build full struct literals; only the artist-id fields are
//! meaningful to the blacklist helpers, everything else is zero/None filler.

use qbz_models::types::{AlbumArtist, AlbumSummary};
use qbz_models::{Album, Artist, Track};

use super::super::AlbumBlacklistFilter;

mod album_predicate_tests;
mod other_predicate_tests;
mod search_tests;

/// Empty album-blacklist filter for the artist-axis tests (the album axis
/// is exercised by the dedicated album-id tests).
fn no_albums() -> AlbumBlacklistFilter {
    AlbumBlacklistFilter::new()
}

fn album_with_artists(primary_id: u64, featured_ids: &[u64]) -> Album {
    let artists = std::iter::once(AlbumArtist {
        id: primary_id,
        name: String::new(),
        roles: Some(vec!["main-artist".to_string()]),
    })
    .chain(featured_ids.iter().map(|&id| AlbumArtist {
        id,
        name: String::new(),
        roles: Some(vec!["featured-artist".to_string()]),
    }))
    .collect();
    Album {
        id: String::new(),
        title: String::new(),
        artist: Artist {
            id: primary_id,
            ..Default::default()
        },
        image: Default::default(),
        release_date_original: None,
        release_date_stream: None,
        streamable: None,
        label: None,
        genre: None,
        tracks_count: None,
        duration: None,
        hires: false,
        hires_streamable: false,
        maximum_sampling_rate: None,
        maximum_bit_depth: None,
        audio_info: None,
        dates: None,
        track_count: None,
        release_type: None,
        tracks: None,
        upc: None,
        description: None,
        goodies: None,
        parental_warning: None,
        artists: Some(artists),
        composer: None,
        version: None,
    }
}

fn track_with(performer_id: Option<u64>, composer_id: Option<u64>) -> Track {
    Track {
        id: 0,
        title: String::new(),
        version: None,
        isrc: None,
        duration: 0,
        track_number: 0,
        media_number: None,
        performer: performer_id.map(|id| Artist {
            id,
            ..Default::default()
        }),
        album: None,
        hires: false,
        hires_streamable: false,
        maximum_sampling_rate: None,
        maximum_bit_depth: None,
        streamable: false,
        parental_warning: false,
        playlist_track_id: None,
        performers: None,
        composer: composer_id.map(|id| Artist {
            id,
            ..Default::default()
        }),
        copyright: None,
        work: None,
    }
}

fn album_with_id(id: &str, primary_artist: u64) -> Album {
    let mut a = album_with_artists(primary_artist, &[]);
    a.id = id.to_string();
    a
}

fn track_with_album(album_id: &str) -> Track {
    let mut t = track_with(Some(1), Some(2));
    t.album = Some(AlbumSummary {
        id: album_id.to_string(),
        title: String::new(),
        image: Default::default(),
        label: None,
        genre: None,
    });
    t
}
