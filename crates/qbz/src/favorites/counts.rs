use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::ComponentHandle;

use crate::{AppWindow, FavoritesState};

/// All five favorites tab counts, seeded up front so the tab badges are
/// ready before the user opens a given tab.
pub struct FavCounts {
    pub tracks: i32,
    pub albums: i32,
    pub artists: i32,
    pub playlists: i32,
    pub labels: i32,
}

async fn total_for<A>(runtime: &Arc<AppRuntime<A>>, key: &str) -> i32
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    runtime
        .core()
        .get_favorites(key, 1, 0)
        .await
        .ok()
        .and_then(|v| {
            v.get(key)
                .and_then(|b| b.get("total"))
                .and_then(|t| t.as_u64())
        })
        .unwrap_or(0) as i32
}

/// Fetch the five favorites counts (cheap limit=1 probes + the playlist
/// count). Runs on a worker; apply with `apply_counts` on the UI thread.
pub async fn load_counts<A>(runtime: &Arc<AppRuntime<A>>) -> FavCounts
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let tracks = total_for(runtime, "tracks").await;
    let albums = total_for(runtime, "albums").await;
    let artists = total_for(runtime, "artists").await;
    let labels = total_for(runtime, "labels").await;
    // The tab badge counts the LOCAL Library (favorited) playlists, matching
    // the "favorites" semantics (cheap local read, no network).
    let playlists = crate::library_db::with_db(|db| db.get_favorite_playlist_ids())
        .map(|ids| ids.len() as i32)
        .unwrap_or(0);
    FavCounts {
        tracks,
        albums,
        artists,
        playlists,
        labels,
    }
}

/// Apply the seeded counts to `FavoritesState` (the tab badges).
pub fn apply_counts(window: &AppWindow, c: FavCounts) {
    let st = window.global::<FavoritesState>();
    st.set_tracks_total(c.tracks);
    st.set_albums_total(c.albums);
    st.set_artists_total(c.artists);
    st.set_playlists_total(c.playlists);
    st.set_labels_total(c.labels);
}
