//! The single-branch `ContentView` arms of `play_track_in_context`:
//! favorites, label, mix, album, artist.
use slint::ComponentHandle;

use crate::playback::context_play::{play_album_from, play_artist_top_from};
use crate::playback::queue_build::from_model::order_by_visible;
use crate::playback::queue_build::model_helpers::model_ids;
use crate::playback::queue_build::play_queue::play_tracks;
use crate::playback::Runtime;
use crate::{AlbumState, ArtistState, AppWindow, FavoritesState, LabelState};

pub(super) fn handle_favorites(
    window: &AppWindow,
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    if let Some((tracks, idx)) = order_by_visible(
        &window.global::<FavoritesState>().get_tracks_visible(),
        crate::favorites::play_tracks(),
        clicked_id,
    ) {
        play_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, idx);
        return true;
    }
    false
}

pub(super) fn handle_label(
    window: &AppWindow,
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    if let Some((tracks, idx)) = order_by_visible(
        &window.global::<LabelState>().get_top_tracks(),
        crate::label::top_tracks_for_play(),
        clicked_id,
    ) {
        let ctx_id = window.global::<LabelState>().get_id().to_string();
        crate::playback::queue_build::play_queue::play_tracks_ctx(
            runtime.clone(),
            weak.clone(),
            handle.clone(),
            tracks,
            idx,
            Some(("label".to_string(), ctx_id)),
        );
        return true;
    }
    false
}

/// Mix has no custom sort/filter, so the cache order is the
/// visible order; anchor on the clicked id.
pub(super) fn handle_mix(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    let tracks = crate::mix::current_tracks();
    if tracks.iter().any(|t| t.id.to_string() == clicked_id) {
        let idx = crate::mix::index_of(clicked_id);
        play_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, idx);
        return true;
    }
    false
}

/// Re-fetch views: build the queue from the catalog, reorder it to the
/// VISIBLE row order (so an in-page search filter is respected), and
/// start at the clicked id. (Local albums are routed earlier.)
pub(super) fn handle_album(
    window: &AppWindow,
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    let album_id = window.global::<AlbumState>().get_id().to_string();
    if album_id.is_empty() {
        return false;
    }
    let visible_ids = model_ids(&window.global::<AlbumState>().get_tracks());
    play_album_from(
        runtime.clone(),
        weak.clone(),
        handle.clone(),
        album_id,
        visible_ids,
        clicked_id.to_string(),
    );
    true
}

pub(super) fn handle_artist(
    window: &AppWindow,
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    let artist_id = window.global::<ArtistState>().get_id().to_string();
    if artist_id.is_empty() {
        return false;
    }
    let visible_ids = model_ids(&window.global::<ArtistState>().get_top_tracks());
    play_artist_top_from(
        runtime.clone(),
        weak.clone(),
        handle.clone(),
        artist_id,
        visible_ids,
        clicked_id.to_string(),
    );
    true
}
