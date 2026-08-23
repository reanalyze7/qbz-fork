use std::collections::HashMap;

use super::scores::RecoScoreEntry;

/// Sort a `{id: score}` map descending and cap it at `max_per_type`, mapping
/// each surviving pair into a track-shaped [`RecoScoreEntry`].
pub(super) fn build_track_entries(
    scores: HashMap<u64, f64>,
    max_per_type: usize,
) -> Vec<RecoScoreEntry> {
    let mut entries: Vec<(u64, f64)> = scores.into_iter().collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries
        .into_iter()
        .take(max_per_type)
        .map(|(track_id, score)| RecoScoreEntry {
            track_id: Some(track_id),
            album_id: None,
            artist_id: None,
            score,
        })
        .collect()
}

pub(super) fn build_album_entries(
    scores: HashMap<String, f64>,
    max_per_type: usize,
) -> Vec<RecoScoreEntry> {
    let mut entries: Vec<(String, f64)> = scores.into_iter().collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries
        .into_iter()
        .take(max_per_type)
        .map(|(album_id, score)| RecoScoreEntry {
            track_id: None,
            album_id: Some(album_id),
            artist_id: None,
            score,
        })
        .collect()
}

pub(super) fn build_artist_entries(
    scores: HashMap<u64, f64>,
    max_per_type: usize,
) -> Vec<RecoScoreEntry> {
    let mut entries: Vec<(u64, f64)> = scores.into_iter().collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries
        .into_iter()
        .take(max_per_type)
        .map(|(artist_id, score)| RecoScoreEntry {
            track_id: None,
            album_id: None,
            artist_id: Some(artist_id),
            score,
        })
        .collect()
}
