//! Weighted title/artist/album scoring with ISRC short-circuit and a
//! hi-res quality tiebreak.

use qbz_models::Track;

use crate::models::ImportTrack;

use super::normalize::similarity;

#[cfg(test)]
mod tests;

const TITLE_WEIGHT: f32 = 0.6;
const ARTIST_WEIGHT: f32 = 0.3;
const ALBUM_WEIGHT: f32 = 0.1;
pub(super) const MIN_SCORE: f32 = 0.65;

pub(super) fn select_best_match<'a>(
    track: &ImportTrack,
    candidates: &'a [Track],
) -> (Option<&'a Track>, f32) {
    let mut best: Option<&Track> = None;
    let mut best_score = 0.0f32;
    let mut best_quality = 0.0f32;

    for candidate in candidates {
        if !candidate.streamable {
            continue;
        }

        let score = score_candidate(track, candidate);
        let quality = quality_score(candidate);

        if score > best_score + 0.0001 {
            best = Some(candidate);
            best_score = score;
            best_quality = quality;
        } else if (score - best_score).abs() < 0.01 && quality > best_quality {
            best = Some(candidate);
            best_quality = quality;
        }
    }

    (best, best_score)
}

fn score_candidate(track: &ImportTrack, candidate: &Track) -> f32 {
    if let (Some(isrc), Some(candidate_isrc)) = (&track.isrc, &candidate.isrc) {
        if isrc.eq_ignore_ascii_case(candidate_isrc) {
            return 1.0;
        }
    }

    let title_score = similarity(&track.title, &candidate.title);
    let artist_score = similarity(
        &track.artist,
        candidate
            .performer
            .as_ref()
            .map(|a| a.name.as_str())
            .unwrap_or(""),
    );
    let album_score = track
        .album
        .as_ref()
        .map(|album| {
            candidate
                .album
                .as_ref()
                .map(|a| similarity(album, &a.title))
                .unwrap_or(0.0)
        })
        .unwrap_or(0.0);

    let mut score =
        title_score * TITLE_WEIGHT + artist_score * ARTIST_WEIGHT + album_score * ALBUM_WEIGHT;

    if let (Some(import_duration), Some(candidate_duration)) = (
        track.duration_ms,
        Some((candidate.duration as u64).saturating_mul(1000)),
    ) {
        let diff = if import_duration > candidate_duration {
            import_duration - candidate_duration
        } else {
            candidate_duration - import_duration
        };

        if diff <= 3000 {
            score += 0.05;
        } else if diff <= 5000 {
            score += 0.02;
        }
    }

    score
}

fn quality_score(track: &Track) -> f32 {
    let bit_depth = track.maximum_bit_depth.unwrap_or(0) as f32;
    let sample_rate = track.maximum_sampling_rate.unwrap_or(0.0) as f32;
    bit_depth * 100000.0 + sample_rate
}
