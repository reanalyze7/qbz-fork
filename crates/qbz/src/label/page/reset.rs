use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AlbumCardItem, AppWindow, DiscoverSection, JumpNavTab, LabelState, SearchPlaylistItem, SlimItem, TrackItem};

/// Clear the landing state before loading a new label.
pub fn reset_label_page(window: &AppWindow) {
    let state = window.global::<LabelState>();
    state.set_name("".into());
    state.set_image_url("".into());
    state.set_image(slint::Image::default());
    state.set_description("".into());
    state.set_description_short("".into());
    state.set_description_truncated(false);
    state.set_is_following(false);
    state.set_top_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_releases_section(DiscoverSection::default());
    state.set_critics_section(DiscoverSection::default());
    state.set_playlists(ModelRc::new(VecModel::from(Vec::<SearchPlaylistItem>::new())));
    state.set_artists(ModelRc::new(VecModel::from(Vec::<SlimItem>::new())));
    state.set_more_labels(ModelRc::new(VecModel::from(Vec::<SlimItem>::new())));
    state.set_jump_tabs(ModelRc::new(VecModel::from(Vec::<JumpNavTab>::new())));
    // Catalog/library toggle — clear the previous label's library state so
    // the toggle never lingers on a label with no library items.
    state.set_label_tab("catalog".into());
    state.set_library_count(0);
    state.set_library_albums(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    state.set_library_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_page_loaded(false);
    state.set_loading(true);
}
