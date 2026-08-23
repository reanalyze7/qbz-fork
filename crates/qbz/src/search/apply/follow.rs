use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, ExternalRecoState, ForYouState, HomeState, LabelState, SearchState, SlimItem};

/// Mark an artist as followed in every `SearchState` list it appears in
/// (results list + most-popular hero). Runs on the Slint event loop.
pub fn mark_artist_followed(window: &AppWindow, artist_id: &str, following: bool) {
    // Flip the Follow chip on EVERY visible artist-card surface, so following
    // from any of them updates the others (Search, For You, the Recommendations
    // carousels). "auto"-mode chips hide once following; "toggle"-mode flip.
    let state = window.global::<SearchState>();
    set_slim_following(&state.get_artists(), artist_id, following);
    set_slim_following(&state.get_artists_carousel(), artist_id, following);
    if state.get_most_popular_kind() == "artist" {
        let mut mp = state.get_most_popular_artist();
        if mp.id == artist_id {
            mp.following = following;
            state.set_most_popular_artist(mp);
        }
    }
    let foryou = window.global::<ForYouState>();
    set_slim_following(&foryou.get_top_artists(), artist_id, following);
    set_slim_following(&foryou.get_artists_to_follow(), artist_id, following);
    let reco = window.global::<ExternalRecoState>();
    set_slim_following(&reco.get_rec_artists_common(), artist_id, following);
    set_slim_following(&reco.get_rec_artists_recent(), artist_id, following);
    set_slim_following(&reco.get_top_artists(), artist_id, following);
    // Home "Top artists" carousel + label-page related artists — walked by the
    // pin twin (`set_artist_row_pinned`) but historically missed here.
    set_slim_following(&window.global::<HomeState>().get_top_artists(), artist_id, following);
    set_slim_following(&window.global::<LabelState>().get_artists(), artist_id, following);
    // Scene-discovery grid (ArtistsByLocationView) — SlimItem rows since the
    // ArtistGridCard migration.
    set_slim_following(
        &window.global::<crate::LocationViewState>().get_artists(),
        artist_id,
        following,
    );
    // Pinned mixed carousel (Home / For You) — nested artist SlimItem.
    crate::set_pinned_artist_following(window, artist_id, following);
}

/// Flip `following` on the row matching `artist_id` in a `[SlimItem]` model.
pub(crate) fn set_slim_following(model: &ModelRc<SlimItem>, artist_id: &str, following: bool) {
    if let Some(vm) = model.as_any().downcast_ref::<VecModel<SlimItem>>() {
        for i in 0..vm.row_count() {
            if let Some(mut item) = vm.row_data(i) {
                if item.id == artist_id {
                    item.following = following;
                    vm.set_row_data(i, item);
                }
            }
        }
    }
}
