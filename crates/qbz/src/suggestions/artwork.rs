//! Artwork jobs for the assembled suggestions: per-card collage slots +
//! rec-track row thumbnails.

use crate::artwork::{ArtworkJob, ArtworkTarget};

use super::types::SuggestionsPayload;

/// Artwork jobs for the assembled suggestions: per-card collage slots +
/// rec-track row thumbnails.
///
/// The radio card's index is recomputed as `payload.playlist_cards.len()` —
/// this must stay in sync with `apply::apply_suggestions`'s card-ordering
/// (playlist cards first, radio card appended last).
pub fn suggestions_artwork_jobs(payload: &SuggestionsPayload) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    // Card collage slots: playlist cards first (their order matches the model),
    // then the radio card (appended last in apply_suggestions).
    for (card_idx, card) in payload.playlist_cards.iter().enumerate() {
        for (slot, url) in card.cover_urls.iter().enumerate() {
            if !url.is_empty() {
                jobs.push(ArtworkJob {
                    url: url.clone(),
                    target: ArtworkTarget::SuggestionCardCover { card_idx, slot },
                });
            }
        }
    }
    if !payload.seed_track_id.is_empty() {
        let radio_idx = payload.playlist_cards.len();
        for (slot, url) in payload.radio_cover_urls.iter().enumerate() {
            if !url.is_empty() {
                jobs.push(ArtworkJob {
                    url: url.clone(),
                    target: ArtworkTarget::SuggestionCardCover {
                        card_idx: radio_idx,
                        slot,
                    },
                });
            }
        }
    }
    // Rec-track row thumbnails.
    for (idx, track) in payload.rec_tracks.iter().enumerate() {
        if let Some(url) = track
            .album
            .as_ref()
            .and_then(|a| a.image.smallest().cloned())
            .filter(|s| !s.is_empty())
        {
            jobs.push(ArtworkJob {
                url,
                target: ArtworkTarget::SuggestionTrackCover { idx },
            });
        }
    }
    jobs
}
