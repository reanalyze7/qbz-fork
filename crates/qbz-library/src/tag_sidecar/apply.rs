use crate::LocalTrack;

use super::AlbumTagSidecar;

pub fn apply_sidecar_to_track(track: &mut LocalTrack, sidecar: &AlbumTagSidecar) {
    if let Some(title) = sidecar
        .album
        .album_title
        .as_ref()
        .and_then(|s| normalize(s))
    {
        track.album = title.clone();
        track.album_group_title = title.clone();
    }

    if let Some(album_artist) = sidecar
        .album
        .album_artist
        .as_ref()
        .and_then(|s| normalize(s))
    {
        track.album_artist = Some(album_artist.clone());
    }

    if let Some(year) = sidecar.album.year {
        track.year = Some(year);
    }

    if let Some(genre) = sidecar.album.genre.as_ref().and_then(|s| normalize(s)) {
        track.genre = Some(genre.clone());
    }

    if let Some(cat) = sidecar
        .album
        .catalog_number
        .as_ref()
        .and_then(|s| normalize(s))
    {
        track.catalog_number = Some(cat.clone());
    }

    if let Some(entry) = sidecar.tracks.iter().find(|t| {
        t.file_path == track.file_path
            && match (t.cue_start_secs, track.cue_start_secs) {
                (Some(a), Some(b)) => (a - b).abs() < 0.001,
                (None, None) => true,
                _ => false,
            }
    }) {
        if let Some(title) = entry.title.as_ref().and_then(|s| normalize(s)) {
            track.title = title.clone();
        }
        if let Some(disc) = entry.disc_number {
            track.disc_number = Some(disc);
        }
        if let Some(no) = entry.track_number {
            track.track_number = Some(no);
        }
    }
}

fn normalize(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
