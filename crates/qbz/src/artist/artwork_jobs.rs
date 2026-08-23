use crate::artist::data::ArtistData;
use crate::artwork::{ArtworkJob, ArtworkTarget};

/// Build artwork download jobs for every release card so the cover grid
/// fills in once the images decode.
pub fn artwork_jobs(data: &ArtistData) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    if let Some(card) = data.last_release.as_ref() {
        // The model drops a blocked last-release (apply_artist), so skip its job
        // too — otherwise the cover would land on the empty slot.
        if !card.artwork_url.is_empty()
            && !crate::artist_blacklist::card_blacklisted(&card.id, &card.artist_id)
        {
            jobs.push(ArtworkJob {
                target: ArtworkTarget::ArtistLastRelease,
                url: card.artwork_url.clone(),
            });
        }
    }
    for (section_idx, section) in data.release_sections.iter().enumerate() {
        // album_idx must be the POST-FILTER index so it matches the filtered
        // model apply_artist builds; otherwise a blocked card shifts every
        // subsequent cover onto the wrong album (and clicks open the wrong one).
        let mut album_idx = 0;
        for card in section.cards.iter() {
            if crate::artist_blacklist::card_blacklisted(&card.id, &card.artist_id) {
                continue;
            }
            if !card.artwork_url.is_empty() {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::ArtistRelease {
                        section_idx,
                        album_idx,
                    },
                    url: card.artwork_url.clone(),
                });
            }
            album_idx += 1;
        }
    }
    // "Popular Tracks" rows carry the album-cover URL but Slint can't fetch
    // network images — decode each into the row's `artwork` (#631). The
    // top-tracks model is built 1:1 from `data.top_tracks` (no blacklist
    // filter, unlike releases), so the enumerate index matches the row.
    for (i, track) in data.top_tracks.iter().enumerate() {
        if !track.artwork_url.is_empty() {
            jobs.push(ArtworkJob {
                target: ArtworkTarget::ArtistTopTrack { index: i },
                url: track.artwork_url.clone(),
            });
        }
    }
    // Curated playlist covers (single rectangle cover per card).
    for (i, playlist) in data.playlists.iter().enumerate() {
        if !playlist.image_url.is_empty() {
            jobs.push(ArtworkJob {
                target: ArtworkTarget::ArtistPlaylistCover { index: i },
                url: playlist.image_url.clone(),
            });
        }
    }
    jobs
}
