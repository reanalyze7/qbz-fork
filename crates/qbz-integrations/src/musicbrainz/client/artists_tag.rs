use super::MusicBrainzClient;
use crate::error::IntegrationResult;
use crate::musicbrainz::models::*;

impl MusicBrainzClient {
    /// Search artists by tag (genre)
    pub async fn search_artists_by_tag(
        &self,
        tag: &str,
        limit: usize,
    ) -> IntegrationResult<ArtistSearchResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let limit = limit.min(100).max(1);
        let query = format!("tag:\"{}\"", Self::escape_query(tag));
        let url = format!(
            "{}/artist?query={}&fmt=json&limit={}",
            base,
            urlencoding::encode(&query),
            limit
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Search artists by tag AND area
    pub async fn search_artists_by_tag_and_area(
        &self,
        tag: &str,
        area_name: &str,
        country: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> IntegrationResult<ArtistSearchResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let limit = limit.min(100).max(1);
        let search_area = country.unwrap_or(area_name);
        let query = format!(
            "tag:\"{}\" AND area:\"{}\"",
            Self::escape_query(tag),
            Self::escape_query(search_area)
        );
        let url = format!(
            "{}/artist?query={}&fmt=json&limit={}&offset={}",
            base,
            urlencoding::encode(&query),
            limit,
            offset
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }
}
