//! Concurrent search + match-entry assembly.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use futures_util::stream::{self, StreamExt};
use qbz_qobuz::QobuzClient;

use crate::errors::PlaylistImportError;
use crate::models::{ImportProgress, ImportTrack, TrackMatch};
use crate::sink::{ImportEvent, ImportProgressSink};

use super::scoring::{select_best_match, MIN_SCORE};

const SEARCH_LIMIT: u32 = 20;
const CONCURRENCY: usize = 8;

pub async fn match_tracks(
    client: &QobuzClient,
    tracks: &[ImportTrack],
    progress: Arc<dyn ImportProgressSink>,
) -> Result<Vec<TrackMatch>, PlaylistImportError> {
    let total = tracks.len() as u32;
    let matched_counter = Arc::new(AtomicU32::new(0));
    let completed_counter = Arc::new(AtomicU32::new(0));

    // Pre-allocate results vector with None slots
    let results: Arc<tokio::sync::Mutex<Vec<Option<TrackMatch>>>> =
        Arc::new(tokio::sync::Mutex::new(vec![None; tracks.len()]));

    let owned_tracks: Vec<(usize, ImportTrack)> = tracks
        .iter()
        .enumerate()
        .map(|(i, tr)| (i, tr.clone()))
        .collect();

    stream::iter(owned_tracks)
        .map(|(idx, track)| {
            let client = client.clone();
            let progress = Arc::clone(&progress);
            let matched_counter = Arc::clone(&matched_counter);
            let completed_counter = Arc::clone(&completed_counter);
            let results = Arc::clone(&results);

            async move {
                let query = format!("{} {}", track.artist, track.title);
                let search_result = client.search_tracks(&query, SEARCH_LIMIT, 0, None).await;

                let match_entry = match search_result {
                    Ok(search) => {
                        let (best, score) = select_best_match(&track, &search.items);
                        match best {
                            Some(candidate) if score >= MIN_SCORE => {
                                matched_counter.fetch_add(1, Ordering::Relaxed);
                                TrackMatch {
                                    source: track.clone(),
                                    qobuz_track_id: Some(candidate.id),
                                    qobuz_title: Some(candidate.title.clone()),
                                    qobuz_artist: candidate
                                        .performer
                                        .as_ref()
                                        .map(|a| a.name.clone()),
                                    score,
                                }
                            }
                            _ => TrackMatch {
                                source: track.clone(),
                                qobuz_track_id: None,
                                qobuz_title: None,
                                qobuz_artist: None,
                                score,
                            },
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Search failed for '{}' - '{}': {}",
                            track.artist,
                            track.title,
                            e
                        );
                        TrackMatch {
                            source: track.clone(),
                            qobuz_track_id: None,
                            qobuz_title: None,
                            qobuz_artist: None,
                            score: 0.0,
                        }
                    }
                };

                // Store result at correct index
                {
                    let mut res = results.lock().await;
                    res[idx] = Some(match_entry);
                }

                let current = completed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                let matched = matched_counter.load(Ordering::Relaxed);

                let current_track = Some(format!("{} - {}", track.artist, track.title));

                progress.emit(ImportEvent::Progress(ImportProgress {
                    phase: "matching".to_string(),
                    current,
                    total,
                    matched_so_far: matched,
                    current_track,
                }));
            }
        })
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<()>>()
        .await;

    // Extract results in order
    let locked = results.lock().await;
    let ordered: Vec<TrackMatch> = locked
        .iter()
        .map(|slot| slot.clone().expect("All slots should be filled"))
        .collect();

    Ok(ordered)
}
