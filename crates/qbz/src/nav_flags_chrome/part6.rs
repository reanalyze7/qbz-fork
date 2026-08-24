use crate::*;

/// Map a top-level view to its persisted key for "Startup page = where you left
/// off" — ONLY safe, id-free views (the main nav destinations). Detail views
/// (album/artist/playlist/…) need a context id that may be stale on restart, so
/// they return None and the last safe view is kept instead.
pub(crate) fn safe_view_key(view: ContentView) -> Option<&'static str> {
    match view {
        // Home is also the Discover landing (its tabs render in the Home view),
        // so this covers both. Detail views (album/artist/playlist/…) + the
        // endpoint-scoped DiscoverBrowse "View all" pages are not restorable.
        ContentView::Home => Some("home"),
        ContentView::Favorites => Some("favorites"),
        ContentView::LocalLibrary => Some("local-library"),
        ContentView::Mixtapes => Some("mixtapes"),
        ContentView::Collections => Some("collections"),
        _ => None,
    }
}

