//! Artist metadata (location/life-span/genre seeds) for the Origin
//! section of the artist network sidebar.

use qbz_integrations::musicbrainz::location;
use qbz_integrations::musicbrainz::ArtistMetadata;
use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Fetch the artist metadata (location, life_span, genre seeds) for
    /// the Origin section of the artist network sidebar. Resolves the
    /// real country from the begin_area hierarchy when a city-level
    /// location is found, because MB's `country` field is where the
    /// artist is active, not where they were born/formed.
    pub async fn musicbrainz_get_artist_metadata(
        &self,
        mbid: &str,
    ) -> Result<ArtistMetadata, CoreError> {
        // Cache lookup — same behavior as Tauri's v2 command.
        if let Ok(guard) = self.musicbrainz_cache.lock() {
            if let Some(cache) = guard.as_ref() {
                if let Ok(Some(cached)) = cache.get_artist_metadata(mbid) {
                    return Ok(cached);
                }
            }
        }

        let artist = self
            .musicbrainz
            .get_artist_with_relations(mbid)
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let mut metadata = location::extract_metadata(&artist);

        if let Some(ref mut loc) = metadata.location {
            if loc.city.is_some() {
                if let Some(ref area_id) = loc.area_id {
                    if let Ok(Some((country_name, country_code))) =
                        self.musicbrainz.resolve_area_country(area_id).await
                    {
                        loc.display_name = format!("{}, {}", loc.display_name, country_name);
                        loc.country = Some(country_name);
                        loc.country_code = country_code;
                    }
                }
            }
        }

        if let Ok(guard) = self.musicbrainz_cache.lock() {
            if let Some(cache) = guard.as_ref() {
                let _ = cache.set_artist_metadata(mbid, &metadata);
            }
        }

        Ok(metadata)
    }
}
