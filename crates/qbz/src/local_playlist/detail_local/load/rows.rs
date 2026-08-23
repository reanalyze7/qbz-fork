use std::collections::{HashMap, HashSet};

use qbz_library::local_playlists as repo;

use crate::local_playlist::row::{LoadedRow, RowItem};

/// Fold the raw playlist track refs into resolved `LoadedRow`s, honoring
/// D11 (hide unresolvable Qobuz rows), the D11.local filename fallback, and
/// the legacy synthetic-id honesty rule. Logs per-category counts.
pub(super) fn build_rows(
    id: &str,
    tracks: Vec<repo::LocalPlaylistTrack>,
    mut fetched: HashMap<u64, qbz_models::Track>,
    mut cached: HashMap<u64, RowItem>,
    locals: &HashMap<String, qbz_library::LocalTrack>,
    on_disk: &HashSet<String>,
) -> Vec<LoadedRow> {
    let mut rows: Vec<LoadedRow> = Vec::new();
    let mut hidden = 0usize;
    let mut missing_files = 0usize;
    let mut unresolved = 0usize;
    for t in tracks {
        let item = match t.source {
            repo::LocalPlaylistTrackSource::Qobuz => {
                let Some(tid) = t.qobuz_track_id else {
                    hidden += 1;
                    continue;
                };
                if tid >= crate::local_library::LEGACY_SYNTHETIC_ID_FLOOR {
                    // NOT a Qobuz catalog id — a legacy synthetic
                    // 2^40-namespaced id stored as qobuz_track_id by the
                    // pre-typed-drag bug. It can never resolve; render it
                    // honestly (removable) instead of D11-hiding it forever.
                    unresolved += 1;
                    log::warn!(
                        "[qbz-slint] local playlist {id}: qobuz ref {tid} is outside the catalog range (legacy mis-typed row) — rendered as unavailable"
                    );
                    RowItem::Unresolved {
                        kind: "qobuz",
                        reference: tid.to_string(),
                    }
                } else if let Some(track) = fetched.remove(&tid) {
                    RowItem::Qobuz(Box::new(track))
                } else if let Some(item) = cached.remove(&tid) {
                    item
                } else {
                    // D11: no metadata source for this Qobuz row right now.
                    hidden += 1;
                    continue;
                }
            }
            repo::LocalPlaylistTrackSource::Local => match t.local_path.as_ref() {
                Some(p) => {
                    if let Some(track) = locals.get(p) {
                        RowItem::Local(Box::new(track.clone()))
                    } else if on_disk.contains(p) {
                        // Index miss but the file exists — render it
                        // (filename fallback) instead of hiding.
                        RowItem::LocalFile { path: p.clone() }
                    } else {
                        // The file itself is gone — hide, but say so.
                        missing_files += 1;
                        continue;
                    }
                }
                None => {
                    hidden += 1;
                    continue;
                }
            },
        };
        rows.push(LoadedRow {
            position: t.position,
            item,
        });
    }
    if hidden > 0 {
        log::info!("[qbz-slint] local playlist {id}: {hidden} row(s) unavailable, hidden (D11)");
    }
    if missing_files > 0 {
        log::info!(
            "[qbz-slint] local playlist {id}: {missing_files} local file row(s) missing on disk, hidden (D11.local)"
        );
    }
    if unresolved > 0 {
        log::info!(
            "[qbz-slint] local playlist {id}: {unresolved} row(s) with unresolvable refs, rendered as unavailable"
        );
    }
    rows
}
