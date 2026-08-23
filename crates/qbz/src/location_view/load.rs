//! Async scene-discovery data loading (MusicBrainz + Qobuz).

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::{map_candidate, LocationData, PAGE_SIZE};
use crate::artist::LocationParams;

/// Run the first page of scene discovery for `params`.
pub async fn load_scene<A>(
    runtime: &Arc<AppRuntime<A>>,
    params: &LocationParams,
    offset: usize,
) -> Result<LocationData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let area_id = (!params.area_id.is_empty()).then_some(params.area_id.as_str());
    let country = (!params.country.is_empty()).then_some(params.country.as_str());
    let response = runtime
        .core()
        .discover_artists_by_location(
            &params.mbid,
            area_id,
            &params.area_name,
            country,
            params.genres.clone(),
            params.tags.clone(),
            PAGE_SIZE,
            offset,
        )
        .await
        .map_err(|e| e.to_string())?;

    let _ = response.next_offset; // offset is recomputed from row_count

    // T8 + D-FIX-c: drop blacklisted scene candidates by resolved Qobuz id.
    //
    // D-FIX-c note: the Slint scene controller does NOT cache results (no
    // 30-day TTL cache exists in `qbz-core::discover_artists_by_location` nor
    // here — the Tauri bug was in the Tauri-only `integrations.rs` command
    // cache, which has no analogue in this path). We still re-apply the
    // blacklist HERE, on every result on the way out, so that even if any
    // upstream cache were ever introduced, a newly-blocked artist disappears
    // immediately. `is_blacklisted` auto-gates on the enabled flag; a
    // missing/None Qobuz id is kept (fail-open). `total` is decremented by
    // the number removed so the count stays honest.
    let mut artists = response.artists;
    let before = artists.len();
    artists.retain(|c| match c.qobuz_id {
        Some(id) if id >= 0 => !crate::artist_blacklist::is_blacklisted(id as u64),
        _ => true,
    });
    let removed = before - artists.len();
    let total = response.total_candidates.saturating_sub(removed);

    Ok(LocationData {
        scene_label: response.scene_label,
        genre_summary: response.genre_summary,
        artists: artists.into_iter().map(map_candidate).collect(),
        total,
    })
}
