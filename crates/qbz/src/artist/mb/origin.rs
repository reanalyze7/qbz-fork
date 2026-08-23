use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::ComponentHandle;

use super::location::store_location_params;
use super::map::map_origin;
use super::types::MbMetadata;
use crate::{AppWindow, MbOriginData, NetworkSidebarState};

/// Resolve the artist name to an MBID, then fetch artist metadata. Returns
/// `Ok(None)` when MB is disabled or no confident match is found — the
/// sidebar treats both the same (Origin section hides).
pub async fn load_mb_metadata<A>(
    runtime: &Arc<AppRuntime<A>>,
    artist_name: &str,
) -> Result<Option<MbMetadata>, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    if !runtime.core().musicbrainz_is_enabled().await {
        return Ok(None);
    }

    let resolved = runtime
        .core()
        .musicbrainz_resolve_artist(artist_name)
        .await
        .map_err(|e| e.to_string())?;
    let Some(resolved) = resolved else {
        return Ok(None);
    };
    // resolve_artist may return a low-confidence match; surface only the
    // mbid when the client gives us one (the qbz-integrations layer
    // already filters out the "no match at all" case before returning).
    let mbid = resolved.mbid;
    if mbid.is_empty() {
        return Ok(None);
    }

    let meta = runtime
        .core()
        .musicbrainz_get_artist_metadata(&mbid)
        .await
        .map_err(|e| e.to_string())?;

    // Cache the location params so the Origin location click can
    // open ArtistsByLocationView without re-resolving. Stored in a
    // cross-thread Mutex because the click handler runs on the UI
    // thread while this loads on a worker.
    store_location_params(&mbid, &meta);

    Ok(Some(MbMetadata {
        mbid: mbid.clone(),
        origin: map_origin(&meta),
    }))
}

/// Apply the MB metadata to NetworkSidebarState. Runs on the Slint
/// event loop.
pub fn apply_mb_metadata(window: &AppWindow, meta: MbMetadata) {
    let state = window.global::<NetworkSidebarState>();
    state.set_mb_mbid(meta.mbid.into());
    state.set_origin(MbOriginData {
        is_person: meta.origin.is_person,
        begin_date: meta.origin.begin_date.into(),
        end_date: meta.origin.end_date.into(),
        location_display: meta.origin.location_display.into(),
        location_clickable: meta.origin.location_clickable,
    });
    state.set_origin_loading(false);
}

/// Mark the sidebar as MB-unavailable (disabled in settings, or no
/// confident match for this artist). The MB-driven sections hide.
pub fn apply_mb_unavailable(window: &AppWindow) {
    let state = window.global::<NetworkSidebarState>();
    state.set_mb_available(false);
    state.set_origin_loading(false);
    state.set_relationships_loading(false);
    state.set_discovery_loading(false);
}
