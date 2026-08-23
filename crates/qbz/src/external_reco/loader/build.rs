//! The cache-miss / stale full build: the cold-start editorial fallback, or
//! the six-way progressive build (each branch paints its own row the moment
//! it resolves).

use std::sync::{Arc, Mutex};

use qbz_external_reco::{
    build_deep_cut_albums, build_editorial, build_fresh_releases, build_rec_albums,
    build_rec_artists_common, build_rec_artists_recent, build_weekly_exploration,
    build_weekly_jams, gather_history, ExternalCarousels, RecoInputs,
};

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::super::apply_albums::apply_albums;
use super::super::apply_artists_tracks::{apply_artists, apply_tracks};
use super::super::artist_rails::apply_artist_rails;
use super::super::row_kinds::{AlbumRow, ArtistRow, TrackRow};

/// Run the appropriate build path (editorial cold-start vs the six-way
/// progressive build), painting each row as it resolves, and return the full
/// collected result for the results-cache write.
pub(super) async fn build_full(
    inputs: &RecoInputs<'_>,
    weak: &slint::Weak<AppWindow>,
    image_cache: &ImageCache,
    cold_start: bool,
) -> ExternalCarousels {
    let collector: Arc<Mutex<ExternalCarousels>> = Arc::new(Mutex::new(ExternalCarousels::default()));

    if cold_start {
        let (albums, artists) = build_editorial(inputs).await;
        if let Ok(mut g) = collector.lock() {
            g.editorial_fallback = true;
            g.top_albums = albums.clone();
            g.top_artists = artists.clone();
        }
        apply_albums(weak, image_cache, albums, AlbumRow::TopAlbums);
        apply_artists(weak, image_cache, artists, ArtistRow::TopArtists);
    } else {
        let history = gather_history(inputs).await;
        let col = &collector;
        // Progressive: each branch paints its row AND collects it for the cache.
        // The two artist rails build in parallel but paint TOGETHER through
        // apply_artist_rails (the shared filter/dedup/backfill choke point —
        // cross-rail dedup needs both pools). The collector stores the FULL
        // validated pools (visible + overflow), so the results blob carries
        // the backfill candidates too.
        let b_artists = async {
            let (common_pool, recent_pool) = tokio::join!(
                build_rec_artists_common(inputs, &history),
                build_rec_artists_recent(inputs, &history),
            );
            if let Ok(mut g) = col.lock() {
                g.rec_artists_common = common_pool.clone();
                g.rec_artists_recent = recent_pool.clone();
            }
            apply_artist_rails(weak, image_cache, common_pool, recent_pool);
        };
        let b_albums = async {
            let r = build_rec_albums(inputs, &history).await;
            if let Ok(mut g) = col.lock() {
                g.rec_albums = r.clone();
            }
            apply_albums(weak, image_cache, r, AlbumRow::RecAlbums);
        };
        let b_fresh = async {
            let r = build_fresh_releases(inputs).await;
            if let Ok(mut g) = col.lock() {
                g.fresh_releases = r.clone();
            }
            apply_albums(weak, image_cache, r, AlbumRow::FreshReleases);
        };
        let b_explore = async {
            let r = build_weekly_exploration(inputs).await;
            if let Ok(mut g) = col.lock() {
                g.weekly_exploration = r.clone();
            }
            apply_tracks(weak, image_cache, r, TrackRow::WeeklyExploration);
        };
        let b_jams = async {
            let r = build_weekly_jams(inputs).await;
            if let Ok(mut g) = col.lock() {
                g.weekly_jams = r.clone();
            }
            apply_tracks(weak, image_cache, r, TrackRow::WeeklyJams);
        };
        let b_deep = async {
            let r = build_deep_cut_albums(inputs).await;
            if let Ok(mut g) = col.lock() {
                g.deep_cut_albums = r.clone();
            }
            apply_albums(weak, image_cache, r, AlbumRow::DeepCuts);
        };
        tokio::join!(b_artists, b_albums, b_fresh, b_explore, b_jams, b_deep);
    }

    // Only `col = &collector` references were ever taken above (never
    // cloned), so exactly one strong ref remains here.
    collector
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|e| e.into_inner().clone())
}
