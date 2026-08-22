//! Genre-compatibility filtering for candidate Qobuz artists.

mod blocklist;

use super::SuggestionsEngine;
use blocklist::{INCOMPATIBLE_GENRES, INCOMPATIBLE_TITLE_KEYWORDS};
use qbz_qobuz::QobuzClient;

impl SuggestionsEngine {
    /// Check if an artist has incompatible genres (bachata, merengue, k-pop, etc.)
    ///
    /// Fetches a few albums and checks their genres against a blocklist.
    /// Returns true if incompatible, false if compatible or unknown.
    pub(super) async fn has_incompatible_genre(
        &self,
        client: &QobuzClient,
        artist_id: u64,
        artist_name: &str,
    ) -> bool {
        // Fetch artist with a few albums (use English locale for consistent genre names)
        match client
            .get_artist_with_pagination_and_locale(artist_id, true, Some(5), None, Some("en"))
            .await
        {
            Ok(artist) => {
                if let Some(albums) = &artist.albums {
                    for album in &albums.items {
                        if let Some(genre) = &album.genre {
                            let genre_lower = genre.name.to_lowercase();

                            // Check if genre matches any incompatible keyword
                            for incompatible in INCOMPATIBLE_GENRES {
                                if genre_lower.contains(incompatible) {
                                    log::debug!(
                                        "[SuggestionsEngine] Artist '{}' has incompatible genre: '{}' (album: {})",
                                        artist_name, genre.name, album.title
                                    );
                                    return true;
                                }
                            }
                        }

                        // Also check album title for genre hints (e.g., "Latino Bachata Amor")
                        let title_lower = album.title.to_lowercase();
                        for incompatible in INCOMPATIBLE_GENRES {
                            if title_lower.contains(incompatible) {
                                log::debug!(
                                    "[SuggestionsEngine] Artist '{}' has incompatible album title: '{}'",
                                    artist_name, album.title
                                );
                                return true;
                            }
                        }

                        for keyword in INCOMPATIBLE_TITLE_KEYWORDS {
                            if title_lower.contains(keyword) {
                                log::debug!(
                                    "[SuggestionsEngine] Artist '{}' has incompatible album title keyword '{}': '{}'",
                                    artist_name, keyword, album.title
                                );
                                return true;
                            }
                        }
                    }
                }
                false
            }
            Err(e) => {
                log::warn!(
                    "[SuggestionsEngine] Failed to fetch albums for genre check ({}): {}",
                    artist_name,
                    e
                );
                // On error, don't block - let it through
                false
            }
        }
    }
}
