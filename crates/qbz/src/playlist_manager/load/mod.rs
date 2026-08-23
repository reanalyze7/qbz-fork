//! Fetch playlists (Qobuz) + folders + settings + stats + local counts
//! (local, library.db) and merge into the Send `PmData`.

mod gather;
mod internal_favorites;
mod offline_synth;

use std::collections::HashMap;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use gather::gather_blocking;
use internal_favorites::surface_internal_favorites;
use offline_synth::synthesize_offline_playlists;

use super::types::{cover_urls, PmData, PmPlaylist};

pub async fn load<A>(runtime: &AppRuntime<A>) -> PmData
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let remote = runtime.core().get_user_playlists().await.unwrap_or_else(|e| {
        log::warn!("[qbz-slint] playlist-manager playlists load failed: {e}");
        Vec::new()
    });
    // B7 producer (names): persist id+name(+owner, track_count) for ALL
    // listed playlists — data this load already fetched, written detached.
    // Offline the fetch is gate-refused (empty), so nothing is written.
    crate::playlist_snapshot::record_names_detached(
        remote
            .iter()
            .map(|p| crate::playlist_snapshot::SnapshotNameEntry {
                qobuz_playlist_id: p.id,
                name: p.name.clone(),
                owner: Some(p.owner.name.clone()).filter(|o| !o.is_empty()),
                track_count: Some(p.tracks_count),
            })
            .collect(),
    );

    let (folders, settings, play_counts, local_counts, locals, snapshot_names, snapshot_available) =
        tokio::task::spawn_blocking(gather_blocking)
            .await
            .unwrap_or_else(|_| {
                (
                    Vec::new(),
                    HashMap::new(),
                    HashMap::new(),
                    HashMap::new(),
                    Vec::new(),
                    HashMap::new(),
                    std::collections::HashSet::new(),
                )
            });

    let folder_ids: std::collections::HashSet<&String> = folders.iter().map(|f| &f.id).collect();

    let mut playlists: Vec<PmPlaylist> = remote
        .iter()
        .map(|p| {
            let s = settings.get(&p.id).cloned().unwrap_or_default();
            // A folder that no longer exists falls back to root (matches the
            // sidebar's `folder_ids.contains` guard).
            let folder_id = s.folder_id.filter(|fid| folder_ids.contains(fid));
            PmPlaylist {
                id: p.id,
                name: p.name.clone(),
                tracks_count: p.tracks_count,
                duration: p.duration,
                local_count: local_counts.get(&p.id).copied().unwrap_or(0),
                play_count: play_counts.get(&p.id).copied().unwrap_or(0),
                is_favorite: s.is_favorite,
                is_hidden: s.hidden,
                folder_id,
                position: s.position,
                cover_urls: cover_urls(p),
                offline_available: snapshot_available.contains(&p.id),
            }
        })
        .collect();

    surface_internal_favorites(
        runtime,
        &mut playlists,
        &folder_ids,
        &settings,
        &play_counts,
        &local_counts,
        &snapshot_available,
    )
    .await;

    synthesize_offline_playlists(
        &mut playlists,
        &folder_ids,
        &settings,
        &play_counts,
        &local_counts,
        &snapshot_names,
        &snapshot_available,
    );

    PmData {
        playlists,
        folders,
        locals,
    }
}
