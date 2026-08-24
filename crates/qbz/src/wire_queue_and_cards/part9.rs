use crate::*;

pub(crate) fn wire_queue_and_cards_part9(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    // Musician click — resolve the (name, role) first; if Qobuz has
    // a confirmed exact match, jump straight to that artist's page.
    // Otherwise open MusicianPageView (Contextual / Weak / None).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<NetworkSidebarActions>()
            .on_musician_clicked(move |name, role| {
                let name = name.to_string();
                let role = role.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                tokio::spawn(async move {
                    let resolved =
                        runtime.core().musicbrainz_resolve_musician(&name, &role).await;
                    match resolved {
                        Ok(r) if matches!(
                            r.confidence,
                            qbz_integrations::musicbrainz::MusicianConfidence::Confirmed
                        ) =>
                        {
                            if let Some(id) = r.qobuz_artist_id {
                                let artist_id = id.to_string();
                                let weak2 = weak.clone();
                                let _ = weak.clone().upgrade_in_event_loop(move |_| {
                                    nav::record(nav::NavEntry::Artist(artist_id.clone()));
                                });
                                navigate_artist(
                                    runtime,
                                    weak2,
                                    &handle,
                                    image_cache,
                                    id.to_string(),
                                );
                                return;
                            }
                            log::warn!(
                                "[qbz-slint] musician confirmed but no qobuz id"
                            );
                        }
                        Ok(_) => {
                            // Fall through to MusicianPageView for
                            // Contextual / Weak / None.
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] musician resolve failed: {e}");
                        }
                    }
                    nav::record(nav::NavEntry::Musician {
                        name: name.clone(),
                        role: role.clone(),
                    });
                    navigate_musician(runtime, weak, &handle, image_cache, name, role);
                });
            });
    }
    // discovery-dismissed — persist the rejection under the current
    // tag, then remove the row from the visible list.
    {
        let weak = window.as_weak();
        window
            .global::<NetworkSidebarActions>()
            .on_discovery_dismissed(move |mbid, name| {
                if let Some(w) = weak.upgrade() {
                    let tag = w
                        .global::<NetworkSidebarState>()
                        .get_discovery_tag()
                        .to_string()
                        .to_lowercase();
                    if !tag.is_empty() {
                        let normalized =
                            qbz_core::normalize_artist_name(name.as_str());
                        discovery_dismiss::dismiss(&tag, &normalized);
                    }
                    artist::remove_discovery_artist(&w, mbid.as_str());
                }
            });
    }
}
