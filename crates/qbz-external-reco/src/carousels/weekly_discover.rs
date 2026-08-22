//! Weekly-playlist discovery + stale-fallback helper.

use qbz_integrations::listenbrainz::LbPlaylistMeta;
use qbz_integrations::ListenBrainzClient;

use crate::types::TrackReco;

use super::Cache;

/// Newest "created for you" playlist matching `source_patch` (created_at desc).
pub(super) async fn find_current_playlist(
    client: &ListenBrainzClient,
    username: &str,
    source_patch: &str,
) -> (usize, Option<LbPlaylistMeta>) {
    let playlists = client.get_created_for_playlists(username, 50).await.unwrap_or_default();
    let matching = playlists
        .iter()
        .filter(|p| p.source_patch.as_deref() == Some(source_patch))
        .count();
    let chosen = playlists
        .into_iter()
        .filter(|p| p.source_patch.as_deref() == Some(source_patch))
        .max_by(|a, b| a.created_at.cmp(&b.created_at));
    (matching, chosen)
}

/// Last resort when the current week can't be built (no playlist returned, or a
/// transient empty resolve): the most recent successfully-cached week, so the
/// row shows something instead of disappearing.
pub(super) fn cached_weekly_fallback(cache: Cache<'_>, source_patch: &str) -> Vec<TrackReco> {
    cache
        .and_then(|c| c.lock().ok().and_then(|g| g.get_latest_weekly_for_patch(source_patch)))
        .and_then(|json| serde_json::from_str::<Vec<TrackReco>>(&json).ok())
        .unwrap_or_default()
}
