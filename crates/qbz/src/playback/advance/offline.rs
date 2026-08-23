//! Offline playability verdicts: "can we actually play this track right
//! now", independent of any error/skip bookkeeping.

use qbz_models::QueueTrack;

/// Offline playability verdict for one queue track (offline-MODE slice 3d).
#[derive(PartialEq)]
pub(in super::super) enum OfflinePlayability {
    Playable,
    /// No offline source for this track (Qobuz without a cached copy).
    Unavailable,
    /// The track IS offline-cached but the D4 subscription grace window has
    /// elapsed — gets its own honest message.
    GraceExpired,
    /// A LOCAL user file whose indexed path is not on disk right now —
    /// typically an unmounted network drive. Checked online AND offline
    /// (library content is never hidden, so playback is where this must
    /// surface) — gets the "is the drive mounted?" message.
    FileMissing,
}

/// Cheap existence guard for a LOCAL queue track's underlying file: resolve
/// the indexed path (ephemeral store, or one indexed library-DB read) and
/// stat it with `Path::exists()`. Unresolvable id/path → `true` (don't
/// invent a skip; the play path has its own not-found handling).
///
/// D-STATE CAVEAT: `exists()` on an UNMOUNTED path returns false instantly
/// (the path simply isn't there) — that is the case this guards. But a stat
/// on a DEAD-yet-still-MOUNTED NFS/CIFS share can block in uninterruptible
/// sleep (D state). This is therefore only ever called from the async layer
/// (the advance walk and the play fast-fail run on the tokio runtime;
/// `play_local_file_audible` checks inside `spawn_blocking`) and NEVER from
/// the audio callback thread — a worst-case hang stalls an advance, not the
/// audio pipeline. Do NOT add mount probing here; the hot path stays one
/// stat per advance.
pub(in super::super) fn local_track_file_exists(track: &QueueTrack) -> bool {
    let path = if crate::ephemeral::is_ephemeral_id(track.id as i64) {
        crate::ephemeral::get_track(track.id as i64).map(|row| row.file_path)
    } else {
        crate::library_db::with_db(|db| db.get_track(track.id as i64))
            .flatten()
            .map(|row| row.file_path)
    };
    match path {
        Some(p) => std::path::Path::new(&p).exists(),
        None => true,
    }
}

/// Decide whether `track` can play under the CURRENT offline status.
/// Local / ephemeral user files → existence-checked regardless of
/// online/offline (the library never hides network-folder content — see
/// local_library.rs's NETWORK-FOLDER VISIBILITY note — so an unmounted
/// drive is caught here, at playback time).
/// Online → otherwise always playable (the normal path pays one status read).
/// Offline:
/// - qobuz (incl. "qobuz_download" copies, which keep the real Qobuz id)
///   → offline-cached AND within the D4 subscription grace window
pub(in super::super) fn offline_playability(track: &QueueTrack) -> OfflinePlayability {
    if matches!(track.source.as_deref(), Some("local") | Some("ephemeral")) {
        return if local_track_file_exists(track) {
            OfflinePlayability::Playable
        } else {
            OfflinePlayability::FileMissing
        };
    }
    let status = crate::offline_mode::engine().status();
    if !status.is_offline() {
        return OfflinePlayability::Playable;
    }
    if track.is_local {
        return OfflinePlayability::Playable;
    }
    // ("local" / "ephemeral" never reach here — handled above.)
    if !crate::offline_cache::is_cached(&track.id.to_string()) {
        OfflinePlayability::Unavailable
    } else if !crate::offline_mode::offline_playback_allowed() {
        OfflinePlayability::GraceExpired
    } else {
        OfflinePlayability::Playable
    }
}

/// Boolean form of [`offline_playability`] for the advance/prefetch walks.
pub(in super::super) fn offline_track_playable(track: &QueueTrack) -> bool {
    offline_playability(track) == OfflinePlayability::Playable
}
