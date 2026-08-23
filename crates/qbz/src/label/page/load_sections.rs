//! The three "extra fetch" pieces of the landing page: Critics' Picks
//! (parsed out of the /label/page payload), the Releases carousel, and
//! the More Labels carousel (both separate API calls).

use std::collections::HashSet;
use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::{Album, LabelPageData};

use super::parse::parse_more_labels;
use super::LabelSlim;
use crate::album_map::{map_album, AlbumCard};

/// Critics' Picks — the releases container whose id mentions
/// award/critic/press (mirrors LabelView.svelte:402-413).
pub(super) fn critics_from_page(
    page: &LabelPageData,
    bl: &HashSet<u64>,
    abl: &HashSet<String>,
) -> Vec<AlbumCard> {
    page.releases
        .as_ref()
        .and_then(|containers| {
            containers
                .iter()
                .find(|c| {
                    c.id
                        .as_deref()
                        .map(|id| {
                            let id = id.to_lowercase();
                            id.contains("award") || id.contains("critic") || id.contains("press")
                        })
                        .unwrap_or(false)
                })
                .and_then(|c| c.data.as_ref())
                .and_then(|d| d.items.as_ref())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|v| serde_json::from_value::<Album>(v.clone()).ok())
                        .filter(|a| !qbz_core::core::album_blacklisted(a, bl, abl))
                        .map(map_album)
                        .collect()
                })
        })
        .unwrap_or_default()
}

/// Releases carousel — first 20 from /label/getAlbums.
pub(super) async fn releases_carousel<A>(
    runtime: &Arc<AppRuntime<A>>,
    label_id: u64,
    bl: &HashSet<u64>,
    abl: &HashSet<String>,
) -> Vec<AlbumCard>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime
        .core()
        .get_label_albums(label_id, 20, 0, None, None, None, None, None)
        .await
    {
        Ok(p) => p
            .items
            .into_iter()
            .filter(|a| !qbz_core::core::album_blacklisted(a, bl, abl))
            .map(map_album)
            .collect(),
        Err(e) => {
            log::warn!("[qbz-slint] label releases carousel failed: {e}");
            Vec::new()
        }
    }
}

/// More labels — /label/explore minus the current label; seed follow.
pub(super) async fn more_labels_carousel<A>(
    runtime: &Arc<AppRuntime<A>>,
    label_id: u64,
    follow_ids: &HashSet<u64>,
) -> Vec<LabelSlim>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime.core().get_label_explore(20, 0).await {
        Ok(resp) => parse_more_labels(&resp, label_id, follow_ids),
        Err(e) => {
            log::warn!("[qbz-slint] label explore failed: {e}");
            Vec::new()
        }
    }
}
