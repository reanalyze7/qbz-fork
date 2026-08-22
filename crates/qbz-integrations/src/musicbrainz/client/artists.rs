use super::MusicBrainzClient;
use crate::error::{IntegrationError, IntegrationResult};
use crate::musicbrainz::models::*;

impl MusicBrainzClient {
    /// Search artists by name
    pub async fn search_artist(
        &self,
        name: &str,
        limit: u32,
    ) -> IntegrationResult<ArtistSearchResponse> {
        if !self.is_enabled().await {
            return Err(IntegrationError::ServiceUnavailable(
                "MusicBrainz integration is disabled".into(),
            ));
        }

        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}/artist?query=artist:{}&limit={}&fmt=json",
            base, encoded_name, limit
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Resolve an artist to get MusicBrainz ID
    ///
    /// Prefers exact name matches over highest score to avoid disambiguation
    /// issues (e.g., multiple artists named "The Warning").
    pub async fn resolve_artist(&self, name: &str) -> IntegrationResult<Option<ResolvedArtist>> {
        let response = self.search_artist(name, 10).await?;

        if response.artists.is_empty() {
            return Ok(None);
        }

        // Prefer exact name match (case-insensitive)
        let target = name.trim().to_lowercase();
        let best = response
            .artists
            .iter()
            .find(|a| a.name.trim().to_lowercase() == target && a.score.unwrap_or(0) >= 90)
            .or_else(|| response.artists.first());

        if let Some(artist) = best {
            let confidence = MatchConfidence::from_score(artist.score);

            return Ok(Some(ResolvedArtist {
                mbid: artist.id.clone(),
                name: artist.name.clone(),
                sort_name: artist.sort_name.clone(),
                artist_type: ArtistType::from(artist.artist_type.as_deref()),
                country: artist.country.clone(),
                disambiguation: artist.disambiguation.clone(),
                confidence,
            }));
        }

        Ok(None)
    }

    /// Get artist details with relationships and tags
    pub async fn get_artist_with_relations(
        &self,
        mbid: &str,
    ) -> IntegrationResult<ArtistFullResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let url = format!("{}/artist/{}?inc=artist-rels+tags&fmt=json", base, mbid);

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Fetch artist tags only (lightweight, no relations)
    pub async fn get_artist_tags(&self, mbid: &str) -> IntegrationResult<Vec<String>> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let url = format!("{}/artist/{}?inc=tags&fmt=json", base, mbid);

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let artist: ArtistFullResponse = response.json().await.map_err(|e| {
            IntegrationError::internal(format!("Failed to parse MusicBrainz response: {}", e))
        })?;

        let mut tags: Vec<_> = artist
            .tags
            .unwrap_or_default()
            .into_iter()
            .filter(|tag| tag.count.unwrap_or(0) > 0)
            .collect();
        tags.sort_by(|a, b| b.count.unwrap_or(0).cmp(&a.count.unwrap_or(0)));
        Ok(tags
            .into_iter()
            .map(|tag| tag.name.to_lowercase())
            .collect())
    }
}
