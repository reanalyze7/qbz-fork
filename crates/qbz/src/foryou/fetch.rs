//! Per-section fetch helpers (network). Each returns owned, mapped data so
//! the orchestrator can fire its apply the moment the call resolves.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::{Album, Artist};

use super::models::map_album;
use super::AlbumCard;

/// `pub(crate)` since #566: Home's Release Watch rail (`home::load_home`)
/// fetches through this SAME pipeline (blacklist filter included) so both
/// tabs render identical data.
pub(crate) async fn fetch_release_watch<A>(runtime: &Arc<AppRuntime<A>>) -> Vec<AlbumCard>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime.core().get_release_watch("artists", 18, 0).await {
        Ok(page) => {
            // T8: drop blacklisted flat Albums (primary OR any-artist,
            // featured-aware via album_blacklisted). Tauri release-watch
            // also runs the availability filter and bundles BOTH removals
            // into one `total` decrement — this For You carousel surfaces
            // no count (it's a fixed 18-item rail) and applies no separate
            // availability filter, so we just drop the blacklisted rows.
            let (bl, abl) = if crate::artist_blacklist::is_enabled() {
                (
                    crate::artist_blacklist::ids_snapshot(),
                    crate::artist_blacklist::album_ids_snapshot(),
                )
            } else {
                Default::default()
            };
            page.items
                .into_iter()
                .filter(|a| !qbz_core::core::album_blacklisted(a, &bl, &abl))
                .map(map_album)
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

pub(super) async fn fetch_fav_artists<A>(runtime: &Arc<AppRuntime<A>>) -> Vec<Artist>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime.core().get_favorites("artists", 50, 0).await {
        Ok(value) => qbz_models::lenient::parse_items_array(&value, "artists", "for-you artist"),
        Err(_) => Vec::new(),
    }
}

pub(super) async fn fetch_fav_albums<A>(runtime: &Arc<AppRuntime<A>>) -> Vec<Album>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime.core().get_favorites("albums", 100, 0).await {
        Ok(value) => qbz_models::lenient::parse_items_array(&value, "albums", "for-you album"),
        Err(_) => Vec::new(),
    }
}

pub(super) async fn fetch_suggest<A>(runtime: &Arc<AppRuntime<A>>, album_id: &str) -> Vec<AlbumCard>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    if album_id.is_empty() {
        return Vec::new();
    }
    match runtime.core().get_album_suggest(album_id).await {
        Ok(resp) => {
            // T8: similar-albums (flat Album). Filter-then-truncate: drop
            // blacklisted BEFORE take(18) (Tauri parity — may yield fewer
            // than the limit, no backfill).
            let (bl, abl) = if crate::artist_blacklist::is_enabled() {
                (
                    crate::artist_blacklist::ids_snapshot(),
                    crate::artist_blacklist::album_ids_snapshot(),
                )
            } else {
                Default::default()
            };
            resp.albums
                .map(|p| p.items)
                .unwrap_or_default()
                .into_iter()
                .filter(|a| !qbz_core::core::album_blacklisted(a, &bl, &abl))
                .take(18)
                .map(map_album)
                .collect()
        }
        Err(_) => Vec::new(),
    }
}
