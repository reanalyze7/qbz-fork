use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::favorites::fetch::FavData;
use crate::search::PlaylistRow;

/// Artwork jobs for the freshly loaded tab.
pub fn artwork_jobs(data: &FavData) -> Vec<ArtworkJob> {
    match data {
        FavData::Tracks { items, .. } => items
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.artwork_url.is_empty())
            .map(|(i, t)| ArtworkJob {
                url: t.artwork_url.clone(),
                target: ArtworkTarget::FavoriteTrack { index: i },
            })
            .collect(),
        // WINDOWED (mirrors the Local Library albums grid): no all-at-once
        // jobs for the full set — covers are dispatched by the viewport-band
        // dispatchers after apply/derive (see `begin_albums_artwork`).
        FavData::Albums { .. } => Vec::new(),
        FavData::Artists { items, .. } => items
            .iter()
            .enumerate()
            .filter(|(_, a)| !a.image_url.is_empty())
            .map(|(i, a)| ArtworkJob {
                url: a.image_url.clone(),
                target: ArtworkTarget::FavoriteArtist { index: i },
            })
            .collect(),
        FavData::Playlists { favorites, following } => {
            fn push(rows: &[PlaylistRow], following: bool, jobs: &mut Vec<ArtworkJob>) {
                for (index, row) in rows.iter().enumerate() {
                    for (slot, url) in row.cover_urls.iter().enumerate().take(4) {
                        if !url.is_empty() {
                            jobs.push(ArtworkJob {
                                url: url.clone(),
                                target: ArtworkTarget::FavPlaylistCover { following, index, slot },
                            });
                        }
                    }
                }
            }
            let mut jobs: Vec<ArtworkJob> = Vec::new();
            push(favorites, false, &mut jobs);
            push(following, true, &mut jobs);
            jobs
        }
        FavData::Labels { items, .. } => items
            .iter()
            .enumerate()
            .filter(|(_, l)| !l.image_url.is_empty())
            .map(|(i, l)| ArtworkJob {
                url: l.image_url.clone(),
                target: ArtworkTarget::FavoriteLabel { index: i },
            })
            .collect(),
    }
}
