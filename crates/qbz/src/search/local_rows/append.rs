use super::caps::{local_artwork_url, LocalCaps};
use super::derive::{derive_local_album_rows, derive_local_artist_rows};
use crate::search::cortinilla::assign_flat_indices;
use crate::search::rows::{CortRow, CortSection, CortinillaData};

/// Map one `LocalTrack` to a cortinilla `CortRow` tagged `source = "local"`.
///
/// `kind = "track"` (it navigates/plays as a track), but the click router keys
/// off `source == "local"` to play it through the LOCAL seam
/// (`playback::play_local_tracks`) rather than the Qobuz media-action. The id
/// is the library row id (`LocalTrack::id`) — the click router resolves the
/// concrete `LocalTrack` back from the per-query snapshot, NOT from this id.
///
/// Artwork prefixing mirrors `playback::local_queue_track` /
/// `local_library::map_local_track`: a raw fs path is `file://`-prefixed unless
/// it already carries a `file://` scheme.
fn map_local_track_to_cort_row(t: &qbz_library::LocalTrack) -> CortRow {
    let artwork_url = local_artwork_url(t.artwork_path.as_deref());
    // Subtitle: "artist · album" when both are present, else whichever exists.
    let subtitle = match (t.artist.is_empty(), t.album.is_empty()) {
        (false, false) => format!("{} · {}", t.artist, t.album),
        (false, true) => t.artist.clone(),
        (true, false) => t.album.clone(),
        (true, true) => String::new(),
    };
    CortRow {
        kind: "track".into(),
        id: t.id.to_string(),
        source: "local".into(),
        title: t.title.clone(),
        subtitle,
        artwork_url,
        flat_index: 0,
    }
}

/// Append the local "on this device" sections to a MAIN cortinilla payload,
/// placed LAST (after every Qobuz category, per D1/D2 — local results live ONLY
/// in the cortinilla). Three sections in display order: **Albums**, **Artists**,
/// **Tracks** (mirrors the Qobuz section order), each capped per [`LocalCaps`].
/// Albums/artists are DERIVED by grouping the local track rows. Section `kind`s
/// are `local-album` / `local-artist` / `local` so the "View more" router opens
/// the matching LocalLibrary tab; per-row `kind` stays album/artist/track so the
/// thumbnail shape + the row click router route correctly. No-op when `rows` is
/// empty. Re-runs `assign_flat_indices` so the local rows get contiguous flat
/// indices AFTER the Qobuz sections.
pub fn append_local_sections(
    data: &mut CortinillaData,
    rows: &[qbz_library::LocalTrack],
    caps: LocalCaps,
) {
    if rows.is_empty() {
        return;
    }
    let (album_rows, albums_more) = derive_local_album_rows(rows, caps.albums);
    if !album_rows.is_empty() {
        data.sections.push(CortSection {
            title: qbz_i18n::t("Albums on Local Library"),
            kind: "local-album".to_string(),
            rows: album_rows,
            has_more: albums_more,
        });
    }
    let (artist_rows, artists_more) = derive_local_artist_rows(rows, caps.artists);
    if !artist_rows.is_empty() {
        data.sections.push(CortSection {
            title: qbz_i18n::t("Artists on Local Library"),
            kind: "local-artist".to_string(),
            rows: artist_rows,
            has_more: artists_more,
        });
    }
    let track_rows: Vec<CortRow> = rows
        .iter()
        .take(caps.tracks)
        .map(map_local_track_to_cort_row)
        .collect();
    if !track_rows.is_empty() {
        let shown = track_rows.len();
        data.sections.push(CortSection {
            title: qbz_i18n::t("On Local Library"),
            kind: "local".to_string(),
            rows: track_rows,
            has_more: rows.len() > shown,
        });
    }
    assign_flat_indices(data);
}
