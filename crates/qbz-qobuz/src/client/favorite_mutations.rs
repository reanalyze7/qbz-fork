use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};

impl QobuzClient {
    /// Add item to favorites
    pub async fn add_favorite(&self, fav_type: &str, item_id: &str) -> Result<()> {
        let url = endpoints::build_url(paths::FAVORITE_CREATE);
        let type_key = format!("{}_ids", fav_type); // album_ids, track_ids, artist_ids

        let response = self
            .signed_get_auth(&url, "favoritecreate", &[(type_key.as_str(), item_id.to_string())])
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApiError::ApiResponse(format!(
                "Failed to add favorite: {}",
                response.status()
            )))
        }
    }

    /// Remove item from favorites
    pub async fn remove_favorite(&self, fav_type: &str, item_id: &str) -> Result<()> {
        let url = endpoints::build_url(paths::FAVORITE_DELETE);
        let type_key = format!("{}_ids", fav_type);

        let response = self
            .signed_get_auth(&url, "favoritedelete", &[(type_key.as_str(), item_id.to_string())])
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApiError::ApiResponse(format!(
                "Failed to remove favorite: {}",
                response.status()
            )))
        }
    }
}
