//! `ArtistVectorBuilder` struct definition, `BuildResult`, and constructor.

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use qbz_integrations::musicbrainz::cache::MusicBrainzCache;
use qbz_integrations::MusicBrainzClient;
use qbz_qobuz::QobuzClient;

use crate::sparse_vector::SparseVector;
use crate::store::ArtistVectorStore;
use crate::weights::RelationshipWeights;

/// Builder for constructing artist vectors from multiple data sources
pub struct ArtistVectorBuilder {
    /// Vector store for persistence
    pub(super) store: Arc<Mutex<Option<ArtistVectorStore>>>,
    /// MusicBrainz client (Send+Sync, no outer lock — matches qbz-core's
    /// `Arc<MusicBrainzClient>`).
    pub(super) mb_client: Arc<MusicBrainzClient>,
    /// MusicBrainz cache (std::sync::Mutex — matches qbz-core; locked only for
    /// short, await-free windows so no guard crosses an `.await`).
    pub(super) mb_cache: Arc<std::sync::Mutex<Option<MusicBrainzCache>>>,
    /// Qobuz client for similar artists (`Option` = no active session).
    pub(super) qobuz_client: Arc<RwLock<Option<QobuzClient>>>,
    /// Configurable weights
    pub(super) weights: RelationshipWeights,
}

/// Result of building a vector
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// The built vector
    pub vector: SparseVector,
    /// Number of MusicBrainz relationships found
    pub mb_relations_count: usize,
    /// Number of Qobuz similar artists found
    pub qobuz_similar_count: usize,
    /// Sources that contributed to the vector
    pub sources: Vec<String>,
}

impl ArtistVectorBuilder {
    /// Create a new builder with the given dependencies
    pub fn new(
        store: Arc<Mutex<Option<ArtistVectorStore>>>,
        mb_client: Arc<MusicBrainzClient>,
        mb_cache: Arc<std::sync::Mutex<Option<MusicBrainzCache>>>,
        qobuz_client: Arc<RwLock<Option<QobuzClient>>>,
        weights: RelationshipWeights,
    ) -> Self {
        Self {
            store,
            mb_client,
            mb_cache,
            qobuz_client,
            weights,
        }
    }
}
