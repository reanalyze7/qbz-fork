//! Per-section apply helpers (recent, release watch, top artists, to
//! follow). Each pushes its model on the UI thread, then fires its artwork
//! jobs (async, per-row). NONE of them touches the `loaded` flag — that is
//! latched once at the end of `spawn_for_you`.
use slint::ComponentHandle;

use slint::ModelRc;
use slint::VecModel;

use crate::artwork::{ArtworkTarget, ImageCache};
use crate::{AppWindow, ForYouState, SlimItem};

use super::jobs::{album_jobs, artist_jobs, track_jobs};
use super::mappers::{artist_items, section};
use super::{AlbumCard, ArtistSlim, TrackSlim};

pub(super) fn apply_recent(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    albums: Vec<AlbumCard>,
    tracks: Vec<TrackSlim>,
) {
    let mut jobs = album_jobs(&albums, |i| ArtworkTarget::ForYouRecentAlbum { index: i });
    jobs.extend(track_jobs(&tracks));
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        let state = w.global::<ForYouState>();
        state.set_recent_albums(section(&qbz_i18n::t("Recently Played Albums"), &albums));
        let slim: Vec<SlimItem> = tracks
            .iter()
            .map(|t| SlimItem {
                id: t.id.clone().into(),
                title: t.title.clone().into(),
                subtitle: t.subtitle.clone().into(),
                rank: "".into(),
                artwork_url: t.artwork_url.clone().into(),
                artwork: slint::Image::default(),
                following: false,
                // Track slims render pin-less rows — tracks are not pinnable.
                is_pinned: false,
            })
            .collect();
        state.set_recent_tracks(ModelRc::new(VecModel::from(slim)));
    });
    // Recently-played albums/tracks mix sources (Qobuz / local), so the
    // artwork must be routed by scheme: http -> Qobuz CDN, else a local file
    // read. The plain HTTP loader (spawn_loads) left local covers blank.
    crate::artwork::spawn_search_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_release_watch(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    cards: Vec<AlbumCard>,
) {
    let jobs = album_jobs(&cards, |i| ArtworkTarget::ForYouReleaseWatch { index: i });
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        w.global::<ForYouState>()
            .set_release_watch(section(&qbz_i18n::t("Release Watch"), &cards));
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_top_artists(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    artists: Vec<ArtistSlim>,
) {
    let jobs = artist_jobs(&artists, |i| ArtworkTarget::ForYouTopArtist { index: i });
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        w.global::<ForYouState>()
            .set_top_artists(ModelRc::new(VecModel::from(artist_items(&artists))));
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}

pub(super) fn apply_to_follow(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    artists: Vec<ArtistSlim>,
) {
    let jobs = artist_jobs(&artists, |i| ArtworkTarget::ForYouToFollow { index: i });
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        w.global::<ForYouState>()
            .set_artists_to_follow(ModelRc::new(VecModel::from(artist_items(&artists))));
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}
