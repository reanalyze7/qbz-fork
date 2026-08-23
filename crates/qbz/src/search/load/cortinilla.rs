use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::search::cortinilla::map_search_all_to_cortinilla;
use crate::search::local_rows::{append_local_sections, load_cortinilla_local, LocalCaps};
use crate::search::rows::CortinillaData;

/// Run a combined search for the live cortinilla, store it in the per-user
/// cache, and map it to the dropdown payload. Reuses the same blacklist +
/// `search_all` shape as [`super::load_search`]. The learned top-result for the query
/// is folded in by `map_search_all_to_cortinilla`.
///
/// Returns the mapped dropdown payload AND the raw `LocalTrack` rows that backed
/// the on-device section, so the caller can snapshot them for click routing
/// (the click router plays a local row through `playback::play_local_tracks`,
/// which needs the concrete `LocalTrack`, not just its id).
pub async fn load_cortinilla<A>(
    runtime: &Arc<AppRuntime<A>>,
    query: &str,
    expand_local: bool,
) -> Result<(CortinillaData, Vec<qbz_library::LocalTrack>), String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let blacklist = if crate::artist_blacklist::is_enabled() {
        crate::artist_blacklist::ids_snapshot()
    } else {
        std::collections::HashSet::new()
    };
    // Album axis shares the same enabled gate.
    let album_blacklist = if crate::artist_blacklist::is_enabled() {
        crate::artist_blacklist::album_ids_snapshot()
    } else {
        std::collections::HashSet::new()
    };
    // Offline / not-signed-in → the dropdown is local-only, so widen the local
    // section caps (and the raw fetch that feeds the derived album/artist groups).
    let caps = LocalCaps::for_session(expand_local);
    let core = runtime.core();
    // Fire the Qobuz search and the local-library search CONCURRENTLY. The local
    // query is independent — if Qobuz is slow/offline the on-device rows still
    // fill in (a Qobuz error falls into the local-only branch below instead of
    // discarding everything; the local rows are already resolved by then).
    let (results, local_rows) = tokio::join!(
        core.search_all(query, &blacklist, &album_blacklist),
        load_cortinilla_local(query, caps.fetch_limit(), true)
    );
    let mut data = match results {
        Ok(results) => {
            // Persist the live page so a later keystroke (or restart) can paint
            // instantly from cache (SWR). No-op when the module is disabled.
            crate::search_service::store(query, &results);
            let top = crate::search_service::top_for_query(query);
            map_search_all_to_cortinilla(query, &results, top)
        }
        Err(e) => {
            // Qobuz failed (offline / API error). The on-device rows resolved
            // independently, so still build a dropdown from JUST the local
            // section rather than dropping everything. An empty local set then
            // yields an empty payload (the overlay shows only "Search for …").
            log::debug!("[qbz-slint] cortinilla: qobuz search failed ({e}); local-only");
            CortinillaData {
                query: query.to_string(),
                top: None,
                sections: Vec::new(),
            }
        }
    };
    // Append the local "on this device" sections LAST (after every Qobuz
    // category) and re-run flat-index assignment so the local rows get
    // contiguous indices.
    append_local_sections(&mut data, &local_rows, caps);
    Ok((data, local_rows))
}
