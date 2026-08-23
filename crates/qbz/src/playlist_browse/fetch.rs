//! The one async network call, isolated so it's easy to see/reason about
//! the API contract independent of UI-thread plumbing.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use super::model::{map_browse, BrowseCard};
use super::PAGE_SIZE;
use crate::adapter::SlintAdapter;

/// Fetch one page of `/discover/playlists` faceted by the tag slug
/// ("" = All) + the shared genre selection. Returns the cards and the
/// backend `has_more` flag (the endpoint carries no `total`).
pub(super) async fn fetch_page(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    tag: &str,
    genre_ids: Option<Vec<u64>>,
    offset: u32,
) -> Result<(Vec<BrowseCard>, bool), String> {
    let tag_opt = (!tag.is_empty()).then(|| tag.to_string());
    let data = runtime
        .core()
        .get_discover_playlists(tag_opt, genre_ids, Some(PAGE_SIZE), Some(offset))
        .await
        .map_err(|e| e.to_string())?;
    let has_more = data.has_more;
    Ok((data.items.into_iter().map(map_browse).collect(), has_more))
}
