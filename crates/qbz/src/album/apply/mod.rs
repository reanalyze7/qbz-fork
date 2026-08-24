//! Apply loaded album data to the `AlbumState` Slint global — the "render"
//! half of the controller (counterpart of `map.rs`).

mod artwork;
mod external_links;
mod tracks;

pub use artwork::apply_artwork;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AlbumState, AppWindow, ArtistCredit};

use super::data::AlbumData;
use super::{FULL_TRACKS, PLAY_TRACKS};
use external_links::apply_external_links;
use tracks::build_track_items;

/// Apply album data to the `AlbumState` global. Runs on the Slint event loop.
pub fn apply_album(window: &AppWindow, data: AlbumData) {
    // These rows belong to the album currently being viewed, so the
    // album link target is this album's own id (the album column is not
    // shown here, but album-id keeps the row model complete).
    let album_id: slint::SharedString = data.id.clone().into();
    let tracks = build_track_items(data.tracks, &album_id, &data.artist_id);

    let has_custom_cover = crate::custom_artwork::album_cover(&data.id).is_some();
    let artwork_url = data.artwork_url.clone();

    let state = window.global::<AlbumState>();
    state.set_id(data.id.into());
    state.set_title(data.title.into());
    state.set_artwork_url(artwork_url.into());
    state.set_has_custom_cover(has_custom_cover);
    state.set_artist(data.artist.into());
    state.set_artist_id(data.artist_id.into());
    let credits: Vec<ArtistCredit> = data
        .artists
        .into_iter()
        .map(|c| ArtistCredit {
            id: c.id.into(),
            name: c.name.into(),
            role: c.role.into(),
        })
        .collect();
    state.set_artists(ModelRc::new(VecModel::from(credits)));
    state.set_info_line(data.info_line.into());
    state.set_meta_pre(data.meta_pre.into());
    state.set_meta_post(data.meta_post.into());
    state.set_quality_tier(data.quality_tier.into());
    state.set_quality_detail(data.quality_detail.into());
    state.set_description(data.description.into());
    state.set_description_short(data.description_short.into());
    state.set_description_shorter(data.description_shorter.into());
    state.set_label(data.label.into());
    state.set_label_id(data.label_id.into());
    state.set_has_booklet(data.has_booklet);
    // Stash the booklet goody URL for the reader controller; cleared on reset.
    crate::booklet::set_current_url(&data.booklet_url);

    let ext_artist = state.get_artist().to_string();
    let ext_title = state.get_title().to_string();
    apply_external_links(&state, &ext_artist, &ext_title);

    // Fully cached = every track already has a ready (3) offline copy. Kept
    // live afterwards by set_row_cache_status as downloads complete.
    let album_fully_cached =
        !tracks.is_empty() && tracks.iter().all(|t| t.cache_status == 3);
    state.set_album_fully_cached(album_fully_cached);
    // Seed the header heart from the favorite-album cache (kept in sync with
    // the server at login + on every toggle).
    state.set_is_favorite(crate::fav_cache::is_album_favorite(album_id.as_str()));
    state.set_is_album_blocked(crate::artist_blacklist::is_album_blacklisted(album_id.as_str()));
    // Seed the pin state from the pinned store (Home "Pinned" section).
    state.set_pinned(crate::pinned::is_pinned("album", album_id.as_str()));
    state.set_favorite_loading(false);

    // Keep the unfiltered list for the track search + the raw tracks for the
    // multi-select bulk actions, then show them all.
    FULL_TRACKS.with(|cell| *cell.borrow_mut() = tracks.clone());
    PLAY_TRACKS.with(|cell| *cell.borrow_mut() = data.raw_tracks);
    // A freshly loaded album starts out of select mode with nothing selected.
    state.set_multi_select(false);
    state.set_selected_count(0);
    state.set_tracks(ModelRc::new(VecModel::from(tracks)));
}
