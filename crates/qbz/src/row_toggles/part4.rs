use crate::*;

/// Playlist counterpart of [`set_album_row_pinned`]: flip the `is-pinned`
/// badge on every visible playlist card matching `playlist_id` (Home rail,
/// Qobuz Playlists browse, Search, Favorites, label landings).
pub(crate) fn set_playlist_row_pinned(window: &AppWindow, playlist_id: &str, pinned: bool) {
    let flip = |model: &slint::ModelRc<SearchPlaylistItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == playlist_id && item.is_pinned != pinned {
                    item.is_pinned = pinned;
                    model.set_row_data(i, item);
                }
            }
        }
    };
    flip(&window.global::<HomeState>().get_playlists());
    flip(&window.global::<SearchState>().get_playlists());
    let browse = window.global::<PlaylistBrowseState>();
    flip(&browse.get_playlists());
    flip(&browse.get_visible());
    let favs = window.global::<FavoritesState>();
    flip(&favs.get_playlists_favorites());
    flip(&favs.get_playlists_following());
    flip(&favs.get_playlists_visible());
    flip(&window.global::<LabelState>().get_playlists());
}

/// Artist counterpart of [`set_album_row_pinned`]: flip the `is-pinned` badge
/// on every visible ARTIST slim-card row matching `artist_id`. Only true
/// artist models are walked — track/label/award slims share `SlimItem` but
/// are not pinnable, and their ids live in other id spaces (flipping them on
/// a numeric collision would lie).
pub(crate) fn set_artist_row_pinned(window: &AppWindow, artist_id: &str, pinned: bool) {
    let flip = |model: &slint::ModelRc<SlimItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == artist_id && item.is_pinned != pinned {
                    item.is_pinned = pinned;
                    model.set_row_data(i, item);
                }
            }
        }
    };
    let search = window.global::<SearchState>();
    flip(&search.get_artists());
    flip(&search.get_artists_carousel());
    flip(&window.global::<HomeState>().get_top_artists());
    let foryou = window.global::<ForYouState>();
    flip(&foryou.get_top_artists());
    flip(&foryou.get_artists_to_follow());
    let reco = window.global::<ExternalRecoState>();
    flip(&reco.get_rec_artists_common());
    flip(&reco.get_rec_artists_recent());
    flip(&reco.get_top_artists());
    flip(&window.global::<LabelState>().get_artists());
    // Library > Favorites artists grid uses FavoriteArtistItem (not SlimItem);
    // flip it too so the pin badge updates live there (grid + A-Z sections).
    let flip_fav = |model: &slint::ModelRc<FavoriteArtistItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == artist_id && item.is_pinned != pinned {
                    item.is_pinned = pinned;
                    model.set_row_data(i, item);
                }
            }
        }
    };
    let favs = window.global::<FavoritesState>();
    flip_fav(&favs.get_artists());
    flip_fav(&favs.get_artists_visible());
    let grouped = favs.get_artists_grouped();
    for gi in 0..grouped.row_count() {
        if let Some(sec) = grouped.row_data(gi) {
            flip_fav(&sec.artists);
        }
    }
}

/// Flip the nested album's `is-favorite` on any Pinned carousel row (Home / For
/// You) matching `album_id`. The album lives inside a `PinnedItem`, so neither
/// the `[AlbumCardItem]` nor the `[DiscoverSection]` sweep reaches it.
pub(crate) fn set_pinned_album_favorite(window: &AppWindow, album_id: &str, favorite: bool) {
    let pm = window.global::<PinnedState>().get_items();
    for i in 0..pm.row_count() {
        if let Some(mut it) = pm.row_data(i) {
            if it.kind == "album" && it.album.id == album_id && it.album.is_favorite != favorite {
                it.album.is_favorite = favorite;
                pm.set_row_data(i, it);
            }
        }
    }
}

/// Flip the nested artist's `following` on any Pinned carousel row matching
/// `artist_id`. Twin of [`set_pinned_album_favorite`] for the follow chip.
/// `pub(crate)` so `search::mark_artist_followed` can reach the Pinned model.
pub(crate) fn set_pinned_artist_following(window: &AppWindow, artist_id: &str, following: bool) {
    let pm = window.global::<PinnedState>().get_items();
    for i in 0..pm.row_count() {
        if let Some(mut it) = pm.row_data(i) {
            if it.kind == "artist" && it.artist.id == artist_id && it.artist.following != following {
                it.artist.following = following;
                pm.set_row_data(i, it);
            }
        }
    }
}

