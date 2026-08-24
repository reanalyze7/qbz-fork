use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part2(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // ---- HiFi Wizard (DAC setup) — Slice 6 (check step) ----
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_open(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            dac_wizard::open_immediate(&w);
            // Probe the audio stack off the UI thread; fill the check step when done.
            let weak2 = w.as_weak();
            handle.spawn_blocking(move || {
                let health = qbz_audio::audio_stack_health();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_health(&w, health);
                });
            });
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DacWizardActions>()
            .on_set_distro(move |index| {
                if let Some(w) = weak.upgrade() {
                    dac_wizard::set_distro(&w, index);
                }
            });
    }
    {
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_set_init(move |index| {
            if let Some(w) = weak.upgrade() {
                dac_wizard::set_init(&w, index);
            }
        });
    }
    {
        // Enumerate DACs (Slice 7) off the UI thread when entering the step.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_run_detect(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            dac_wizard::begin_detect(&w);
            let weak2 = w.as_weak();
            handle.spawn_blocking(move || {
                let data = dac_wizard::detect_blocking();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_candidates(&w, data);
                });
            });
        });
    }
    {
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_toggle_dac(move |i| {
            if let Some(w) = weak.upgrade() {
                dac_wizard::toggle_dac(&w, i);
            }
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DacWizardActions>()
            .on_validate_manual(move |t| {
                if let Some(w) = weak.upgrade() {
                    dac_wizard::validate_manual(&w, t.as_str());
                }
            });
    }
}
