use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::favorites::{MAX_ITEMS, PAGE_SIZE};

/// Fetch the full set of favorite ALBUM ids from the server (paginated),
/// for seeding `fav_cache` at login so the album header heart is correct
/// without first visiting the Favorites view. Mirrors Tauri's
/// `albumFavoritesStore.syncFromApi`.
pub async fn favorite_album_ids<A>(
    runtime: &Arc<AppRuntime<A>>,
) -> std::collections::HashSet<String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut offset = 0u32;
    loop {
        let value = match runtime.core().get_favorites("albums", PAGE_SIZE, offset).await {
            Ok(v) => v,
            Err(e) => {
                log::warn!("[qbz-slint] favorite album ids fetch failed: {e}");
                break;
            }
        };
        let branch = value.get("albums");
        let total = branch
            .and_then(|b| b.get("total"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;
        let page: Vec<serde_json::Value> = branch
            .and_then(|b| b.get("items"))
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();
        let page_len = page.len();
        for item in &page {
            match item.get("id") {
                Some(v) if v.is_string() => {
                    if let Some(s) = v.as_str() {
                        ids.insert(s.to_string());
                    }
                }
                Some(v) if v.is_u64() => {
                    if let Some(n) = v.as_u64() {
                        ids.insert(n.to_string());
                    }
                }
                _ => {}
            }
        }
        offset += page_len as u32;
        let exhausted = page_len < PAGE_SIZE as usize
            || (total > 0 && offset as usize >= total)
            || ids.len() >= MAX_ITEMS;
        if exhausted {
            break;
        }
    }
    ids
}
