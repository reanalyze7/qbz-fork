//! Fan out to every source, normalize + merge into one date-ordered feed.
//! Qobuz-only for now (favorites + following); local items arrive
//! with the Phase 2 local-favorites layer behind the `show-local` switch.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;

use super::feed::Feed;

mod following;
mod local;
mod playlists;
mod tracks_albums;

type Runtime = Arc<AppRuntime<SlintAdapter>>;

pub async fn load_library_all(runtime: &Runtime) -> Result<Vec<Feed>, String> {
    let mut feed: Vec<Feed> = Vec::new();

    tracks_albums::load(runtime, &mut feed).await;
    following::load(runtime, &mut feed).await;
    playlists::load(runtime, &mut feed).await;
    local::load(&mut feed);

    // Merge by recency proxy (stable so equal ranks keep source order).
    feed.sort_by(|a, b| {
        a.added_rank
            .partial_cmp(&b.added_rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(feed)
}
