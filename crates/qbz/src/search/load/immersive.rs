use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use crate::search::cortinilla::map_search_all_to_immersive;
use crate::search::local_rows::{append_immersive_local_albums, load_cortinilla_local, LocalCaps};
use crate::search::rows::CortinillaData;

/// Run a combined search for the in-immersive dropdown and map it to the
/// immersive payload (Albums/Artists/Playlists only — no local section, no
/// top-result hero). Reuses the same blacklist snapshot + `search_all` shape as
/// [`super::load_cortinilla`], but does NOT query the local library (immersive has no
/// on-device section) and does NOT persist to the search cache / learn a top
/// result (the immersive dropdown is playback-only, so the ranking-feedback
/// surface stays the main cortinilla's).
pub async fn load_immersive_search<A>(
    runtime: &Arc<AppRuntime<A>>,
    query: &str,
    expand_local: bool,
) -> Result<CortinillaData, String>
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
    let caps = LocalCaps::for_session(expand_local);
    let core = runtime.core();
    // Qobuz catalog + local albums CONCURRENTLY. Local is ungated (immersive
    // search has its own "search action" enable, independent of the main
    // cortinilla's intelligent-search toggle).
    let (results, local_rows) = tokio::join!(
        core.search_all(query, &blacklist, &album_blacklist),
        load_cortinilla_local(query, caps.fetch_limit(), false)
    );
    // A Qobuz error (offline / not signed in) still yields a local-only dropdown
    // rather than discarding everything — the local albums are already resolved.
    let mut data = match results {
        Ok(results) => map_search_all_to_immersive(query, &results),
        Err(e) => {
            log::debug!("[qbz-slint] immersive search: qobuz failed ({e}); local-only");
            CortinillaData {
                query: query.to_string(),
                top: None,
                sections: Vec::new(),
            }
        }
    };
    // Immersive shows albums ONLY; selecting one queues it per the action.
    append_immersive_local_albums(&mut data, &local_rows, caps.albums);
    Ok(data)
}
