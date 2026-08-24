use crate::*;

/// Run a search and show the results view. Shared by the search-submit
/// callback, the live-search debounce, and history back/forward.
pub(crate) fn navigate_search(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    query: String,
) {
    // Capture a version so a slow, stale load cannot overwrite a newer
    // search's results (the user kept typing).
    let version = search::next_search_version();
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            search::reset_search(&w);
            w.global::<NavState>().set_view(ContentView::Search);
        });
        match search::load_search(&runtime, &query).await {
            Ok(data) => {
                let jobs = search::artwork_jobs(&data);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    if search::is_current_version(version) {
                        search::apply_search(&w, data);
                        w.global::<SearchState>().set_loading(false);
                    }
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache);
            }
            Err(e) => {
                log::error!("[qbz-slint] search load failed: {e}");
                let _ = weak.upgrade_in_event_loop(move |w| {
                    if search::is_current_version(version) {
                        w.global::<SearchState>().set_loading(false);
                    }
                });
            }
        }
    });
}

/// Apply a history entry — set the view and re-load entity pages.
/// Stable scroll-restore id for an entry's primary list container, matching
/// the `restore-scope` strings the Slint scroll containers compare against.
/// Returns `""` for views without a wired scroll memory (no container will
/// match, so nothing restores). Tab/sub-page views carry the tab in the id so
/// each tab keeps its own position. Keep in sync with the `.slint` views.
pub(crate) fn scope_for(entry: &nav::NavEntry) -> String {
    match entry {
        // HomeView is one persistent Flickable shared by the Discover tabs;
        // a single scope is enough (each tab entry stores its own scroll).
        nav::NavEntry::Home | nav::NavEntry::Discover { .. } => "home".into(),
        nav::NavEntry::Favorites { tab } => format!("fav:{tab}"),
        nav::NavEntry::LocalLibrary { tab } => format!("ll:{tab}"),
        nav::NavEntry::DiscoverBrowse { .. } => "discover-browse".into(),
        nav::NavEntry::PlaylistBrowse => "playlist-browse".into(),
        nav::NavEntry::RecentAlbums => "recent-albums".into(),
        nav::NavEntry::MostPlayedAlbums => "most-played-albums".into(),
        nav::NavEntry::Mix { .. } => "mix".into(),
        nav::NavEntry::Playlist(_) => "playlist".into(),
        nav::NavEntry::PlaylistManager => "playlist-manager".into(),
        nav::NavEntry::OfflineManager => "offline-manager".into(),
        nav::NavEntry::BlacklistManager => "blacklist-manager".into(),
        nav::NavEntry::Mixtapes => "mixtapes".into(),
        nav::NavEntry::Collections => "collections".into(),
        nav::NavEntry::MixtapeDetail(_) => "mixtape-detail".into(),
        nav::NavEntry::Album(_) => "album".into(),
        nav::NavEntry::LocalAlbum(_) => "local-album".into(),
        nav::NavEntry::Artist(_) => "artist".into(),
        nav::NavEntry::Settings => "settings".into(),
        nav::NavEntry::Search(_) => "search".into(),
        nav::NavEntry::Musician { .. } => "musician".into(),
        nav::NavEntry::Label { .. } => "label".into(),
        nav::NavEntry::LabelReleases { .. } => "label-releases".into(),
        nav::NavEntry::ArtistReleases { .. } => "artist-releases".into(),
        nav::NavEntry::Location { .. } => "location".into(),
    }
}

/// Arm `NavState` so the destination scroll container restores its saved
/// position once it mounts. Must run before `apply_entry` switches the view.
pub(crate) fn arm_scroll_restore(weak: &slint::Weak<AppWindow>, entry: &nav::NavEntry, scroll: f32) {
    if let Some(w) = weak.upgrade() {
        let ns = w.global::<NavState>();
        ns.set_restore_scope(scope_for(entry).into());
        ns.set_scroll_restore(scroll);
    }
}

