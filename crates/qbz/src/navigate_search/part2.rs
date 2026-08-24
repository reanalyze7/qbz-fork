// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for.
use crate::*;

pub(crate) fn apply_entry(
    entry: nav::NavEntry,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    match entry {
        nav::NavEntry::Home => {
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NavState>().set_view(ContentView::Home);
            });
        }
        nav::NavEntry::Discover { tab } => {
            let for_you = tab == "forYou";
            let recommendations = tab == "recommendations";
            {
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                let _ = weak.clone().upgrade_in_event_loop(move |w| {
                    w.global::<NavState>().set_view(ContentView::Home);
                    let jobs = home::select_tab(&w, &tab);
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                });
            }
            if for_you {
                ensure_for_you_loaded(runtime, weak, handle, image_cache);
            }
            if recommendations {
                external_reco::ensure_loaded(runtime, weak, handle, image_cache);
            }
        }
        nav::NavEntry::Favorites { tab } => {
            if let Some(fav_tab) = favorites::FavTab::from_tab_id(&tab) {
                navigate_favorites(
                    runtime.clone(),
                    weak.clone(),
                    handle,
                    image_cache.clone(),
                    fav_tab,
                    &tab,
                );
            }
        }
        nav::NavEntry::LocalLibrary { tab } => {
            if let Some(lib_tab) = local_library::LibTab::from_tab_id(&tab) {
                navigate_local_library(
                    runtime.clone(),
                    weak.clone(),
                    handle,
                    image_cache.clone(),
                    lib_tab,
                );
            }
        }
        nav::NavEntry::Settings => {
            let _ = weak.upgrade_in_event_loop(|w| {
                seed_blacklist_status(&w);
                w.global::<NavState>().set_view(ContentView::Settings);
            });
        }
        nav::NavEntry::Album(id) => {
            navigate_album(runtime.clone(), weak.clone(), handle, image_cache.clone(), id);
        }
        nav::NavEntry::LocalAlbum(gk) => {
            navigate_local_album(runtime.clone(), weak.clone(), handle, image_cache.clone(), gk);
        }
        nav::NavEntry::Artist(id) => {
            navigate_artist(runtime.clone(), weak.clone(), handle, image_cache.clone(), id);
        }
        nav::NavEntry::Search(query) => {
            navigate_search(runtime.clone(), weak.clone(), handle, image_cache.clone(), query);
        }
        nav::NavEntry::Musician { name, role } => {
            navigate_musician(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
                name,
                role,
            );
        }
        nav::NavEntry::Label { id, name } => {
            navigate_label(runtime.clone(), weak.clone(), handle, image_cache.clone(), id, name);
        }
        nav::NavEntry::LabelReleases { id, name } => {
            navigate_label_releases(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
                id,
                name,
            );
        }
        nav::NavEntry::ArtistReleases {
            id,
            name,
            release_type,
        } => {
            navigate_artist_releases(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
                id,
                name,
                release_type,
            );
        }
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

