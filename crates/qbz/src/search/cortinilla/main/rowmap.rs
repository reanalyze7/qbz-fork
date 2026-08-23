//! Row builders shared between the ranked-section pass and the top-result
//! fallback in [`super::map_search_all_to_cortinilla`].

use qbz_models::{Album, Artist, Playlist, Track};

use crate::search::mappers::{map_album, map_artist, map_playlist, map_track};
use crate::search::rows::CortRow;

pub(super) fn to_artist_row(a: &Artist) -> CortRow {
    CortRow {
        kind: "artist".into(),
        id: a.id.to_string(),
        source: "qobuz".into(),
        title: a.name.clone(),
        subtitle: map_artist(a, false).subtitle,
        artwork_url: a
            .image
            .as_ref()
            .and_then(|i| i.best().cloned())
            .unwrap_or_default(),
        flat_index: 0,
    }
}

pub(super) fn to_album_row(al: &Album) -> CortRow {
    let m = map_album(al.clone());
    CortRow {
        kind: "album".into(),
        id: m.id,
        source: "qobuz".into(),
        title: m.title,
        subtitle: m.artist,
        artwork_url: m.artwork_url,
        flat_index: 0,
    }
}

pub(super) fn to_track_row(t: &Track) -> CortRow {
    let m = map_track(t.clone());
    CortRow {
        kind: "track".into(),
        id: m.id,
        source: "qobuz".into(),
        title: m.title,
        subtitle: m.artist,
        artwork_url: m.artwork_url,
        flat_index: 0,
    }
}

pub(super) fn to_playlist_row(p: &Playlist) -> CortRow {
    let m = map_playlist(p.clone());
    CortRow {
        kind: "playlist".into(),
        id: m.id,
        source: "qobuz".into(),
        title: m.title,
        subtitle: m.subtitle,
        artwork_url: m.cover_urls.first().cloned().unwrap_or_default(),
        flat_index: 0,
    }
}
