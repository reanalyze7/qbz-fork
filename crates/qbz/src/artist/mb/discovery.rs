use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, DiscoveryArtist, NetworkSidebarState};

/// Plain, `Send` payload for the Discovery section. `primary_tag` is
/// kept alongside so the dismiss callback can look up the right key in
/// the dismiss store.
pub struct MbDiscoveryData {
    pub primary_tag: String,
    pub artists: Vec<MbDiscoveryRow>,
}

#[derive(Clone)]
pub struct MbDiscoveryRow {
    pub mbid: String,
    pub name: String,
    pub qobuz_id: String,
}

/// Load discovery candidates for `seed_mbid` (the artist's MB id) using
/// `similar_names` to suppress already-shown rows and the local
/// discovery_dismiss store to suppress thumbs-downed rows.
pub async fn load_mb_discovery<A>(
    runtime: &Arc<AppRuntime<A>>,
    seed_mbid: &str,
    seed_name: &str,
    similar_names: Vec<String>,
) -> Result<MbDiscoveryData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    // Tauri's listen threshold: artists with strictly more than 2
    // plays count as "already known" and are excluded from
    // suggestions.
    let known_threshold: u32 = 2;
    let response = runtime
        .core()
        .musicbrainz_discover_artists(
            seed_mbid,
            seed_name,
            &similar_names,
            &|tag| crate::discovery_dismiss::dismissed_for_tag(tag),
            &|| {
                // play_history supplies the (id, name) known set; reco augments
                // the id set with its richer signal (artists played >threshold
                // OR favorited). Names stay from play_history -- reco_events has
                // no artist names (schema frozen to match Tauri).
                let (mut ids, names) = crate::play_history::known_artists(known_threshold);
                if let Some(reco_ids) = crate::reco::known_artist_ids(known_threshold) {
                    ids.extend(reco_ids);
                }
                (ids, names)
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    let artists = response
        .artists
        .into_iter()
        .map(|a| MbDiscoveryRow {
            mbid: a.mbid,
            name: a.name,
            qobuz_id: a.qobuz_id.map(|id| id.to_string()).unwrap_or_default(),
        })
        .collect();

    Ok(MbDiscoveryData {
        primary_tag: response.primary_tag,
        artists,
    })
}

/// Apply discovery candidates to NetworkSidebarState. Runs on the
/// Slint event loop. `primary_tag` is stored on the sidebar state for
/// the dismiss callback to read.
pub fn apply_mb_discovery(window: &AppWindow, data: MbDiscoveryData) {
    let state = window.global::<NetworkSidebarState>();
    state.set_discovery_tag(data.primary_tag.into());
    let rows: Vec<DiscoveryArtist> = data
        .artists
        .into_iter()
        // T8: smart-discovery rail (v2_get_discovery_artists equivalent) —
        // skip candidates whose resolved Qobuz id is blacklisted.
        // is_blacklisted_id_str auto-gates on the enabled flag and treats a
        // missing/non-numeric id as not-blacklisted (kept).
        .filter(|r| !crate::artist_blacklist::is_blacklisted_id_str(&r.qobuz_id))
        .map(|r| DiscoveryArtist {
            mbid: r.mbid.into(),
            name: r.name.into(),
            qobuz_id: r.qobuz_id.into(),
        })
        .collect();
    state.set_discovery_artists(ModelRc::new(VecModel::from(rows)));
    state.set_discovery_loading(false);
}

/// Remove a dismissed row from the visible Discovery list. The dismiss
/// store persistence is handled by the caller before this is invoked.
pub fn remove_discovery_artist(window: &AppWindow, mbid: &str) {
    let state = window.global::<NetworkSidebarState>();
    let model = state.get_discovery_artists();
    let kept: Vec<DiscoveryArtist> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|row| row.mbid.as_str() != mbid)
        .collect();
    state.set_discovery_artists(ModelRc::new(VecModel::from(kept)));
}
