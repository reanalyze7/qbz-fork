//! Synchronous local-DB resolver family + the full item-type dispatch matrix.
//! Frontend-agnostic: no `&LibraryDatabase` is ever held across an `.await`.

use qbz_models::mixtape::{ItemType, MixtapeCollectionItem};
use qbz_models::QueueTrack as CoreQueueTrack;

use super::mapping::local_track_to_queue_track;

// ── Local album (synchronous, frontend-agnostic) ──

/// Resolve a `Local` album item's `source_item_id` into tracks against the
/// passed `&LibraryDatabase`. Synchronous — no `&LibraryDatabase` is held
/// across an `.await`.
pub fn resolve_local_album(
    db: &qbz_library::LibraryDatabase,
    group_key: &str,
) -> Result<Vec<CoreQueueTrack>, String> {
    resolve_local_album_tracks(db, group_key)
}

/// Resolve a local album group against the library DB.
pub fn resolve_local_album_tracks(
    db: &qbz_library::LibraryDatabase,
    group_key: &str,
) -> Result<Vec<CoreQueueTrack>, String> {
    let tracks = db
        .get_album_tracks(group_key)
        .map_err(|e| format!("local get_album_tracks({}) failed: {}", group_key, e))?;

    if tracks.is_empty() {
        return Err(format!("local album {} has 0 tracks", group_key));
    }

    Ok(tracks.iter().map(local_track_to_queue_track).collect())
}

// ── Local track (synchronous) ──

pub fn resolve_local_track(
    db: &qbz_library::LibraryDatabase,
    track_id: i64,
) -> Result<Vec<CoreQueueTrack>, String> {
    let track = db
        .get_track(track_id)
        .map_err(|e| format!("local get_track({}) failed: {}", track_id, e))?
        .ok_or_else(|| format!("local track {} not found", track_id))?;

    Ok(vec![local_track_to_queue_track(&track)])
}

/// Full local-item dispatch contract (Album / Track / Playlist), centralized in
/// the crate so frontends do NOT re-implement (and silently drift from) the
/// matrix. This is the synchronous `&LibraryDatabase` counterpart of the async
/// Qobuz resolvers; a frontend wires it into `ProdItemResolver`'s `local`
/// closure through its own DB accessor — e.g. Slint's
/// `with_db(|db| resolve_local_item(db, item))` (the closure runs in a sync
/// scope, so `&LibraryDatabase` never crosses an `.await`).
///
/// Mirrors the original src-tauri `ProdItemResolver` Local arms exactly:
/// - Album    → [`resolve_local_album`]
/// - Track    → parse `source_item_id` to `i64` (`invalid local track id`) → [`resolve_local_track`]
/// - Playlist → hard error `local playlists not supported in this release`
pub fn resolve_local_item(
    db: &qbz_library::LibraryDatabase,
    item: &MixtapeCollectionItem,
) -> Result<Vec<CoreQueueTrack>, String> {
    match item.item_type {
        ItemType::Album => resolve_local_album(db, &item.source_item_id),
        ItemType::Track => {
            let track_id: i64 = item
                .source_item_id
                .parse()
                .map_err(|_| format!("invalid local track id: {}", item.source_item_id))?;
            resolve_local_track(db, track_id)
        }
        ItemType::Playlist => {
            // Local playlists are not supported in this release. The library DB
            // schema stores qobuz_playlist_id + local_track_id rows but there is
            // no unique "local-only playlist id" to resolve against.
            Err("local playlists not supported in this release".into())
        }
    }
}
