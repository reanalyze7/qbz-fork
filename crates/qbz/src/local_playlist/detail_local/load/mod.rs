mod local;
mod qobuz;
mod rows;

use local::resolve_local;
use qobuz::resolve_qobuz;
use rows::build_rows;

use crate::local_playlist::repo::{get_blocking, get_tracks_blocking};
use crate::local_playlist::row::LocalPlaylistData;
use crate::local_playlist::Runtime;

/// Load + resolve a local playlist off the UI thread. Qobuz rows resolve
/// via `get_tracks_batch` when online, via the offline-cache index when
/// offline (or when the batch fails); local rows via library.db by path.
/// Unresolvable QOBUZ rows are filtered out (D11); a LOCAL row that misses
/// the index still renders (filename fallback) while its file exists, and
/// hides (logged distinctly) only when the file itself is gone; a
/// `qobuz_track_id` in the legacy synthetic namespace (mis-typed garbage)
/// renders an honest "unavailable" row the user can still select and remove.
pub async fn load(runtime: &Runtime, playlist_id: &str) -> Option<LocalPlaylistData> {
    let id = playlist_id.to_string();
    let (header, tracks) = tokio::task::spawn_blocking({
        let id = id.clone();
        move || (get_blocking(&id), get_tracks_blocking(&id))
    })
    .await
    .ok()?;
    let header = header?;

    let offline = crate::offline_mode::engine().is_offline();

    let qobuz_ids: Vec<u64> = tracks.iter().filter_map(|t| t.qobuz_track_id).collect();
    let (fetched, cached) = resolve_qobuz(runtime, &id, &qobuz_ids, offline).await;

    let local_paths: Vec<String> = tracks.iter().filter_map(|t| t.local_path.clone()).collect();
    let (locals, on_disk) = resolve_local(local_paths).await;

    let rows = build_rows(&id, tracks, fetched, cached, &locals, &on_disk);

    Some(LocalPlaylistData {
        id: header.id,
        name: header.name,
        description: header.description.unwrap_or_default(),
        offline_only: header.offline_only,
        custom_artwork_path: header.custom_artwork_path.filter(|p| !p.is_empty()),
        rows,
    })
}
