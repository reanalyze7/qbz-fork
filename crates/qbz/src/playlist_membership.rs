//! Membership checks + removal for the "Add to playlist" picker's per-row
//! checkbox — the LOCAL-playlist-as-target half (spec
//! `PLAYLIST-REDESIGN-SPEC.md` §4). The Qobuz-playlist-as-target half lives
//! in `playlist_membership_qobuz.rs` (split to stay under the 130-line
//! budget; pure DB-membership logic, no Slint/window access).
//!
//! The picker needs to know, for every LOCAL playlist it lists, whether the
//! pending track(s)/refs are already members (checkbox state), and to remove
//! them again when the user unchecks a row. Refs are resolved exactly like
//! `local_playlist::add_local_refs_blocking` does (source-aware: an offline
//! Qobuz-download row becomes a Qobuz match).

use qbz_library::local_playlists as repo;

use crate::local_playlist::local_row_input;

/// Resolve pending picker refs to repo inputs, source-aware in local-refs
/// mode (mirrors `add_local_refs_blocking`), or plain Qobuz ids otherwise.
fn resolved_inputs(ids: &[String], local_mode: bool) -> Vec<repo::LocalPlaylistTrackInput> {
    if !local_mode {
        return ids
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .map(repo::LocalPlaylistTrackInput::Qobuz)
            .collect();
    }
    crate::library_db::with_db(|db| {
        let mut out = Vec::new();
        for r in ids {
            if let Ok(rid) = r.parse::<i64>() {
                if let Some(input) = local_row_input(db, rid)? {
                    out.push(input);
                }
            }
        }
        Ok(out)
    })
    .unwrap_or_default()
}

fn input_matches(input: &repo::LocalPlaylistTrackInput, existing: &repo::LocalPlaylistTrack) -> bool {
    match input {
        repo::LocalPlaylistTrackInput::Qobuz(qid) => existing.qobuz_track_id == Some(*qid),
        repo::LocalPlaylistTrackInput::Local(path) => existing.local_path.as_deref() == Some(path.as_str()),
    }
}

/// True when EVERY pending id/ref is already a member of the LOCAL playlist
/// `playlist_id`. Blocking (DB) — run on a worker thread.
pub fn already_has_blocking(playlist_id: &str, ids: &[String], local_mode: bool) -> bool {
    let pending = resolved_inputs(ids, local_mode);
    if pending.is_empty() {
        return false;
    }
    let existing = crate::local_playlist::get_tracks_blocking(playlist_id);
    pending.iter().all(|p| existing.iter().any(|e| input_matches(p, e)))
}

/// Remove every pending id/ref from the LOCAL playlist `playlist_id`.
/// Removes highest position first so `repo::remove_track`'s position
/// compaction never shifts a not-yet-removed match out from under us.
/// Blocking (DB). Returns the removed count.
pub fn remove_ids_blocking(playlist_id: &str, ids: &[String], local_mode: bool) -> usize {
    let pending = resolved_inputs(ids, local_mode);
    if pending.is_empty() {
        return 0;
    }
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            let existing = repo::get_tracks(conn, playlist_id).unwrap_or_default();
            let mut positions: Vec<i32> = existing
                .iter()
                .filter(|e| pending.iter().any(|p| input_matches(p, e)))
                .map(|e| e.position)
                .collect();
            positions.sort_unstable_by(|a, b| b.cmp(a));
            let mut removed = 0usize;
            for pos in positions {
                if repo::remove_track(conn, playlist_id, pos).is_ok() {
                    removed += 1;
                }
            }
            removed
        }))
    })
    .unwrap_or(0)
}
