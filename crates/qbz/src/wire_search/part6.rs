use crate::*;

pub(crate) fn wire_search_part6(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Cortinilla: "View more" on a section. Qobuz categories open the full
    // results page on the matching tab (albums=1, tracks=2, artists=3,
    // playlists=4); the "local" section opens the LocalLibrary Tracks tab
    // pre-filtered to the live query (local results never enter the Qobuz
    // results page — D1/D2).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SearchActions>()
            .on_cortinilla_view_more(move |kind| {
                let Some(w) = weak.upgrade() else { return };
                let kind = kind.to_string();
                let q = w
                    .global::<SearchState>()
                    .get_cortinilla_query()
                    .trim()
                    .to_string();
                if q.chars().count() < 2 {
                    return;
                }
                {
                    let st = w.global::<SearchState>();
                    st.set_cortinilla_open(false);
                    // Clear the input so it can't re-invoke the dropdown later.
                    st.set_header_search_text("".into());
                }
                SEARCH_DEBOUNCE.with(|t| t.stop());

                // On-device "View more": leave the Qobuz results page entirely
                // and open the matching LocalLibrary tab pre-filtered to the live
                // query (D1/D2: local results never live in the Qobuz results
                // page). Albums / Artists / Tracks each route to their own tab,
                // setting that tab's search filter then forcing a re-derive so the
                // filtered set renders on both first-visit and re-entry.
                if kind == "local-album" {
                    w.global::<LocalLibraryState>().set_albums_search(q.clone().into());
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: local_library::LibTab::Albums.tab_id().to_string(),
                    });
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        local_library::LibTab::Albums,
                    );
                    // Force a reload so the freshly-set search filter applies even
                    // when the Albums tab was already loaded (re-entry).
                    local_library::reload_albums(weak.clone(), handle.clone(), image_cache.clone());
                    update_nav_flags(&w);
                    return;
                }
                if kind == "local-artist" {
                    w.global::<LocalLibraryState>().set_artists_search(q.clone().into());
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: local_library::LibTab::Artists.tab_id().to_string(),
                    });
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        local_library::LibTab::Artists,
                    );
                    // Re-derive in place so the filter applies on re-entry (the
                    // async first-load re-derives with the same filter on its own).
                    local_library::derive_artists(&w);
                    update_nav_flags(&w);
                    return;
                }
                if kind == "local" {
                    w.global::<LocalLibraryState>().set_tracks_search(q.clone().into());
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: local_library::LibTab::Tracks.tab_id().to_string(),
                    });
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        local_library::LibTab::Tracks,
                    );
                    // `navigate_local_library` only lazy-loads on an EMPTY tracks
                    // model (re-entry keeps the prior set), so force a reload to
                    // apply the freshly-set search filter regardless.
                    local_library::reload_tracks(weak.clone(), handle.clone());
                    update_nav_flags(&w);
                    return;
                }

                // Qobuz category → open the full results page on the matching tab.
                let tab = match kind.as_str() {
                    "album" => 1,
                    "track" => 2,
                    "artist" => 3,
                    "playlist" => 4,
                    _ => 0,
                };
                nav::push_or_replace_search(q.clone());
                navigate_search(runtime.clone(), weak.clone(), &handle, image_cache.clone(), q);
                // search_all loads every category; the tab switch only changes
                // which list renders. Apply it after navigate so it sticks.
                w.global::<SearchState>().set_tab(tab);
                update_nav_flags(&w);
            });
    }
}
