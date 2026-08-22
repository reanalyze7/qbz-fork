//! Per-row builder wrappers (thin delegations to `carousels`) plus the
//! convenience combinator that builds the whole set at once.

use crate::types::{AlbumReco, ArtistReco, ExternalCarousels, ExtHistory, TrackReco};
use crate::{carousels, gather_history, is_cold_start, RecoInputs};

pub async fn build_rec_artists_common(
    inputs: &RecoInputs<'_>,
    history: &ExtHistory,
) -> Vec<ArtistReco> {
    carousels::build_rec_artists_common(inputs, history).await
}
pub async fn build_rec_artists_recent(
    inputs: &RecoInputs<'_>,
    history: &ExtHistory,
) -> Vec<ArtistReco> {
    carousels::build_rec_artists_recent(inputs, history).await
}
pub async fn build_rec_albums(inputs: &RecoInputs<'_>, history: &ExtHistory) -> Vec<AlbumReco> {
    carousels::build_rec_albums(inputs, history).await
}
pub async fn build_fresh_releases(inputs: &RecoInputs<'_>) -> Vec<AlbumReco> {
    carousels::build_fresh_releases(inputs).await
}
pub async fn build_weekly_exploration(inputs: &RecoInputs<'_>) -> Vec<TrackReco> {
    carousels::build_weekly(inputs, "weekly-exploration").await
}
pub async fn build_weekly_jams(inputs: &RecoInputs<'_>) -> Vec<TrackReco> {
    carousels::build_weekly(inputs, "weekly-jams").await
}
pub async fn build_deep_cut_albums(inputs: &RecoInputs<'_>) -> Vec<AlbumReco> {
    carousels::build_deep_cut_albums(inputs).await
}
/// Album page: albums similar to a seed album, derived from its primary
/// artist's Last.fm similar artists (one top album each). `exclude_pairs` are
/// the (artist, title) already shown by the Qobuz suggestions row.
pub async fn build_similar_albums_seeded(
    inputs: &RecoInputs<'_>,
    seed_artist: &str,
    exclude_pairs: &[(String, String)],
) -> Vec<AlbumReco> {
    carousels::build_similar_albums_seeded(inputs, seed_artist, exclude_pairs).await
}
/// Cold-start editorial (top albums + artists).
pub async fn build_editorial(inputs: &RecoInputs<'_>) -> (Vec<AlbumReco>, Vec<ArtistReco>) {
    carousels::build_editorial(inputs).await
}

/// Convenience: build the whole set at once (non-progressive callers / tests).
pub async fn build_external_carousels(inputs: RecoInputs<'_>) -> ExternalCarousels {
    if is_cold_start(&inputs) {
        let (top_albums, top_artists) = build_editorial(&inputs).await;
        return ExternalCarousels {
            editorial_fallback: true,
            top_albums,
            top_artists,
            ..Default::default()
        };
    }
    let history = gather_history(&inputs).await;
    let (
        rec_artists_common,
        rec_artists_recent,
        rec_albums,
        fresh_releases,
        weekly_exploration,
        weekly_jams,
        deep_cut_albums,
    ) = tokio::join!(
        build_rec_artists_common(&inputs, &history),
        build_rec_artists_recent(&inputs, &history),
        build_rec_albums(&inputs, &history),
        build_fresh_releases(&inputs),
        build_weekly_exploration(&inputs),
        build_weekly_jams(&inputs),
        build_deep_cut_albums(&inputs),
    );
    ExternalCarousels {
        editorial_fallback: false,
        rec_artists_common,
        rec_artists_recent,
        rec_albums,
        fresh_releases,
        weekly_exploration,
        weekly_jams,
        deep_cut_albums,
        top_albums: Vec::new(),
        top_artists: Vec::new(),
    }
}
