use std::collections::HashSet;

use super::caps::{local_album_artist, local_artwork_url};
use crate::search::rows::CortRow;

/// Group local TRACK rows into local ALBUM cortinilla rows (`source = "local"`,
/// `kind = "album"`). Grouped by `album_group_key` in first-seen order (the DB
/// returns rows by match relevance). `id` is the group key — the click router
/// opens the LocalAlbum view with it (`navigate_local_album`). Returns the
/// capped rows plus whether more distinct albums existed than shown.
pub(crate) fn derive_local_album_rows(
    rows: &[qbz_library::LocalTrack],
    cap: usize,
) -> (Vec<CortRow>, bool) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<CortRow> = Vec::new();
    let mut total = 0usize;
    for t in rows {
        let key = t.album_group_key.clone();
        if key.is_empty() || !seen.insert(key.clone()) {
            continue;
        }
        total += 1;
        if out.len() >= cap {
            continue; // keep counting for an honest has_more
        }
        let title = if t.album_group_title.is_empty() {
            t.album.clone()
        } else {
            t.album_group_title.clone()
        };
        out.push(CortRow {
            kind: "album".into(),
            id: key,
            source: "local".into(),
            title,
            subtitle: local_album_artist(t),
            artwork_url: local_artwork_url(t.artwork_path.as_deref()),
            flat_index: 0,
        });
    }
    let has_more = total > out.len();
    (out, has_more)
}

/// Group local TRACK rows into local ARTIST cortinilla rows (`source = "local"`,
/// `kind = "artist"`). Grouped by the canonical album-artist, case-insensitively,
/// in first-seen order. Local artists have no id — the click router opens the
/// LocalLibrary Artists tab by NAME (the row `title`), so `id` is left empty.
pub(crate) fn derive_local_artist_rows(
    rows: &[qbz_library::LocalTrack],
    cap: usize,
) -> (Vec<CortRow>, bool) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<CortRow> = Vec::new();
    let mut total = 0usize;
    for t in rows {
        let name = local_album_artist(t);
        if name.is_empty() || !seen.insert(name.to_lowercase()) {
            continue;
        }
        total += 1;
        if out.len() >= cap {
            continue;
        }
        out.push(CortRow {
            kind: "artist".into(),
            id: String::new(),
            source: "local".into(),
            title: name,
            subtitle: String::new(),
            artwork_url: local_artwork_url(t.artwork_path.as_deref()),
            flat_index: 0,
        });
    }
    let has_more = total > out.len();
    (out, has_more)
}
