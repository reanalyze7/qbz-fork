use slint::ComponentHandle;

use crate::{AppWindow, MbOriginData, NetworkSidebarState, SettingsState, ShellState};

/// Location parameters for the "artists from the same place" scene
/// view, captured from the Origin metadata. None until an artist's
/// metadata with a location resolves.
#[derive(Clone, Default)]
pub struct LocationParams {
    pub mbid: String,
    pub area_id: String,
    pub area_name: String,
    pub country: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
}

static LOCATION_PARAMS: std::sync::Mutex<Option<LocationParams>> =
    std::sync::Mutex::new(None);

pub(crate) fn store_location_params(
    mbid: &str,
    meta: &qbz_integrations::musicbrainz::ArtistMetadata,
) {
    let params = meta.location.as_ref().map(|loc| LocationParams {
        mbid: mbid.to_string(),
        area_id: loc.area_id.clone().unwrap_or_default(),
        area_name: loc
            .city
            .clone()
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| loc.display_name.clone()),
        country: loc.country.clone().unwrap_or_default(),
        genres: meta.affinity_seeds.genres.clone(),
        tags: meta.affinity_seeds.tags.clone(),
    });
    if let Ok(mut guard) = LOCATION_PARAMS.lock() {
        *guard = params;
    }
}

/// The location params for the currently loaded artist, if it has a
/// resolved MB location. Read by the Origin location-click handler.
pub fn location_params() -> Option<LocationParams> {
    LOCATION_PARAMS.lock().ok().and_then(|g| g.clone())
}

/// Reset the network sidebar's MB-driven state and (re)apply the open
/// state on artist change so a stale Origin / Relationships / Discovery
/// never bleeds across artists. The sidebar opens fresh on every artist
/// visit (per user policy — close is per-session, never persisted)
/// EXCEPT when the content area is space-constrained (a Queue / Lyrics
/// right panel is open on a non-wide window). In that case it stays
/// collapsed so the Popular Tracks list keeps priority — mirroring the
/// `!net-cramped` rule the ArtistPageView Slint handlers use. Reading
/// the constraint here (instead of unconditionally force-opening) is
/// what fixes the artist->artist navigation case: when a panel was
/// already open, navigating to a new artist no longer re-opens the
/// sidebar over the Slint handler.
pub fn reset_network_sidebar(window: &AppWindow) {
    // Drop the previous artist's cached location params so a stale
    // scene-view click can't fire for the wrong artist.
    if let Ok(mut guard) = LOCATION_PARAMS.lock() {
        *guard = None;
    }
    // Open only when there's room — mirror ShellState.content-constrained
    // (the same signal AlbumView + the ArtistPageView `net-cramped`
    // handlers use). Constrained => keep collapsed.
    let constrained = window.global::<ShellState>().get_content_constrained();
    // MusicBrainz opt-out: when MB is off the Network tab (relationships /
    // discovery / origin) has nothing to show, so mark it unavailable and open
    // on the MB-independent Magazine/Stories tab instead of an empty Network
    // tab. The internal core guards (load_mb_metadata, musicbrainz_*) stay as
    // belt-and-suspenders.
    let mb_on = window.global::<SettingsState>().get_musicbrainz_enabled();
    let state = window.global::<NetworkSidebarState>();
    state.set_open(!constrained);
    // (Re)open a new artist on the Network tab when MB is on, else Magazine.
    let default_tab = if mb_on { "network" } else { "magazine" };
    state.set_active_tab(default_tab.into());
    state.set_mb_available(mb_on);
    state.set_mb_mbid("".into());
    state.set_origin_loading(false);
    state.set_origin(MbOriginData::default());
    state.set_relationships_loading(false);
    state.set_discovery_loading(false);
}
