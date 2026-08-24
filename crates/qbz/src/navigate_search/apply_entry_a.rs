use crate::*;

// `apply_entry` match arms: Home, Discover, Favorites, LocalLibrary,
// Settings, Album, LocalAlbum, Artist, Search, Musician, Label,
// LabelReleases, ArtistReleases. Returns the entry back to the caller
// unconsumed for any variant NOT in this subset, so `apply_entry_b`
// (apply_entry_b.rs) can try it next.
pub(crate) fn apply_entry_a(
    entry: nav::NavEntry,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) -> Option<nav::NavEntry> {
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
        other => return Some(other),
    }
    None
}
