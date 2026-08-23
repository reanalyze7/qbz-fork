//! Fetch + map the Qobuz playlist list (the online half of `load()`).

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::SidebarPlaylist;

/// Pick up to four de-duplicated cover URLs for a playlist, preferring the
/// largest available list (mirrors Tauri's `images150 ?? images300 ??
/// images`, but de-duplicated and capped at four for the 2x2 collage).
pub(super) fn playlist_cover_urls(p: &qbz_models::Playlist) -> Vec<String> {
    let source = [&p.images300, &p.images150, &p.images]
        .into_iter()
        .flatten()
        .find(|v| !v.is_empty());
    let mut out: Vec<String> = Vec::new();
    if let Some(list) = source {
        for url in list {
            if !url.is_empty() && !out.contains(url) {
                out.push(url.clone());
            }
            if out.len() == 4 {
                break;
            }
        }
    }
    out
}

/// Fetch the user's Qobuz playlists and map them to `SidebarPlaylist`.
/// Offline (or on any fetch error) returns empty — the offline synthesis in
/// `load_offline.rs` fills the gap.
pub(super) async fn fetch_playlists<A>(runtime: &AppRuntime<A>) -> Vec<SidebarPlaylist>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime.core().get_user_playlists().await {
        Ok(pls) => {
            // B7 producer (names): persist id+name(+owner, track_count) for
            // ALL listed playlists — data this load already fetched, written
            // detached so the render never waits. Offline the fetch is
            // gate-refused (Err), so the snapshot is never clobbered.
            crate::playlist_snapshot::record_names_detached(
                pls.iter()
                    .map(|p| crate::playlist_snapshot::SnapshotNameEntry {
                        qobuz_playlist_id: p.id,
                        name: p.name.clone(),
                        owner: Some(p.owner.name.clone()).filter(|o| !o.is_empty()),
                        track_count: Some(p.tracks_count),
                    })
                    .collect(),
            );
            pls.into_iter()
                .map(|p| SidebarPlaylist {
                    id: p.id,
                    name: p.name.clone(),
                    description: p.description.clone().unwrap_or_default(),
                    tracks_count: p.tracks_count,
                    cover_urls: playlist_cover_urls(&p),
                    position: 0,
                })
                .collect()
        }
        Err(e) => {
            log::warn!("[qbz-slint] sidebar playlists load failed: {e}");
            Vec::new()
        }
    }
}
