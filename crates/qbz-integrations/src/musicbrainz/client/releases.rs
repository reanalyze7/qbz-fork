use super::MusicBrainzClient;
use crate::error::IntegrationResult;
use crate::musicbrainz::models::*;

impl MusicBrainzClient {
    /// Search releases by barcode (UPC/EAN)
    pub async fn search_release_by_barcode(
        &self,
        barcode: &str,
    ) -> IntegrationResult<ReleaseSearchResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let url = format!(
            "{}/release?query=barcode:{}&fmt=json&limit=5",
            base, barcode
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Search releases by title and artist
    pub async fn search_release(
        &self,
        title: &str,
        artist: &str,
    ) -> IntegrationResult<ReleaseSearchResponse> {
        self.search_releases_extended(title, artist, None, 5).await
    }

    /// Search releases with extended options
    pub async fn search_releases_extended(
        &self,
        title: &str,
        artist: &str,
        catalog_number: Option<&str>,
        limit: usize,
    ) -> IntegrationResult<ReleaseSearchResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let query = if let Some(catno) = catalog_number.filter(|s| !s.trim().is_empty()) {
            format!(
                "catno:\"{}\" AND artist:\"{}\"",
                Self::escape_query(catno),
                Self::escape_query(artist)
            )
        } else {
            format!(
                "release:\"{}\" AND artist:\"{}\"",
                Self::escape_query(title),
                Self::escape_query(artist)
            )
        };

        let limit = limit.min(25).max(1);
        let url = format!(
            "{}/release?query={}&fmt=json&limit={}",
            base,
            urlencoding::encode(&query),
            limit
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Get full release details including tracks
    pub async fn get_release_with_tracks(
        &self,
        release_id: &str,
    ) -> IntegrationResult<ReleaseFullResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let url = format!(
            "{}/release/{}?inc=recordings+artist-credits+labels+tags&fmt=json",
            base, release_id
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }
}
