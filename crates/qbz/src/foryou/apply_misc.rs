//! Per-section apply helpers (rediscover, favorite albums, most played
//! albums, more from library, spotlight). See `apply_sections.rs` for the
//! shared conventions (paint on UI thread, then fire artwork jobs; none
//! touch the `loaded` flag).

use slint::ModelRc;
use slint::VecModel;

use crate::artwork::{ArtworkTarget, ImageCache};
use crate::{AppWindow, ForYouState};

use super::jobs::{album_jobs, spotlight_jobs};
use super::mappers::{album_items, section};
use super::{AlbumCard, SpotlightData};

pub(super) fn apply_rediscover(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    cards: Vec<AlbumCard>,
) {
    let jobs = album_jobs(&cards, |i| ArtworkTarget::ForYouRediscover { index: i });
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        w.global::<ForYouState>()
            .set_rediscover(section(&qbz_i18n::t("Rediscover Your Library"), &cards));
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_favorite_albums(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    cards: Vec<AlbumCard>,
) {
    let jobs = album_jobs(&cards, |i| ArtworkTarget::ForYouFavoriteAlbum { index: i });
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        w.global::<ForYouState>()
            .set_favorite_albums(section(&qbz_i18n::t("Library Albums"), &cards));
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_most_played_albums(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    cards: Vec<AlbumCard>,
) {
    let jobs = album_jobs(&cards, |i| ArtworkTarget::ForYouMostPlayedAlbum { index: i });
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        w.global::<ForYouState>()
            .set_most_played_albums(section(&qbz_i18n::t("Most Played Albums"), &cards));
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_more_from_library(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    cards: Vec<AlbumCard>,
    seed_title: String,
) {
    // 1:1 with Tauri's `discovery.similarTo` = "Similar to {seed}", where the
    // seed is the album the suggestions are seeded from. Falls back to the
    // plain "More From Your Library" when there is no seed (never titleless).
    let title = if seed_title.is_empty() {
        qbz_i18n::t("More From Your Library")
    } else {
        qbz_i18n::t_args("Similar to {}", &[&seed_title])
    };
    let jobs = album_jobs(&cards, |i| ArtworkTarget::ForYouMoreFromLibrary { index: i });
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        w.global::<ForYouState>()
            .set_more_from_library(section(&title, &cards));
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_spotlight(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    sp: Option<SpotlightData>,
) {
    let jobs = sp.as_ref().map(spotlight_jobs).unwrap_or_default();
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        let state = w.global::<ForYouState>();
        if let Some(sp) = &sp {
            state.set_spotlight_visible(true);
            state.set_spotlight_artist_id(sp.artist_id.clone().into());
            state.set_spotlight_name(sp.artist_name.clone().into());
            state.set_spotlight_category(sp.category.clone().into());
            state.set_spotlight_image_url(sp.image_url.clone().into());
            state.set_spotlight_has_top_tracks(sp.has_top_tracks);
            state.set_spotlight_albums(ModelRc::new(VecModel::from(album_items(&sp.albums))));
        } else {
            state.set_spotlight_visible(false);
        }
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}
