//! Whole-tab apply helpers: the cached-blob paint and the Weekly-rows
//! (re)build that rides both the cached and the full-build paths.

use qbz_external_reco::{build_weekly_exploration, build_weekly_jams, ExternalCarousels, RecoInputs};

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::apply_albums::apply_albums;
use super::apply_artists_tracks::{apply_artists, apply_tracks};
use super::artist_rails::apply_artist_rails;
use super::row_kinds::{AlbumRow, ArtistRow, TrackRow};

/// Paint the NON-weekly rows from a cached 48h blob (empty rows self-hide). The
/// two Weekly rows are intentionally NOT painted here — they are (re)built from
/// their own per-week cache by `build_and_apply_weeklies`, so the blob can never
/// pin a stale/empty weekly for the 48h window.
pub(super) fn apply_all(weak: &slint::Weak<AppWindow>, cache: &ImageCache, r: ExternalCarousels) {
    apply_artist_rails(weak, cache, r.rec_artists_common, r.rec_artists_recent);
    apply_albums(weak, cache, r.rec_albums, AlbumRow::RecAlbums);
    apply_albums(weak, cache, r.fresh_releases, AlbumRow::FreshReleases);
    apply_albums(weak, cache, r.deep_cut_albums, AlbumRow::DeepCuts);
    apply_albums(weak, cache, r.top_albums, AlbumRow::TopAlbums);
    apply_artists(weak, cache, r.top_artists, ArtistRow::TopArtists);
}

/// Build + paint the two Weekly rows from their own per-week cache (cheap on a
/// hit; one ListenBrainz `createdfor` call + a SQLite read). Used on the
/// instant-paint path so the weeklies follow ListenBrainz's weekly cadence
/// independently of the 48h results blob. The full-build path paints them via
/// its own `b_explore`/`b_jams` branches, which call the same cache-backed
/// builders.
pub(super) async fn build_and_apply_weeklies(
    inputs: &RecoInputs<'_>,
    weak: &slint::Weak<AppWindow>,
    image_cache: &ImageCache,
) {
    if inputs.listenbrainz.is_none() {
        return;
    }
    let (explore, jams) =
        tokio::join!(build_weekly_exploration(inputs), build_weekly_jams(inputs));
    apply_tracks(weak, image_cache, explore, TrackRow::WeeklyExploration);
    apply_tracks(weak, image_cache, jams, TrackRow::WeeklyJams);
}
