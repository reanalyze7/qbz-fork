use super::*;
use qbz_models::{AlbumSummary, Artist, ImageSet};

mod score_candidate;
mod select_best_match;

fn import_track(title: &str, artist: &str) -> ImportTrack {
    ImportTrack {
        title: title.to_string(),
        artist: artist.to_string(),
        album: None,
        duration_ms: None,
        isrc: None,
        provider_id: None,
        provider_url: None,
    }
}

fn qobuz_track(id: u64, title: &str, artist: &str) -> Track {
    Track {
        id,
        title: title.to_string(),
        version: None,
        work: None,
        isrc: None,
        duration: 0,
        track_number: 0,
        media_number: None,
        performer: Some(Artist {
            name: artist.to_string(),
            ..Artist::default()
        }),
        album: None,
        hires: false,
        hires_streamable: false,
        maximum_sampling_rate: None,
        maximum_bit_depth: None,
        streamable: true,
        parental_warning: false,
        playlist_track_id: None,
        performers: None,
        composer: None,
        copyright: None,
    }
}

fn album_summary(title: &str) -> AlbumSummary {
    AlbumSummary {
        id: String::new(),
        title: title.to_string(),
        image: ImageSet::default(),
        label: None,
        genre: None,
    }
}
