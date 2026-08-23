//! Surface INTERNAL favorites (hearted playlists) that are neither owned
//! nor subscribed — they don't come back from `get_user_playlists`, so
//! without this a favorited-but-not-followed Qobuz playlist would be
//! invisible in the manager. Online only; mirrors Favorites>Playlists.

use std::collections::{HashMap, HashSet};

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::folders::PlaylistSettingsLite;

use super::super::types::{cover_urls, PmPlaylist};

#[allow(clippy::too_many_arguments)]
pub(super) async fn surface_internal_favorites<A>(
    runtime: &AppRuntime<A>,
    playlists: &mut Vec<PmPlaylist>,
    folder_ids: &HashSet<&String>,
    settings: &HashMap<u64, PlaylistSettingsLite>,
    play_counts: &HashMap<u64, u32>,
    local_counts: &HashMap<u64, u32>,
    snapshot_available: &HashSet<u64>,
) where
    A: FrontendAdapter + Send + Sync + 'static,
{
    if crate::offline_mode::engine().is_offline() {
        return;
    }
    let fav_ids =
        crate::library_db::with_db(|db| db.get_favorite_playlist_ids()).unwrap_or_default();
    let known: HashSet<u64> = playlists.iter().map(|p| p.id).collect();
    for fid in fav_ids {
        if known.contains(&fid) {
            continue;
        }
        if let Ok(p) = runtime.core().get_playlist(fid).await {
            let s = settings.get(&fid).cloned().unwrap_or_default();
            playlists.push(PmPlaylist {
                id: fid,
                name: p.name.clone(),
                tracks_count: p.tracks_count,
                duration: p.duration,
                local_count: local_counts.get(&fid).copied().unwrap_or(0),
                play_count: play_counts.get(&fid).copied().unwrap_or(0),
                is_favorite: true,
                is_hidden: s.hidden,
                folder_id: s.folder_id.filter(|f| folder_ids.contains(f)),
                position: s.position,
                cover_urls: cover_urls(&p),
                offline_available: snapshot_available.contains(&fid),
            });
        }
    }
}
