//! Artwork download job builders for the search results page.

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::search::pagination::MoreRows;
use crate::search::rows::{MostPopularRow, SearchData};

/// Cover-download jobs for an album/track/artist row at `idx`.
pub(crate) fn simple_job(target: ArtworkTarget, url: &str) -> Option<ArtworkJob> {
    (!url.is_empty()).then(|| ArtworkJob {
        target,
        url: url.to_string(),
    })
}

/// Playlist collage jobs — one per cover URL the row carries.
fn playlist_jobs(idx: usize, urls: &[String], jobs: &mut Vec<ArtworkJob>) {
    for (slot, url) in urls.iter().enumerate().take(4) {
        if !url.is_empty() {
            jobs.push(ArtworkJob {
                target: ArtworkTarget::SearchPlaylistCover { idx, slot },
                url: url.clone(),
            });
        }
    }
}

/// Build artwork download jobs for a freshly applied `SearchData`.
pub fn artwork_jobs(data: &SearchData) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    for (idx, row) in data.albums.iter().enumerate() {
        jobs.extend(simple_job(ArtworkTarget::SearchAlbum { idx }, &row.artwork_url));
    }
    for (idx, row) in data.tracks.iter().enumerate() {
        jobs.extend(simple_job(ArtworkTarget::SearchTrack { idx }, &row.artwork_url));
    }
    for (idx, row) in data.artists.iter().enumerate() {
        jobs.extend(simple_job(ArtworkTarget::SearchArtist { idx }, &row.artwork_url));
    }
    for (idx, row) in data.playlists.iter().enumerate() {
        playlist_jobs(idx, &row.cover_urls, &mut jobs);
    }
    let mp_url = match &data.most_popular {
        MostPopularRow::Album(r) => r.artwork_url.as_str(),
        MostPopularRow::Artist(r) => r.artwork_url.as_str(),
        MostPopularRow::Track(r) => r.artwork_url.as_str(),
        MostPopularRow::None => "",
    };
    jobs.extend(simple_job(ArtworkTarget::SearchMostPopular, mp_url));
    jobs
}

/// Build artwork jobs for a load-more page, targeting the rows that were
/// just appended (`start` is the index of the first appended row).
pub fn artwork_jobs_for_more(more: &MoreRows, start: usize) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    match more {
        MoreRows::Albums(rows) => {
            for (i, row) in rows.iter().enumerate() {
                jobs.extend(simple_job(
                    ArtworkTarget::SearchAlbum { idx: start + i },
                    &row.artwork_url,
                ));
            }
        }
        MoreRows::Tracks(rows) => {
            for (i, row) in rows.iter().enumerate() {
                jobs.extend(simple_job(
                    ArtworkTarget::SearchTrack { idx: start + i },
                    &row.artwork_url,
                ));
            }
        }
        MoreRows::Artists(rows) => {
            for (i, row) in rows.iter().enumerate() {
                jobs.extend(simple_job(
                    ArtworkTarget::SearchArtist { idx: start + i },
                    &row.artwork_url,
                ));
            }
        }
        MoreRows::Playlists(rows) => {
            for (i, row) in rows.iter().enumerate() {
                playlist_jobs(start + i, &row.cover_urls, &mut jobs);
            }
        }
    }
    jobs
}
