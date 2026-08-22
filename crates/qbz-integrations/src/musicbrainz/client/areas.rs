use super::MusicBrainzClient;
use crate::error::IntegrationResult;
use crate::musicbrainz::models::*;

impl MusicBrainzClient {
    /// Browse artists by area MBID
    pub async fn browse_artists_by_area(
        &self,
        area_id: &str,
        limit: usize,
        offset: usize,
    ) -> IntegrationResult<ArtistBrowseResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let limit = limit.min(100).max(1);
        let url = format!(
            "{}/artist?area={}&fmt=json&limit={}&offset={}&inc=tags",
            base, area_id, limit, offset
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Search for an area by name
    pub async fn search_area(
        &self,
        name: &str,
        area_type: Option<&str>,
    ) -> IntegrationResult<AreaSearchResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let query = if let Some(atype) = area_type {
            format!(
                "area:\"{}\" AND type:\"{}\"",
                Self::escape_query(name),
                Self::escape_query(atype)
            )
        } else {
            format!("area:\"{}\"", Self::escape_query(name))
        };

        let url = format!(
            "{}/area?query={}&fmt=json&limit=5",
            base,
            urlencoding::encode(&query)
        );

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }

    /// Look up an area and its parent relationships
    pub async fn get_area_with_relations(
        &self,
        area_id: &str,
    ) -> IntegrationResult<AreaDetailResponse> {
        self.check_enabled().await?;
        self.rate_limiter.wait().await;

        let base = self.base_url().await;
        let url = format!("{}/area/{}?inc=area-rels&fmt=json", base, area_id);

        let response = self.client.get(&url).send().await?;
        let response = self.handle_response_status(response).await?;
        response.json().await.map_err(Into::into)
    }
}
