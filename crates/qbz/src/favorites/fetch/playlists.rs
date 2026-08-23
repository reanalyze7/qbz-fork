use std::collections::HashMap;
use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::Playlist;

use super::FavData;
use crate::search::{self, PlaylistRow};

/// Playlists has two sub-tabs from two different sources (mirror Tauri):
///   - Following = playlists the user follows on Qobuz but does NOT own
///     (`get_user_playlists` filtered by `owner.id != current_user_id`).
///   - Library   = LOCALLY-favorited playlist ids (SQLite), in favorited
///     order. We intersect the already-fetched `get_user_playlists` set
///     (cheap, no extra fetch); for a favorited id not in that set we fall
///     back to a single `get_playlist`.
pub(crate) async fn load_playlists<A>(runtime: &Arc<AppRuntime<A>>) -> Result<FavData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let all = runtime
        .core()
        .get_user_playlists()
        .await
        .map_err(|e| e.to_string())?;
    let uid = crate::library_db::current_user_id();
    // Following = non-owned playlists present in get_user_playlists →
    // subscribed by definition, so stamp is_following for the card overlay.
    let following: Vec<PlaylistRow> = match uid {
        Some(uid) => all
            .iter()
            .filter(|p| p.owner.id != uid)
            .cloned()
            .map(|p| {
                let mut r = search::map_playlist(p);
                r.is_following = true;
                r
            })
            .collect(),
        None => Vec::new(),
    };
    let fav_ids =
        crate::library_db::with_db(|db| db.get_favorite_playlist_ids()).unwrap_or_default();
    let by_id: HashMap<u64, &Playlist> = all.iter().map(|p| (p.id, p)).collect();
    // The user's OWNED playlists belong in their Library sub-tab. They don't
    // come back from SQLite as "favorites" unless manually hearted, and the
    // Following list above explicitly excludes them (owner.id != uid), so
    // without seeding them here they appear NOWHERE (bug: only followed
    // playlists showed). Owned first, then hearted-but-not-owned (dedup).
    let mut favorites: Vec<PlaylistRow> = Vec::new();
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
    if let Some(uid) = uid {
        for p in all.iter().filter(|p| p.owner.id == uid) {
            seen.insert(p.id);
            favorites.push(search::map_playlist(p.clone()));
        }
    }
    for fid in fav_ids {
        if !seen.insert(fid) {
            continue;
        }
        if let Some(p) = by_id.get(&fid) {
            // Hearted AND in the user's playlists: owned ones were already
            // seeded above, so a non-owned one here is a followed playlist.
            let mut r = search::map_playlist((**p).clone());
            if !r.is_owned {
                r.is_following = true;
            }
            favorites.push(r);
        } else if let Ok(p) = runtime.core().get_playlist(fid).await {
            // Hearted but NOT in the user's playlists → not subscribed.
            favorites.push(search::map_playlist(p));
        }
    }
    Ok(FavData::Playlists { favorites, following })
}
