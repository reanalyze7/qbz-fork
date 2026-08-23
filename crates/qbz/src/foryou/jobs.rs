//! Per-section artwork job builders.

use crate::artwork::{ArtworkJob, ArtworkTarget};

use super::{AlbumCard, ArtistSlim, SpotlightData, TrackSlim};

pub(super) fn album_jobs(
    cards: &[AlbumCard],
    target: impl Fn(usize) -> ArtworkTarget,
) -> Vec<ArtworkJob> {
    cards
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.artwork_url.is_empty())
        .map(|(i, c)| ArtworkJob {
            url: c.artwork_url.clone(),
            target: target(i),
        })
        .collect()
}

pub(super) fn artist_jobs(
    artists: &[ArtistSlim],
    target: impl Fn(usize) -> ArtworkTarget,
) -> Vec<ArtworkJob> {
    artists
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.artwork_url.is_empty())
        .map(|(i, a)| ArtworkJob {
            url: a.artwork_url.clone(),
            target: target(i),
        })
        .collect()
}

pub(super) fn track_jobs(tracks: &[TrackSlim]) -> Vec<ArtworkJob> {
    tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.artwork_url.is_empty())
        .map(|(i, t)| ArtworkJob {
            url: t.artwork_url.clone(),
            target: ArtworkTarget::ForYouRecentTrack { index: i },
        })
        .collect()
}

pub(super) fn spotlight_jobs(sp: &SpotlightData) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    if !sp.image_url.is_empty() {
        jobs.push(ArtworkJob {
            url: sp.image_url.clone(),
            target: ArtworkTarget::ForYouSpotlightArtist,
        });
    }
    for (i, c) in sp.albums.iter().enumerate() {
        if !c.artwork_url.is_empty() {
            jobs.push(ArtworkJob {
                url: c.artwork_url.clone(),
                target: ArtworkTarget::ForYouSpotlightAlbum { index: i },
            });
        }
    }
    jobs
}
