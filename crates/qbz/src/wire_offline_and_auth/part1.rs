use crate::*;

pub(crate) fn wire_offline_and_auth_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>, login_task: &Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>) {
    let on_browser_login = {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let settings_ctx = settings_ctx.clone();
        let login_task = login_task.clone();
        move || {
            // Runs on the UI thread: flip to the waiting state immediately so
            // a second click cannot start a parallel flow, and clear any
            // previous failure.
            if let Some(w) = weak.upgrade() {
                let login_state = w.global::<LoginState>();
                if login_state.get_phase() != 0 {
                    return;
                }
                login_state.set_error("".into());
                // A new sign-in supersedes any stale boot-restore error —
                // without this both error boxes can show at once (review fix).
                w.global::<OfflineState>().set_login_error("".into());
                login_state.set_phase(1);
            }
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            let settings_ctx = settings_ctx.clone();
            let task = handle.spawn(async move {
                let phase_weak = weak.clone();
                let result = auth::login_via_system_browser(&runtime, move |phase| {
                    let value = match phase {
                        auth::LoginPhase::WaitingForBrowser => 1,
                        auth::LoginPhase::Authenticating => 2,
                    };
                    let _ = phase_weak.upgrade_in_event_loop(move |w| {
                        w.global::<LoginState>().set_phase(value);
                    });
                })
                .await;
                match result {
                    Ok(session) => {
                        log::info!(
                            "[qbz-slint] authenticated as user {}",
                            session.user_id
                        );
                        enter_shell(runtime, weak, image_cache, settings_ctx, session).await;
                    }
                    Err(e) => {
                        log::error!("[qbz-slint] sign-in failed: {e}");
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let login_state = w.global::<LoginState>();
                            login_state.set_phase(0);
                            login_state.set_error(e.into());
                        });
                    }
                }
            });
            *login_task.lock().unwrap() = Some(task);
        }
    };

    {
        let login = on_browser_login.clone();
        window.on_sign_in_via_browser(move || {
            dispatch(AppCommand::SignInViaBrowser);
            login();
        });
    }

    // Cancel link on the login screen (visible only while waiting for the
    // browser): abort the pending OAuth task and return to idle. Aborting
    // drops the local listener; the browser tab just fails to redirect.
    {
        let weak = window.as_weak();
        let login_task = login_task.clone();
        window.on_cancel_login(move || {
            if let Some(task) = login_task.lock().unwrap().take() {
                task.abort();
            }
            if let Some(w) = weak.upgrade() {
                let login_state = w.global::<LoginState>();
                login_state.set_phase(0);
                login_state.set_error("".into());
            }
            log::info!("[qbz-slint] browser sign-in cancelled by user");
        });
    }

    // Offline: enter a full offline session at the last user (local library,
    // offline cache, settings — no Qobuz auth), then show the shell.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let settings_ctx = settings_ctx.clone();
        window.on_start_offline(move || {
            dispatch(AppCommand::StartOffline);
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            let settings_ctx = settings_ctx.clone();
            handle.spawn(async move {
                if let Err(e) = enter_shell_offline(runtime, weak, image_cache, settings_ctx).await
                {
                    log::error!("[qbz-slint] offline start failed: {e}");
                }
            });
        });
    }
}
