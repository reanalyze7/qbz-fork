//! Playlist "Suggested Songs" generation via the artist_vectors engine.

use std::collections::HashSet;
use std::sync::Arc;

use qbz_models::FrontendAdapter;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Generate playlist "Suggested Songs" via the artist_vectors engine.
    /// Resolves each playlist artist NAME to a confident MusicBrainz id, then
    /// runs the SuggestionsEngine over the core-owned clients + the per-user
    /// vector store. The names come from the playlist's own Qobuz tracks (so
    /// they are already canonical — the Tauri command's extra Qobuz-name
    /// re-fetch is unnecessary; `qobuz_id` is accepted for forward use).
    pub async fn generate_playlist_suggestions(
        &self,
        artists: Vec<(Option<u64>, String)>,
        exclude_track_ids: Vec<u64>,
        include_reasons: bool,
        config: Option<qbz_reco::SuggestionConfig>,
    ) -> Result<qbz_reco::SuggestionResult, String> {
        if artists.is_empty() {
            return Ok(qbz_reco::SuggestionResult {
                tracks: Vec::new(),
                source_artists: Vec::new(),
                playlist_artists_count: 0,
                similar_artists_count: 0,
            });
        }

        // Resolve each artist name to a confident MusicBrainz id, deduped.
        let mut resolved: Vec<(String, String)> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for (_qobuz_id, name) in &artists {
            if let Ok(Some(r)) = self.musicbrainz_resolve_artist(name).await {
                if seen.insert(r.mbid.clone()) {
                    resolved.push((r.mbid, name.clone()));
                }
            }
        }

        if resolved.is_empty() {
            log::warn!("[suggestions] no playlist artists resolved to MusicBrainz ids");
            return Ok(qbz_reco::SuggestionResult {
                tracks: Vec::new(),
                source_artists: Vec::new(),
                playlist_artists_count: artists.len(),
                similar_artists_count: 0,
            });
        }

        let config = config.unwrap_or_default();
        let builder = Arc::new(qbz_reco::ArtistVectorBuilder::new(
            self.artist_vectors.clone(),
            self.musicbrainz.clone(),
            self.musicbrainz_cache.clone(),
            self.client.clone(),
            qbz_reco::RelationshipWeights::default(),
        ));
        let engine = qbz_reco::SuggestionsEngine::new(
            self.artist_vectors.clone(),
            builder,
            self.client.clone(),
            config,
        );

        let exclude: HashSet<u64> = exclude_track_ids.into_iter().collect();
        engine
            .generate_suggestions(&resolved, &exclude, include_reasons)
            .await
    }
}
