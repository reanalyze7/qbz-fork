//! `to_item` mapping, `apply_library_all`, and the artwork-job builder.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, LibraryAllState, LibraryFeedItem};

use super::derive::derive;
use super::feed::Feed;

fn to_item(f: &Feed) -> LibraryFeedItem {
    LibraryFeedItem {
        kind: f.kind.clone().into(),
        group: f.group.clone().into(),
        source: f.source.clone().into(),
        id: f.id.clone().into(),
        title: f.title.clone().into(),
        subtitle: f.subtitle.clone().into(),
        artist: f.artist.clone().into(),
        artist_id: f.artist_id.clone().into(),
        album: f.album.clone().into(),
        album_id: f.album_id.clone().into(),
        image_url: f.image_url.clone().into(),
        image: slint::Image::default(),
        quality_tier: f.quality_tier.clone().into(),
        quality_detail: f.quality_detail.clone().into(),
        is_favorite: f.is_favorite,
        removing: false,
        sort_title: f.title.to_lowercase().into(),
        sort_artist: f.artist.to_lowercase().into(),
        genre: f.genre.to_lowercase().into(),
        playlist_owned: f.playlist_owned,
        playlist_following: f.playlist_following,
        playlist_copied: f.playlist_copied,
    }
}

/// Push the full merged feed into `LibraryAllState` and derive the first view.
pub fn apply_library_all(window: &AppWindow, feed: Vec<Feed>) {
    let items: Vec<LibraryFeedItem> = feed.iter().map(to_item).collect();
    let total = items.len() as i32;
    let st = window.global::<LibraryAllState>();
    st.set_items(ModelRc::new(VecModel::from(items)));
    st.set_total(total);
    st.set_loading(false);
    st.set_load_error("".into());
    derive(window);
}

/// Build cover-download jobs for the CURRENT visible feed. Call after apply and
/// after every derive (the ImageCache dedups already-decoded covers, so
/// re-dispatching on filter/sort is cheap). Indices target `items-visible`.
pub fn artwork_jobs(window: &AppWindow) -> Vec<ArtworkJob> {
    let visible = window.global::<LibraryAllState>().get_items_visible();
    let mut jobs = Vec::new();
    for i in 0..visible.row_count() {
        if let Some(item) = visible.row_data(i) {
            let url = item.image_url.to_string();
            if !url.is_empty() {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::LibraryAllCover { index: i },
                    url,
                });
            }
        }
    }
    jobs
}
