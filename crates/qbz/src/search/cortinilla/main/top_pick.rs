//! Top-result selection for [`super::map_search_all_to_cortinilla`]: prefer
//! the learned `(kind, id)` pick, else the most-popular hero, else the first
//! artist/album.

use qbz_models::{MostPopularItem, SearchAllResults};

use super::rowmap::{to_album_row, to_artist_row, to_playlist_row, to_track_row};
use crate::search::rows::CortRow;

/// Pick the top result. The promoted row is identified by (kind, id) so it
/// can be located across the already-mapped section rows; if the learned
/// pick is not present in the (truncated) sections, fall back to mapping the
/// raw catalog entry directly so it still shows even when ranked out.
pub(super) fn pick_top(
    results: &SearchAllResults,
    top_kind_id: Option<(String, String)>,
    artists: &[CortRow],
    albums: &[CortRow],
    tracks: &[CortRow],
    playlists: &[CortRow],
) -> Option<CortRow> {
    let find_in = |kind: &str, id: &str| -> Option<CortRow> {
        let sect = match kind {
            "artist" => artists,
            "album" => albums,
            "track" => tracks,
            "playlist" => playlists,
            _ => return None,
        };
        sect.iter().find(|r| r.id == id).cloned()
    };

    top_kind_id
        .and_then(|(kind, id)| {
            // Prefer a row already mapped; else map the raw catalog entry.
            find_in(&kind, &id).or_else(|| match kind.as_str() {
                "artist" => results
                    .artists
                    .items
                    .iter()
                    .find(|a| a.id.to_string() == id)
                    .map(to_artist_row),
                "album" => results
                    .albums
                    .items
                    .iter()
                    .find(|a| a.id == id)
                    .map(to_album_row),
                "track" => results
                    .tracks
                    .items
                    .iter()
                    .find(|t| t.id.to_string() == id)
                    .map(to_track_row),
                "playlist" => results
                    .playlists
                    .items
                    .iter()
                    .find(|p| p.id.to_string() == id)
                    .map(to_playlist_row),
                _ => None,
            })
        })
        .or_else(|| match &results.most_popular {
            // Reuse the existing most-popular shape as a fallback top result.
            Some(MostPopularItem::Artists(a)) => Some(to_artist_row(a)),
            Some(MostPopularItem::Albums(a)) => Some(to_album_row(a)),
            Some(MostPopularItem::Tracks(t)) => Some(to_track_row(t)),
            None => None,
        })
        .or_else(|| artists.first().cloned())
        .or_else(|| albums.first().cloned())
}
