//! Validation pools (concurrent, blend-ordered, deduped).

use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};

use crate::types::{AlbumCandidate, AlbumReco, ArtistCandidate, ArtistReco, TrackCandidate, TrackReco};
use crate::validate::{validate_album, validate_artist, validate_track};
use crate::RecoCatalog;

use super::{Cache, VALIDATE_CONCURRENCY};

pub(super) async fn validate_artist_pool(
    catalog: &dyn RecoCatalog,
    cache: Cache<'_>,
    cands: Vec<ArtistCandidate>,
) -> Vec<ArtistReco> {
    let resolved: Vec<Option<ArtistReco>> = stream::iter(
        cands.into_iter().map(|cand| async move { validate_artist(catalog, cache, &cand).await }),
    )
    .buffered(VALIDATE_CONCURRENCY)
    .collect()
    .await;
    let mut seen = HashSet::new();
    resolved
        .into_iter()
        .flatten()
        .filter(|r| seen.insert(r.qobuz_artist_id))
        .collect()
}

pub(super) async fn validate_album_pool(
    catalog: &dyn RecoCatalog,
    cache: Cache<'_>,
    cands: Vec<AlbumCandidate>,
) -> Vec<AlbumReco> {
    let resolved: Vec<Option<AlbumReco>> = stream::iter(
        cands.into_iter().map(|cand| async move { validate_album(catalog, cache, &cand).await }),
    )
    .buffered(VALIDATE_CONCURRENCY)
    .collect()
    .await;
    let mut seen = HashSet::new();
    resolved
        .into_iter()
        .flatten()
        .filter(|r| seen.insert(r.qobuz_album_id.clone()))
        .collect()
}

pub(super) async fn validate_track_pool(
    catalog: &dyn RecoCatalog,
    mb: &qbz_integrations::MusicBrainzClient,
    cache: Cache<'_>,
    cands: Vec<TrackCandidate>,
    skip_negative: bool,
    skip_mb: bool,
) -> Vec<TrackReco> {
    let resolved: Vec<Option<TrackReco>> = stream::iter(cands.into_iter().map(|cand| async move {
        validate_track(catalog, mb, cache, &cand, skip_negative, skip_mb).await
    }))
    .buffered(VALIDATE_CONCURRENCY)
    .collect()
    .await;
    let mut seen = HashSet::new();
    resolved
        .into_iter()
        .flatten()
        .filter(|r| seen.insert(r.qobuz_track_id))
        .collect()
}
