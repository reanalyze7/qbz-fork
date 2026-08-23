use qbz_library::local_playlists as repo;

/// Append Qobuz track ids. Returns inserted count. Ids in the legacy
/// synthetic namespace (>= 2^40 — see `local_library::LEGACY_SYNTHETIC_ID_FLOOR`)
/// are NOT Qobuz catalog ids; storing one writes a forever-unresolvable
/// row (the field garbage class), so they are refused and logged here,
/// at the last gate before the repo write.
pub fn add_qobuz_tracks_blocking(id: &str, track_ids: &[u64]) -> usize {
    let entries: Vec<repo::LocalPlaylistTrackInput> = track_ids
        .iter()
        .filter(|&&tid| {
            if tid >= crate::local_library::LEGACY_SYNTHETIC_ID_FLOOR {
                log::warn!(
                    "[qbz-slint] local playlist add: refused non-catalog id {tid} as a Qobuz ref"
                );
                false
            } else {
                true
            }
        })
        .map(|&tid| repo::LocalPlaylistTrackInput::Qobuz(tid))
        .collect();
    if entries.is_empty() {
        return 0;
    }
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::add_tracks(conn, id, &entries)))
    })
    .and_then(|r| r.ok())
    .unwrap_or(0)
}

/// Resolve a `local_tracks` row to its playlist input, source-aware:
/// offline copies (`qobuz_download`) become Qobuz refs (real catalog id),
/// everything else a local file path.
pub(crate) fn local_row_input(
    db: &qbz_library::LibraryDatabase,
    rid: i64,
) -> Result<Option<repo::LocalPlaylistTrackInput>, qbz_library::LibraryError> {
    let Some(track) = db.get_track(rid)? else {
        log::warn!("[qbz-slint] local playlist add: unknown local row {rid}");
        return Ok(None);
    };
    Ok(Some(match track.source.as_deref() {
        Some("qobuz_download") => match track.qobuz_track_id {
            Some(qid) => repo::LocalPlaylistTrackInput::Qobuz(qid as u64),
            None => repo::LocalPlaylistTrackInput::Local(track.file_path.clone()),
        },
        _ => repo::LocalPlaylistTrackInput::Local(track.file_path.clone()),
    }))
}

fn add_inputs_blocking(id: &str, entries: &[repo::LocalPlaylistTrackInput]) -> usize {
    if entries.is_empty() {
        return 0;
    }
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::add_tracks(conn, id, entries)))
    })
    .and_then(|r| r.ok())
    .unwrap_or(0)
}

/// Append local-mode picker refs — `"<i64>"` LocalLibrary row ids (resolved
/// source-aware via [`local_row_input`]). Returns inserted count.
pub fn add_local_refs_blocking(id: &str, refs: &[String]) -> usize {
    let entries: Vec<repo::LocalPlaylistTrackInput> = crate::library_db::with_db(|db| {
        let mut out = Vec::new();
        for r in refs {
            if let Ok(rid) = r.parse::<i64>() {
                if let Some(input) = local_row_input(db, rid)? {
                    out.push(input);
                }
            } else {
                log::warn!("[qbz-slint] local playlist add: unrecognized ref {r}");
            }
        }
        Ok(out)
    })
    .unwrap_or_default();
    add_inputs_blocking(id, &entries)
}

/// Append a drag payload (sidebar drop), mapping every variant to its own
/// playlist ref — local file rows store `local_path`, Qobuz/offline-cached
/// rows `qobuz_track_id`. Returns inserted count.
pub fn add_drag_tracks_blocking(id: &str, tracks: &[crate::drag::DragTrack]) -> usize {
    let entries: Vec<repo::LocalPlaylistTrackInput> = crate::library_db::with_db(|db| {
        let mut out = Vec::new();
        for item in tracks {
            match item {
                crate::drag::DragTrack::Qobuz(tid) => {
                    if *tid >= crate::local_library::LEGACY_SYNTHETIC_ID_FLOOR {
                        // Not a catalog id (legacy synthetic namespace) — a
                        // mis-typed payload; refuse rather than store a
                        // forever-unresolvable row.
                        log::warn!(
                            "[qbz-slint] local playlist drop: refused non-catalog id {tid} as a Qobuz ref"
                        );
                        continue;
                    }
                    out.push(repo::LocalPlaylistTrackInput::Qobuz(*tid));
                }
                crate::drag::DragTrack::LocalRow(rid) => {
                    if let Some(input) = local_row_input(db, *rid)? {
                        out.push(input);
                    }
                }
            }
        }
        Ok(out)
    })
    .unwrap_or_default();
    add_inputs_blocking(id, &entries)
}
