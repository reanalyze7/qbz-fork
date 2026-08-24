use crate::*;

/// Flip the `is-favorite` flag on every visible row matching `track_id`,
/// across all track-list surfaces (album, artist Popular, search,
/// playlist, mix, favorites). Used for the optimistic favorite toggle so
/// the heart updates the instant the user clicks, regardless of which
/// view they are on.
pub(crate) fn set_row_favorite(window: &AppWindow, track_id: &str, favorite: bool) {
    let flip = |model: &slint::ModelRc<TrackItem>| {
        if let Some(vm) = model.as_any().downcast_ref::<slint::VecModel<TrackItem>>() {
            for i in 0..vm.row_count() {
                if let Some(mut item) = vm.row_data(i) {
                    if item.id == track_id {
                        if item.is_favorite != favorite {
                            item.is_favorite = favorite;
                            vm.set_row_data(i, item);
                        }
                    }
                }
            }
        }
    };
    flip(&window.global::<AlbumState>().get_tracks());
    flip(&window.global::<ArtistState>().get_top_tracks());
    flip(&window.global::<SearchState>().get_tracks());
    flip(&window.global::<PlaylistState>().get_tracks());
    flip(&window.global::<MixState>().get_tracks());
    flip(&window.global::<FavoritesState>().get_tracks());

    // Search's most-popular track hero is a standalone TrackItem.
    let search = window.global::<SearchState>();
    let mut hero = search.get_most_popular_track();
    if hero.id == track_id && hero.is_favorite != favorite {
        hero.is_favorite = favorite;
        search.set_most_popular_track(hero);
    }
}

