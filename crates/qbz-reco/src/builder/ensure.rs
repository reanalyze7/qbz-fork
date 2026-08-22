//! Freshness check + build-if-needed entry point.

use super::ArtistVectorBuilder;

impl ArtistVectorBuilder {
    /// Ensure a vector exists and is fresh, building if necessary.
    ///
    /// Returns true if the vector was built/updated, false if an existing fresh
    /// vector was used.
    pub async fn ensure_vector(
        &self,
        artist_mbid: &str,
        artist_name: Option<&str>,
        qobuz_artist_id: Option<u64>,
        max_age_days: i64,
    ) -> Result<bool, String> {
        let max_age_secs = max_age_days * 24 * 60 * 60;

        // Check if we have a fresh vector
        let has_fresh = {
            let guard__ = self.store.lock().await;
            let store = guard__
                .as_ref()
                .ok_or("No active session - please log in")?;
            store.has_fresh_vector(artist_mbid, max_age_secs)
        };

        if has_fresh {
            return Ok(false);
        }

        // Build new vector
        match self
            .build_vector(artist_mbid, artist_name, qobuz_artist_id)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                log::error!(
                    "[VectorBuilder] Failed to build vector for {}: {}",
                    artist_mbid,
                    e
                );
                Err(e)
            }
        }
    }
}
