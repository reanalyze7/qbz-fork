//! Membership checks + removal for the "Add to playlist" picker's per-row
//! checkbox — the QOBUZ-playlist-as-target half, LOCAL-mode refs only (spec
//! `PLAYLIST-REDESIGN-SPEC.md` §4). Companion to `playlist_membership.rs`
//! (the LOCAL-playlist-as-target half).
//!
//! QOBUZ playlist ← Qobuz catalog ids is NOT handled here: the caller uses
//! the existing `check_playlist_duplicates` core call (cheap, catalog-ids
//! only) for the has-check, and resolves `playlist_track_id`s via
//! `get_playlist` only when an actual removal is requested — avoiding a
//! heavy paginated fetch just to render a checkbox. QOBUZ playlist ← local
//! refs never touches the Qobuz API at all: local/Plex rows attach to a
//! Qobuz playlist through the `playlist_local_tracks` / `playlist_plex_tracks`
//! sidecar tables (see `main.rs`'s `on_pick` local-mode branch), so both the
//! has-check and the removal below stay fully local (DB only).

/// True when EVERY pending local-mode ref is already attached to the QOBUZ
/// playlist `pid` via the local/Plex sidecar tables. Blocking (DB).
pub fn already_has_refs_blocking(pid: u64, refs: &[String]) -> bool {
    if refs.is_empty() {
        return false;
    }
    let (local_ids, plex_keys): (Vec<i64>, Vec<String>) = crate::library_db::with_db(|db| {
        let locals = db.get_playlist_local_tracks(pid).unwrap_or_default();
        let plex = db.get_playlist_plex_tracks_with_position(pid).unwrap_or_default();
        Ok((locals.into_iter().map(|t| t.id).collect(), plex.into_iter().map(|(k, _)| k).collect()))
    })
    .unwrap_or_default();
    refs.iter().all(|r| {
        if let Some(key) = r.strip_prefix("plex:") {
            plex_keys.iter().any(|k| k == key)
        } else if let Ok(rid) = r.parse::<i64>() {
            local_ids.contains(&rid)
        } else {
            false
        }
    })
}

/// Detach every pending local-mode ref from the QOBUZ playlist `pid`'s
/// sidecar tables. Blocking (DB). Returns the removed count.
pub fn remove_refs_blocking(pid: u64, refs: &[String]) -> usize {
    crate::library_db::with_db(|db| {
        let mut removed = 0usize;
        for r in refs {
            if let Some(key) = r.strip_prefix("plex:") {
                if db.remove_plex_track_from_playlist(pid, key).is_ok() {
                    removed += 1;
                }
            } else if let Ok(rid) = r.parse::<i64>() {
                if db.remove_local_track_from_playlist(pid, rid).is_ok() {
                    removed += 1;
                }
            }
        }
        Ok(removed)
    })
    .unwrap_or(0)
}
