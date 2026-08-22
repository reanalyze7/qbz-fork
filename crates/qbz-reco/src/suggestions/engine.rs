//! `SuggestionsEngine` struct definition + constructor.

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use qbz_qobuz::QobuzClient;

use super::SuggestionConfig;
use crate::builder::ArtistVectorBuilder;
use crate::store::ArtistVectorStore;

/// Playlist suggestions engine
pub struct SuggestionsEngine {
    /// Vector store for similarity lookups
    pub(super) store: Arc<Mutex<Option<ArtistVectorStore>>>,
    /// Vector builder for lazy construction
    pub(super) builder: Arc<ArtistVectorBuilder>,
    /// Qobuz client for track search
    pub(super) qobuz_client: Arc<RwLock<Option<QobuzClient>>>,
    /// Configuration
    pub(super) config: SuggestionConfig,
}

impl SuggestionsEngine {
    /// Create a new suggestions engine
    pub fn new(
        store: Arc<Mutex<Option<ArtistVectorStore>>>,
        builder: Arc<ArtistVectorBuilder>,
        qobuz_client: Arc<RwLock<Option<QobuzClient>>>,
        config: SuggestionConfig,
    ) -> Self {
        Self {
            store,
            builder,
            qobuz_client,
            config,
        }
    }
}
