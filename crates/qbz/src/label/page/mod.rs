//! `LabelPageView` — the rich label landing page: header + popular tracks
//! + releases/critics/playlists/artists/more-labels carousels. Mirrors
//! Tauri's LabelView.svelte. Data comes from /label/page
//! (top_tracks/releases/playlists/top_artists), the first /label/getAlbums
//! page (releases carousel), /label/explore (more labels), and the user's
//! favorite-labels set (follow state).

mod apply;
mod follow;
mod jump_tabs;
mod load;
mod load_sections;
mod parse;
mod parse_track;
mod reset;
mod selection;
mod to_slint;
mod value_helpers;

use std::cell::RefCell;

use qbz_models::Track;

pub use apply::{apply_label_page, apply_label_library, page_artwork_jobs};
pub use follow::{label_following_state, mark_label_followed, more_label_name};
pub use load::load_label_page;
pub use reset::reset_label_page;
pub use selection::{clear_selection, recount_selected, select_all, selected_ids, selected_play_tracks, set_multi_select};

/// Plain, `Send` payload for the rich label landing page.
pub struct LabelPagePayload {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub description: String,
    pub description_short: String,
    pub description_truncated: bool,
    pub is_following: bool,
    pub top_tracks: Vec<TopTrack>,
    pub releases: Vec<crate::album_map::AlbumCard>,
    pub critics: Vec<crate::album_map::AlbumCard>,
    pub playlists: Vec<PlaylistSlim>,
    pub artists: Vec<ArtistSlim>,
    pub more_labels: Vec<LabelSlim>,
    /// Catalog tracks kept for "Play all" — deserialized from the page
    /// top_tracks and queued verbatim (mirrors Tauri's buildTopTracksQueue).
    pub play_tracks: Vec<Track>,
}

#[derive(Clone)]
pub struct TopTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub album_id: String,
    pub album: String,
    pub artwork_url: String,
    pub duration: String,
    pub quality_tier: String,
    pub quality_detail: String,
}

#[derive(Clone)]
pub struct PlaylistSlim {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub image_url: String,
}

#[derive(Clone)]
pub struct ArtistSlim {
    pub id: String,
    pub name: String,
    pub image_url: String,
}

#[derive(Clone)]
pub struct LabelSlim {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub following: bool,
}

// Catalog tracks for the landing's "Play all", cached on the UI thread
// (set in `apply_label_page`, read by the play-top media action).
thread_local! {
    pub(super) static PLAY_TOP_TRACKS: RefCell<Vec<Track>> = const { RefCell::new(Vec::new()) };
}

/// The label's popular tracks as a play-ready queue source.
pub fn top_tracks_for_play() -> Vec<Track> {
    PLAY_TOP_TRACKS.with(|c| c.borrow().clone())
}
