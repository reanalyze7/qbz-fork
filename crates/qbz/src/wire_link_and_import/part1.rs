use crate::*;

pub(crate) fn wire_link_and_import_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window.global::<LinkResolverActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<LinkResolverState>().set_open(false);
            }
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LinkResolverActions>()
            .on_open_importer(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<LinkResolverState>().set_open(false);
                    w.global::<PlaylistImportState>().set_open(true);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<LinkResolverActions>().on_submit(move |url| {
            let url = url.trim().to_string();
            if url.is_empty() {
                return;
            }
            if let Some(w) = weak.upgrade() {
                let s = w.global::<LinkResolverState>();
                s.set_resolving(true);
                s.set_error("".into());
                s.set_playlist_detected(false);
            }
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            let image_cache = image_cache.clone();
            handle.clone().spawn(async move {
                let result = link_resolver::resolve(runtime.clone(), url).await;
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let s = w.global::<LinkResolverState>();
                    s.set_resolving(false);
                    match result {
                        Ok(qbz_music_link::MusicLinkResult::Resolved { link, .. }) => {
                            s.set_open(false);
                            apply_resolved_link(
                                link,
                                &runtime,
                                &w.as_weak(),
                                &handle,
                                &image_cache,
                            );
                        }
                        Ok(qbz_music_link::MusicLinkResult::PlaylistDetected { provider }) => {
                            s.set_playlist_detected(true);
                            s.set_playlist_provider(provider.into());
                        }
                        Ok(qbz_music_link::MusicLinkResult::NotOnQobuz { .. }) => {
                            s.set_error(
                                qbz_i18n::t("This content is not available on Qobuz").into(),
                            );
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] open-link resolve failed: {e}");
                            s.set_error(qbz_i18n::t("Could not resolve that link").into());
                        }
                    }
                });
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<NavState>().on_request_back(move || {
            if let Some((entry, scroll)) = nav::go_back() {
                arm_scroll_restore(&weak, &entry, scroll);
                apply_entry(entry, &runtime, &weak, &handle, &image_cache);
            }
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<NavState>().on_request_forward(move || {
            if let Some((entry, scroll)) = nav::go_forward() {
                arm_scroll_restore(&weak, &entry, scroll);
                apply_entry(entry, &runtime, &weak, &handle, &image_cache);
            }
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
        });
    }
    {
        // The mounted scroll container reports its live viewport-y here so the
        // nav module can stamp the outgoing entry on the next navigation.
        window
            .global::<NavState>()
            .on_report_scroll(|y| nav::set_live_scroll(y));
    }
}
