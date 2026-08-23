//! Worker-thread fetch + mapping pipeline: discover index -> `HomeData`.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

mod playlists;
mod recent;
mod sections;

pub use recent::{recent_album_cards, recent_track_slims};

use super::HomeData;

/// Fetch the discover index (optionally genre-filtered) and map it
/// into the Home / Editor's Picks / For You section sets.
pub async fn load_home<A>(
    runtime: &Arc<AppRuntime<A>>,
    genre_ids: Option<Vec<u64>>,
) -> Result<HomeData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    // The personalized Home rails (#566: Library Albums / Release Watch /
    // Your Top Artists) resolve CONCURRENTLY with the index so they add no
    // latency to the home load. Fetched unconditionally (like the
    // Qobuzissimes cache-pool precedent below): the configurator re-render
    // is cache-only, so enabling a section must find its data populated.
    let (response, favorite_albums, release_watch, top_artists) = futures_util::join!(
        runtime.core().get_discover_index(genre_ids),
        crate::foryou::favorite_album_cards(runtime),
        crate::foryou::fetch_release_watch(runtime),
        crate::foryou::top_artist_cards(runtime),
    );
    let response = response.map_err(|e| e.to_string())?;
    let mut containers = response.containers;

    sections::apply_blacklist(&mut containers);

    // Genre filtering is server-side: the selected genre ids (parent OR
    // sub-genre, raw) were passed to get_discover_index, which honors sub-genre
    // ids in `genre_ids`. No client-side narrowing — 1:1 with Tauri
    // discovery-v2, which rendered the faceted response as-is (narrowing here
    // wrongly dropped albums tagged only at top level).

    // Editorial-only set for the Editor's Picks tab, built by cloning the
    // containers so the same data can also feed the Home set and the
    // most-streamed slim grid below.
    let editor_sections = sections::editor_sections(&containers);
    let popular = sections::popular_slims(containers.most_streamed.take());
    let sections = sections::home_sections(&mut containers);

    // For You is loaded separately + lazily by crate::foryou into its
    // own dedicated view, so the home load no longer builds a For You
    // section set here.

    // Recently played comes from the local play-history store, not the
    // discover index. Empty until the playback session records plays.
    let recent = recent::recent_track_slims();
    let recent_albums = recent::recent_album_cards();

    let (playlists, editor_playlists) = playlists::cards(&mut containers);
    let playlist_tags = playlists::tags(&mut containers);

    Ok(HomeData {
        sections,
        editor_sections,
        popular,
        recent,
        recent_albums,
        playlists,
        editor_playlists,
        playlist_tags,
        favorite_albums,
        release_watch,
        top_artists,
        most_played_albums: crate::foryou::most_played_album_cards(),
    })
}
