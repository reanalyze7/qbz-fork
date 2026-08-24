use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // "Use my own music": start the read-back without queuing the test
        // tracks — the user plays whatever they want; the poll reads the rate.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_verify_own(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            let runtime = runtime.clone();
            let weak2 = w.as_weak();
            handle.spawn(async move {
                // Guardrail: don't start a read-back on an empty queue.
                let (tracks, _) = runtime.core().get_all_queue_tracks().await;
                let empty = tracks.is_empty();
                if !empty {
                    let _ = runtime.core().resume();
                }
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    if empty {
                        dac_wizard::queue_empty_notice(&w);
                    } else {
                        dac_wizard::begin_test(&w);
                    }
                });
            });
        });
    }
    {
        // Generate the per-DAC copy-paste config (Slice 10): re-probe rates off
        // the UI thread, then fill the review step.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_gen_configs(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            let dacs = dac_wizard::checked_dacs(&w);
            let weak2 = w.as_weak();
            handle.spawn_blocking(move || {
                let data = dac_wizard::gen_configs_blocking(dacs);
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_configs(&w, data);
                });
            });
        });
    }
    {
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_toggle_config(move |i| {
            if let Some(w) = weak.upgrade() {
                dac_wizard::toggle_config(&w, i);
            }
        });
    }
    {
        window
            .global::<DacWizardActions>()
            .on_copy_command(move |cmd| {
                share::copy_to_clipboard(cmd.to_string());
            });
    }

    // ---- Sandbox (Flatpak/Snap) settings section ----
    // Seed the install method once (drives section visibility) and wire the
    // copy-to-clipboard action for the permission commands.
    {
        let method = qbz_app::diagnostics::system_info().install_method;
        window.global::<SandboxState>().set_install_method(method.into());
        window
            .global::<SandboxState>()
            .on_copy_command(move |cmd| {
                share::copy_to_clipboard(cmd.to_string());
            });
    }

    // ---- Playlist Importer (public playlists) — spec §3.3 ----
    {
        // No cancel exists: a running import task continues to completion
        // (§1.8); closing only hides the modal.
        let weak = window.as_weak();
        window.global::<PlaylistImportActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<PlaylistImportState>().set_open(false);
            }
        });
    }
    {
        // Provider detection per keystroke (Slint 1.16 has no `.contains`).
        let weak = window.as_weak();
        window
            .global::<PlaylistImportActions>()
            .on_url_edited(move |text| {
                if let Some(w) = weak.upgrade() {
                    playlist_import::on_url_edited(&w, text.as_str());
                }
            });
    }
    {
        window
            .global::<PlaylistImportActions>()
            .on_name_edited(move |text| {
                playlist_import::on_name_edited(text.as_str());
            });
    }
}
