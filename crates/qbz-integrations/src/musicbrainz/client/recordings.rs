use super::MusicBrainzClient;
use crate::error::{IntegrationError, IntegrationResult};
use crate::musicbrainz::models::*;

impl MusicBrainzClient {
    /// Search recordings by ISRC
    pub async fn search_recording_by_isrc(
        &self,
        isrc: &str,
    ) -> IntegrationResult<RecordingSearchResponse> {
        if !self.is_enabled().await {
            return Err(IntegrationError::ServiceUnavailable(
                "MusicBrainz integration is disabled".into(),
            ));
        }

        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let url = format!("{}/recording?query=isrc:{}&fmt=json", base, isrc);

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Search recordings by title and artist
    pub async fn search_recording(
        &self,
        title: &str,
        artist: &str,
    ) -> IntegrationResult<RecordingSearchResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let query = format!(
            "recording:\"{}\" AND artist:\"{}\"",
            Self::escape_query(title),
            Self::escape_query(artist)
        );
        let url = format!(
            "{}/recording?query={}&fmt=json&limit=5",
            base,
            urlencoding::encode(&query)
        );

        let response = self.client.get(&url).send().await?;
        self.check_response(&response).await;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// MBID -> ISRCs bridge. Looks up a recording's ISRCs (the strong key for Qobuz matching).
    /// GET {base}/recording/{recording_mbid}?inc=isrcs&fmt=json
    /// Returns the ISRC list, or an EMPTY vec on any non-success/parse failure (a missing ISRC is normal, not an error).
    pub async fn get_recording_isrcs(&self, recording_mbid: &str) -> IntegrationResult<Vec<String>> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;
        let base = self.base_url().await;
        let url = format!("{}/recording/{}?inc=isrcs&fmt=json", base, recording_mbid);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Ok(Vec::new());
        }
        let parsed: RecordingLookupResponse = match response.json().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };
        Ok(parsed.isrcs.unwrap_or_default())
    }

    /// Resolve a track to get MusicBrainz IDs
    ///
    /// Searches by ISRC if available, falling back to text search.
    pub async fn resolve_track(
        &self,
        artist: &str,
        title: &str,
        isrc: Option<&str>,
    ) -> IntegrationResult<Option<ResolvedTrack>> {
        // Try ISRC first (most accurate)
        if let Some(isrc) = isrc {
            let response = self.search_recording_by_isrc(isrc).await?;
            if let Some(recording) = response.recordings.first() {
                let confidence = if recording
                    .isrcs
                    .as_ref()
                    .map_or(false, |isrcs| isrcs.contains(&isrc.to_string()))
                {
                    MatchConfidence::Exact
                } else {
                    MatchConfidence::from_score(recording.score)
                };

                return Ok(Some(ResolvedTrack {
                    recording_mbid: recording.id.clone(),
                    title: recording.title.clone().unwrap_or_default(),
                    artist_mbids: recording
                        .artist_credit
                        .as_ref()
                        .map(|ac| ac.iter().map(|a| a.artist.id.clone()).collect())
                        .unwrap_or_default(),
                    release_mbid: recording
                        .releases
                        .as_ref()
                        .and_then(|r| r.first())
                        .map(|r| r.id.clone()),
                    isrcs: recording.isrcs.clone().unwrap_or_default(),
                    confidence,
                }));
            }
        }

        // TODO: Implement text-based search fallback
        // For now, return None if ISRC search fails
        let _ = (artist, title); // Silence unused warnings
        Ok(None)
    }
}
