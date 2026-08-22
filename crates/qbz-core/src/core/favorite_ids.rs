//! Paginated favorite/follow id-set fetchers. The two loops below are
//! near-identical (track vs. artist favorites) — per the refactor plan
//! this is a known opportunity for a shared `collect_favorite_ids`
//! helper, but factoring it is a behavior-preserving refactor beyond
//! this split's scope, so both loops are kept verbatim.

use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Fetch the set of the user's favorite track IDs. Pages through the
    /// favorites endpoint until exhausted. Used by clients that need to
    /// reflect favorite state on individual tracks (e.g. the Queue
    /// sidebar's now-playing heart).
    pub async fn favorite_track_ids(&self) -> Result<std::collections::HashSet<u64>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        let mut ids = std::collections::HashSet::new();
        let page_size: u32 = 500;
        let mut offset: u32 = 0;
        loop {
            let value = client
                .get_favorites("tracks", page_size, offset)
                .await
                .map_err(CoreError::Api)?;
            let items = value
                .get("tracks")
                .and_then(|t| t.get("items"))
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();
            let count = items.len() as u32;
            for item in &items {
                if let Some(id) = item.get("id").and_then(|v| v.as_u64()) {
                    ids.insert(id);
                }
            }
            if count < page_size {
                break;
            }
            offset += page_size;
        }
        Ok(ids)
    }

    /// Fetch the set of the user's favorite (followed) artist IDs. Pages
    /// through the favorites endpoint until exhausted. Used to reflect
    /// follow state on artist cards.
    pub async fn favorite_artist_ids(&self) -> Result<std::collections::HashSet<u64>, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        let mut ids = std::collections::HashSet::new();
        let page_size: u32 = 500;
        let mut offset: u32 = 0;
        loop {
            let value = client
                .get_favorites("artists", page_size, offset)
                .await
                .map_err(CoreError::Api)?;
            let items = value
                .get("artists")
                .and_then(|a| a.get("items"))
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();
            let count = items.len() as u32;
            for item in &items {
                if let Some(id) = item.get("id").and_then(|v| v.as_u64()) {
                    ids.insert(id);
                }
            }
            if count < page_size {
                break;
            }
            offset += page_size;
        }
        Ok(ids)
    }
}
