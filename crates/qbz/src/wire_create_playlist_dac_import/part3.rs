use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Self-service playback test (Slice 9): resolve the 4 curated tracks
        // (id-hint then "artist title" search), route output to the DAC under
        // test, and play them. The N6 read-back is driven by on_poll_test.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_start_test(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            dac_wizard::begin_test(&w);
            let runtime = runtime.clone();
            let weak2 = w.as_weak();
            let play_handle = handle.clone();
            handle.spawn(async move {
                let mut tracks: Vec<qbz_models::Track> = Vec::new();
                for seed in dac_wizard::TEST_SEEDS.iter() {
                    let mut chosen = match runtime.core().get_track(seed.id_hint).await {
                        Ok(t) if dac_wizard::track_matches_seed(&t, seed) => Some(t),
                        _ => None,
                    };
                    if chosen.is_none() {
                        let q = format!("{} {}", seed.artist, seed.title);
                        if let Ok(page) = runtime.core().search_tracks(&q, 10, 0, None).await {
                            chosen = page
                                .items
                                .into_iter()
                                .find(|t| dac_wizard::track_matches_seed(t, seed));
                        }
                    }
                    if let Some(t) = chosen {
                        tracks.push(t);
                    }
                }
                // Keep the resolved tracks so the user can jump between them.
                dac_wizard::stash_test_tracks(tracks.clone());
                let runtime2 = runtime.clone();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    if tracks.is_empty() {
                        w.global::<DacWizardState>()
                            .set_test_requested_label("Couldn't load the test tracks (offline?)".into());
                        return;
                    }
                    crate::playback::play_tracks(runtime2, w.as_weak(), play_handle, tracks, 0);
                });
            });
        });
    }
    {
        // Poll the requested vs negotiated rate while the test plays.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_poll_test(move || {
            if weak.upgrade().is_none() {
                return;
            }
            let runtime = runtime.clone();
            let weak2 = weak.clone();
            handle.spawn_blocking(move || {
                let player = runtime.core().player();
                let req_rate = player.state.get_sample_rate();
                let req_bits = player.state.get_bit_depth();
                let negotiated = qbz_audio::negotiated_active_rate();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_poll(&w, req_rate, req_bits, negotiated);
                });
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_stop_test(move || {
            let _ = runtime.core().pause();
            if let Some(w) = weak.upgrade() {
                dac_wizard::end_test(&w);
            }
        });
    }
    {
        // Jump straight to one of the test tracks (skip the long waits) by
        // re-setting the queue at that index via the working play path.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<DacWizardActions>()
            .on_test_play_index(move |i| {
                let tracks = dac_wizard::test_tracks();
                if tracks.is_empty() {
                    return;
                }
                let start = (i.max(0) as usize).min(tracks.len().saturating_sub(1));
                crate::playback::play_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    tracks,
                    start,
                );
            });
    }
}
