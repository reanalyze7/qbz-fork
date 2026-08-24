use crate::*;

// `apply_entry` match arms: DiscoverBrowse, PlaylistBrowse, RecentAlbums,
// MostPlayedAlbums, Mix, Playlist, PlaylistManager, OfflineManager,
// BlacklistManager, Mixtapes, Collections, MixtapeDetail, Location. Tried
// by `apply_entry` (part2.rs) only for whatever `apply_entry_a` didn't
// consume.
pub(crate) fn apply_entry_b(
    entry: nav::NavEntry,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    match entry {
        nav::NavEntry::DiscoverBrowse { endpoint, title } => {
            discover_browse::navigate(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
                endpoint,
                title,
                current_genre_filter(),
            );
        }
        nav::NavEntry::PlaylistBrowse => {
            // History re-entry keeps the session tag (reset_tag = false);
            // only a fresh open from the rail resets it to All.
            playlist_browse::navigate(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
                current_genre_filter(),
                false,
            );
        }
        nav::NavEntry::RecentAlbums => {
            navigate_recent_albums(weak.clone(), handle, image_cache.clone());
        }
        nav::NavEntry::MostPlayedAlbums => {
            navigate_most_played_albums(weak.clone(), handle, image_cache.clone());
        }
        nav::NavEntry::Mix { kind } => {
            navigate_mix(runtime.clone(), weak.clone(), handle, image_cache.clone(), kind);
        }
        nav::NavEntry::Playlist(id) => {
            navigate_playlist(runtime.clone(), weak.clone(), handle, image_cache.clone(), id);
        }
        nav::NavEntry::PlaylistManager => {
            playlist_manager::navigate(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
            );
        }
        nav::NavEntry::OfflineManager => {
            let w2 = weak.clone();
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NavState>().set_view(ContentView::OfflineManager);
            });
            offline_manager::load(w2, handle.clone());
        }
        nav::NavEntry::BlacklistManager => {
            let w2 = weak.clone();
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NavState>().set_view(ContentView::BlacklistManager);
            });
            blacklist_manager::load(w2);
        }
        nav::NavEntry::Mixtapes => {
            myqbz::navigate(
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                qbz_models::mixtape::CollectionKind::Mixtape,
            );
        }
        nav::NavEntry::Collections => {
            myqbz::navigate(
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                qbz_models::mixtape::CollectionKind::Collection,
            );
        }
        nav::NavEntry::MixtapeDetail(id) => {
            myqbz_detail::navigate(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                id,
            );
        }
        nav::NavEntry::Location {
            mbid,
            area_id,
            area_name,
            country,
            genres,
            tags,
        } => {
            let params = artist::LocationParams {
                mbid,
                area_id,
                area_name,
                country,
                genres,
                tags,
            };
            navigate_location(runtime.clone(), weak.clone(), handle, image_cache.clone(), params);
        }
    }
}
