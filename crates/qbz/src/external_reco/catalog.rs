//! `RecoCatalog` over `QbzCore` (errors -> empty).

use std::sync::Arc;

use async_trait::async_trait;
use qbz_app::shell::AppRuntime;
use qbz_external_reco::RecoCatalog;
use qbz_models::{Album, Artist, Track};

use crate::adapter::SlintAdapter;

pub(super) struct CoreRecoCatalog {
    pub(super) runtime: Arc<AppRuntime<SlintAdapter>>,
}

#[async_trait]
impl RecoCatalog for CoreRecoCatalog {
    async fn search_tracks(&self, query: &str, limit: usize) -> Vec<Track> {
        self.runtime
            .core()
            .search_tracks(query, limit as u32, 0, None)
            .await
            .map(|p| p.items)
            .unwrap_or_default()
    }
    async fn search_artists(&self, query: &str, limit: usize) -> Vec<Artist> {
        self.runtime
            .core()
            .search_artists(query, limit as u32, 0, None)
            .await
            .map(|p| p.items)
            .unwrap_or_default()
    }
    async fn search_albums(&self, query: &str, limit: usize) -> Vec<Album> {
        self.runtime
            .core()
            .search_albums(query, limit as u32, 0, None)
            .await
            .map(|p| p.items)
            .unwrap_or_default()
    }
    async fn artist_top_tracks(&self, artist_id: u64, limit: usize) -> Vec<Track> {
        self.runtime
            .core()
            .get_artist_tracks(artist_id, limit as u32, 0)
            .await
            .map(|c| c.items)
            .unwrap_or_default()
    }
    async fn artist_albums(&self, artist_id: u64, limit: usize) -> Vec<Album> {
        self.runtime
            .core()
            .get_artist_albums(artist_id, Some(limit as u32), Some(0))
            .await
            .map(|a| a.items)
            .unwrap_or_default()
    }
    async fn featured_albums(&self, kind: &str, limit: usize) -> Vec<Album> {
        self.runtime
            .core()
            .get_featured_albums(kind, limit as u32, 0, None)
            .await
            .map(|p| p.items)
            .unwrap_or_default()
    }
    async fn get_artist(&self, artist_id: u64) -> Option<Artist> {
        self.runtime.core().get_artist(artist_id).await.ok()
    }
}
