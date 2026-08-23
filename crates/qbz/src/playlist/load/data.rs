//! `PlaylistData` — plain, `Send` playlist data produced on the worker
//! thread — and the Tauri-parity absolute-slot interleave that merges Qobuz
//! tracks with local sidecar rows.

use qbz_models::Track;

use crate::local_playlist::{LoadedRow, RowItem};

/// Plain, `Send` playlist data produced on the worker thread.
pub struct PlaylistData {
    pub id: String,
    pub name: String,
    pub owner: String,
    /// Qobuz owner id — compared against the current user id to decide
    /// ownership (owned => deletable; not owned => followed/subscribed).
    pub owner_id: u64,
    pub description: String,
    pub description_short: String,
    pub cover_url: String,
    /// Local custom artwork path (from playlist_settings), if the user
    /// set one — overrides the collage / server image.
    pub custom_artwork_path: Option<String>,
    /// The MERGED row list (Qobuz tracks interleaved with the local
    /// sidecar rows at their absolute slots — Seam A) in display order.
    /// Pure-Qobuz playlists are simply all `RowItem::Qobuz`.
    pub rows: Vec<LoadedRow>,
}

/// Tauri's absolute-slot interleave — the `displayTracks` contract
/// (spec §1.2): sidecar rows claim their STORED positions as slots in the
/// merged list; Qobuz tracks fill the remaining slots in server order;
/// `total = max(sum of rows, max stored position + 1)` so stale high slots
/// still render (E3); unclaimed slots with no Qobuz track left are skipped
/// (never a blank); leftover Qobuz tracks append. Same-slot collisions emit
/// ALL claimants, in stable claim order — instead of Tauri's Map collapse
/// (E1/E2 fix-forward; healing repairs the stored data separately). Display
/// numbering is the emit order (contiguous).
pub(crate) fn interleave_rows(qobuz: Vec<Track>, sidecar: Vec<LoadedRow>) -> Vec<LoadedRow> {
    let qobuz_to_row = |(i, t): (usize, Track)| LoadedRow {
        position: i as i32,
        item: RowItem::Qobuz(Box::new(t)),
    };
    if sidecar.is_empty() {
        return qobuz.into_iter().enumerate().map(qobuz_to_row).collect();
    }
    let sidecar_len = sidecar.len();
    let mut max_pos: i32 = -1;
    let mut buckets: std::collections::HashMap<i32, Vec<LoadedRow>> =
        std::collections::HashMap::new();
    for row in sidecar {
        // Corrupt negative positions claim slot 0 rather than vanishing.
        let pos = row.position.max(0);
        max_pos = max_pos.max(pos);
        buckets.entry(pos).or_default().push(row);
    }
    let total = (qobuz.len() + sidecar_len).max((max_pos + 1) as usize);
    let mut out: Vec<LoadedRow> = Vec::with_capacity(qobuz.len() + sidecar_len);
    let mut qobuz_iter = qobuz.into_iter();
    for pos in 0..total as i32 {
        if let Some(rows) = buckets.remove(&pos) {
            out.extend(rows);
        } else if let Some(track) = qobuz_iter.next() {
            out.push(LoadedRow {
                position: pos,
                item: RowItem::Qobuz(Box::new(track)),
            });
        }
        // else: an unclaimed slot past the Qobuz tracks — a gap, skipped.
    }
    for track in qobuz_iter {
        out.push(LoadedRow {
            position: 0,
            item: RowItem::Qobuz(Box::new(track)),
        });
    }
    // Positions in the merged output are the contiguous display slots; the
    // stored sidecar positions did their job claiming the order.
    for (i, row) in out.iter_mut().enumerate() {
        row.position = i as i32;
    }
    out
}

/// Word-boundary truncation for the 2-line header description (the
/// full text lives in the Read-more modal).
pub(super) fn truncate_words(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max).collect();
    let cut = truncated.rfind(' ').unwrap_or(truncated.len());
    format!("{}…", truncated[..cut].trim_end())
}
