//! Enable/disable toggle and name-to-id resolution.

use qbz_integrations::musicbrainz::ResolvedArtist;
use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Whether MusicBrainz integration is currently enabled.
    pub async fn musicbrainz_is_enabled(&self) -> bool {
        self.musicbrainz.is_enabled().await
    }

    /// Enable or disable MusicBrainz integration.
    pub async fn musicbrainz_set_enabled(&self, enabled: bool) {
        self.musicbrainz.set_enabled(enabled).await;
    }

    /// Resolve an artist name to a MusicBrainz id. Returns `None` if no
    /// confident match is found.
    pub async fn musicbrainz_resolve_artist(
        &self,
        name: &str,
    ) -> Result<Option<ResolvedArtist>, CoreError> {
        self.musicbrainz
            .resolve_artist(name)
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))
    }
}
