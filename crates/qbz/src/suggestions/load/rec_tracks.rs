//! Recommended-track assembly: base (tracks-appears-on) + sparse fallback +
//! deterministic shuffle + radio-collage cover harvest.

use std::collections::HashSet;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::super::covers::track_album_cover;
use super::super::shuffle::shuffle_tracks;
use super::super::{FALLBACK_LIMIT, RADIO_COVERS, REC_LIMIT, SPARSE_THRESHOLD};

/// Result of the recommended-track pipeline: the shuffled+truncated track
/// list plus its derived radio diamond-collage cover urls.
pub(super) struct RecTracks {
    pub(super) tracks: Vec<qbz_models::Track>,
    pub(super) radio_cover_urls: Vec<String>,
}

/// Build the recommended-track list for `artist_id` / `current_track_id`:
/// base = `tracks_appears_on` (current track filtered, deduped by title),
/// with a sparse fallback merging `get_artist_tracks`, deterministically
/// shuffled and truncated to [`REC_LIMIT`].
pub(super) async fn build<A>(
    runtime: &AppRuntime<A>,
    artist_id: u64,
    current_track_id: u64,
    artist: &qbz_models::Artist,
) -> RecTracks
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let mut rec: Vec<qbz_models::Track> = Vec::new();
    let mut seen_titles: HashSet<String> = HashSet::new();
    if let Some(container) = artist.tracks_appears_on.as_ref() {
        for track in &container.items {
            if track.id == current_track_id {
                continue;
            }
            let key = track.title.to_lowercase().trim().to_string();
            if key.is_empty() || !seen_titles.insert(key) {
                continue;
            }
            rec.push(track.clone());
        }
    }

    // Sparse fallback: merge artist popular tracks (dedupe by title + id).
    if rec.len() < SPARSE_THRESHOLD {
        match runtime
            .core()
            .get_artist_tracks(artist_id, FALLBACK_LIMIT, 0)
            .await
        {
            Ok(container) => {
                let existing_ids: HashSet<u64> = rec.iter().map(|t| t.id).collect();
                for track in container.items {
                    if track.id == current_track_id || existing_ids.contains(&track.id) {
                        continue;
                    }
                    let key = track.title.to_lowercase().trim().to_string();
                    if key.is_empty() || !seen_titles.insert(key) {
                        continue;
                    }
                    rec.push(track);
                }
            }
            Err(e) => log::warn!("[qbz-slint] suggestions artist-tracks fallback failed: {e}"),
        }
    }

    // Shuffle (deterministic per artist+track), take REC_LIMIT.
    let seed = (artist_id << 1) ^ current_track_id.wrapping_add(1);
    shuffle_tracks(&mut rec, seed);
    rec.truncate(REC_LIMIT);

    // Radio diamond collage: up to RADIO_COVERS distinct rec-track album covers.
    let mut radio_cover_urls: Vec<String> = Vec::new();
    for track in &rec {
        if let Some(url) = track_album_cover(track) {
            if !radio_cover_urls.contains(&url) {
                radio_cover_urls.push(url);
                if radio_cover_urls.len() >= RADIO_COVERS {
                    break;
                }
            }
        }
    }

    RecTracks {
        tracks: rec,
        radio_cover_urls,
    }
}
