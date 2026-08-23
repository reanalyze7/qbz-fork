mod rowmap;
mod top_pick;

use qbz_models::SearchAllResults;

use rowmap::{to_album_row, to_artist_row, to_playlist_row, to_track_row};
use top_pick::pick_top;

use super::assign_flat_indices;
use crate::search::rows::{
    CortRow, CortSection, CortinillaData, CORTINILLA_CAP_ALBUMS, CORTINILLA_CAP_ARTISTS,
    CORTINILLA_CAP_PLAYLISTS, CORTINILLA_CAP_TRACKS,
};

/// Build the cortinilla payload from a combined-search result.
///
/// Section order (display + flat-index order): **Top result**, then **Albums,
/// Artists, Tracks, Playlists** (spec §6.2.3). Per-category caps (albums 5,
/// artists 2, tracks/playlists 3) — artists are rarely opened past the first
/// hit, so albums get the freed space; `has_more` is set when the category's
/// reported total exceeds the rows shown.
///
/// Intra-category order applies the qbz-app learned ranking
/// (`search_service::rank_within`) BEFORE truncation, so a frequently-opened
/// entity floats to the top of its section.
///
/// Top result: see [`top_pick::pick_top`] — learned pick, else most-popular
/// hero, else first artist/album. The promoted entity is NOT removed from its
/// section (the cortinilla is small; a one-row dup is acceptable and matches
/// the results page, which keeps the artist in the Artists tab).
pub fn map_search_all_to_cortinilla(
    query: &str,
    results: &SearchAllResults,
    top_kind_id: Option<(String, String)>,
) -> CortinillaData {
    // Map each category to CortRow (source = "qobuz"), apply ranking, truncate.
    let rank_and_take = |kind: &str, mut rows: Vec<CortRow>, cap: usize| -> (Vec<CortRow>, usize) {
        let total = rows.len();
        crate::search_service::rank_within(query, kind, &mut rows, |r| r.id.clone());
        rows.truncate(cap);
        (rows, total)
    };

    let artist_rows: Vec<CortRow> = results.artists.items.iter().map(to_artist_row).collect();
    let album_rows: Vec<CortRow> = results.albums.items.iter().map(to_album_row).collect();
    let track_rows: Vec<CortRow> = results.tracks.items.iter().map(to_track_row).collect();
    let playlist_rows: Vec<CortRow> =
        results.playlists.items.iter().map(to_playlist_row).collect();

    let (artists, _) = rank_and_take("artist", artist_rows, CORTINILLA_CAP_ARTISTS);
    let (albums, _) = rank_and_take("album", album_rows, CORTINILLA_CAP_ALBUMS);
    let (tracks, _) = rank_and_take("track", track_rows, CORTINILLA_CAP_TRACKS);
    let (playlists, _) = rank_and_take("playlist", playlist_rows, CORTINILLA_CAP_PLAYLISTS);

    let top = pick_top(results, top_kind_id, &artists, &albums, &tracks, &playlists);

    // Assemble sections in display order (spec §6.2.3): Albums, Artists,
    // Tracks, Playlists. The local "on this device" sections are appended LAST,
    // outside this function (see `append_local_sections`).
    let mut sections: Vec<CortSection> = Vec::new();
    let mut push_section = |title: &str, kind: &str, rows: Vec<CortRow>, total: u32| {
        if !rows.is_empty() {
            sections.push(CortSection {
                title: title.to_string(),
                kind: kind.to_string(),
                has_more: total as usize > rows.len(),
                rows,
            });
        }
    };
    push_section(&qbz_i18n::t("Albums"), "album", albums, results.albums.total);
    push_section(&qbz_i18n::t("Artists"), "artist", artists, results.artists.total);
    push_section(&qbz_i18n::t("Tracks"), "track", tracks, results.tracks.total);
    push_section(&qbz_i18n::t("Playlists"), "playlist", playlists, results.playlists.total);

    // Assign the flat selection index across the whole navigable list:
    // top-result = 0, then every section's rows in display order.
    let mut data = CortinillaData {
        query: query.to_string(),
        top,
        sections,
    };
    assign_flat_indices(&mut data);
    data
}
