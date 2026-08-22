//! Step 4c: if the pool is still small, fall back to Qobuz's own
//! "similar artists" API (guarantees the artists exist in Qobuz).
//!
//! Locking discipline: the `qobuz_client` read guard is a tokio lock and is
//! intentionally held for the whole loop (across the `get_similar_artists`,
//! `has_incompatible_genre`, and `search_artist_tracks_by_qobuz_id` awaits),
//! exactly matching the pre-split behavior — this is safe only because it is
//! a tokio (not std) lock.

use super::super::{SuggestedTrack, SuggestionsEngine};
use std::collections::HashSet;
use std::time::Instant;

const MIN_TRACKS_BEFORE_QOBUZ_SIMILAR: usize = 20;

impl SuggestionsEngine {
    pub(super) async fn qobuz_similar_fallback(
        &self,
        playlist_artists: &[(String, String)],
        exclude_track_ids: &HashSet<u64>,
        include_reasons: bool,
        source_artists: &mut Vec<String>,
        all_tracks: &mut Vec<SuggestedTrack>,
    ) -> Result<(), String> {
        if all_tracks.len() >= MIN_TRACKS_BEFORE_QOBUZ_SIMILAR {
            return Ok(());
        }

        log::info!(
            "[SuggestionsEngine] Step 4c: Pool too small ({}), fetching Qobuz similar artists",
            all_tracks.len()
        );
        let step4c_start = Instant::now();

        // Get client for Qobuz API calls
        let guard__ = self.qobuz_client.read().await;
        let client = guard__
            .as_ref()
            .ok_or_else(|| "No active session - please log in".to_string())?;

        // For each playlist artist, get their Qobuz similar artists
        let mut qobuz_similar_ids: HashSet<u64> = HashSet::new();

        for (_mbid, name) in playlist_artists {
            // First, find the Qobuz artist ID for this playlist artist
            if let Some((qobuz_id, _)) = self.validate_qobuz_artist(&client, name).await {
                // Get similar artists from Qobuz (up to 10 per playlist artist)
                match client.get_similar_artists(qobuz_id, 10, 0).await {
                    Ok(similar_page) => {
                        for similar_artist in similar_page.items {
                            // Skip if we've already processed this artist
                            if qobuz_similar_ids.contains(&similar_artist.id) {
                                continue;
                            }
                            qobuz_similar_ids.insert(similar_artist.id);

                            // Check genre compatibility
                            if self
                                .has_incompatible_genre(
                                    &client,
                                    similar_artist.id,
                                    &similar_artist.name,
                                )
                                .await
                            {
                                log::debug!(
                                    "[SuggestionsEngine] Skipping Qobuz similar '{}' - incompatible genre",
                                    similar_artist.name
                                );
                                continue;
                            }

                            if !source_artists.contains(&similar_artist.name) {
                                source_artists.push(similar_artist.name.clone());
                            }

                            // Search tracks for this similar artist (use empty MBID since we have Qobuz ID)
                            let tracks = self
                                .search_artist_tracks_by_qobuz_id(
                                    &client,
                                    similar_artist.id,
                                    &similar_artist.name,
                                    0.8, // High similarity since Qobuz says they're similar
                                )
                                .await;

                            for mut track in tracks {
                                if exclude_track_ids.contains(&track.track_id) {
                                    continue;
                                }

                                if include_reasons {
                                    track.reason = Some(format!("Similar to {} (Qobuz)", name));
                                }

                                all_tracks.push(track);
                            }

                            // Stop if we have enough
                            if all_tracks.len() >= self.config.max_pool_size {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "[SuggestionsEngine] Failed to get Qobuz similar artists for '{}': {}",
                            name, e
                        );
                    }
                }
            }

            if all_tracks.len() >= self.config.max_pool_size {
                break;
            }
        }

        log::debug!(
            "[SuggestionsEngine] Step 4c completed in {:?}, now have {} tracks from {} Qobuz similar artists",
            step4c_start.elapsed(),
            all_tracks.len(),
            qobuz_similar_ids.len()
        );

        Ok(())
    }
}
