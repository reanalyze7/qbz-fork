use qbz_models::{Album, Artist, Playlist, SearchAllResults};

use super::assign_flat_indices;
use crate::search::mappers::{map_album, map_artist, map_playlist};
use crate::search::rows::{CortRow, CortSection, CortinillaData};

/// Per-category caps for the IMMERSIVE search cortinilla (owner sketch).
const IMMERSIVE_CAP_ARTISTS: usize = 2;
const IMMERSIVE_CAP_ALBUMS: usize = 5;
const IMMERSIVE_CAP_PLAYLISTS: usize = 2;

/// Immersive-search variant of [`super::map_search_all_to_cortinilla`]: **Albums /
/// Artists / Playlists ONLY** (no tracks, no local, no top-result hero —
/// immersive has no navigation, so selecting a row acts on the queue instead).
/// Section order matches the owner sketch: Artists, Albums, Playlists. Intra-
/// category order still applies the learned ranking before truncation.
pub fn map_search_all_to_immersive(query: &str, results: &SearchAllResults) -> CortinillaData {
    let to_artist_row = |a: &Artist| CortRow {
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
    };
    let to_album_row = |al: &Album| {
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
    };
    let to_playlist_row = |p: &Playlist| {
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
    };

    let take = |kind: &str, mut rows: Vec<CortRow>, cap: usize| -> Vec<CortRow> {
        crate::search_service::rank_within(query, kind, &mut rows, |r| r.id.clone());
        rows.truncate(cap);
        rows
    };

    let artists = take(
        "artist",
        results.artists.items.iter().map(to_artist_row).collect(),
        IMMERSIVE_CAP_ARTISTS,
    );
    let albums = take(
        "album",
        results.albums.items.iter().map(to_album_row).collect(),
        IMMERSIVE_CAP_ALBUMS,
    );
    let playlists = take(
        "playlist",
        results.playlists.items.iter().map(to_playlist_row).collect(),
        IMMERSIVE_CAP_PLAYLISTS,
    );

    let mut sections: Vec<CortSection> = Vec::new();
    let mut push = |title: &str, kind: &str, rows: Vec<CortRow>, total: u32| {
        if !rows.is_empty() {
            sections.push(CortSection {
                title: title.to_string(),
                kind: kind.to_string(),
                has_more: total as usize > rows.len(),
                rows,
            });
        }
    };
    push(&qbz_i18n::t("Artists"), "artist", artists, results.artists.total);
    push(&qbz_i18n::t("Albums"), "album", albums, results.albums.total);
    push(&qbz_i18n::t("Playlists"), "playlist", playlists, results.playlists.total);

    let mut data = CortinillaData {
        query: query.to_string(),
        top: None,
        sections,
    };
    assign_flat_indices(&mut data);
    data
}
