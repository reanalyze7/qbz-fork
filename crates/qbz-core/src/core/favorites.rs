//! Favorites: get/add/remove. See `favorite_ids.rs` for the paginated
//! id-set fetchers used to reflect favorite/follow state on cards.

use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Get favorites (albums, tracks, or artists)
    pub async fn get_favorites(
        &self,
        fav_type: &str,
        limit: u32,
        offset: u32,
    ) -> Result<serde_json::Value, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .get_favorites(fav_type, limit, offset)
            .await
            .map_err(CoreError::Api)
    }

    /// Add item to favorites
    pub async fn add_favorite(&self, fav_type: &str, item_id: &str) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .add_favorite(fav_type, item_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Remove item from favorites
    pub async fn remove_favorite(&self, fav_type: &str, item_id: &str) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        client
            .remove_favorite(fav_type, item_id)
            .await
            .map_err(CoreError::Api)
    }

    /// Toggle the favorite state of a track. `make_favorite = true` adds it,
    /// `false` removes it. Thin convenience over `add_favorite` /
    /// `remove_favorite` so callers do not duplicate the type string.
    pub async fn set_track_favorite(
        &self,
        track_id: u64,
        make_favorite: bool,
    ) -> Result<(), CoreError> {
        let id = track_id.to_string();
        if make_favorite {
            self.add_favorite("track", &id).await
        } else {
            self.remove_favorite("track", &id).await
        }
    }
}
