use crate::*;

pub(crate) fn wire_offline_and_auth_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>, login_task: &Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>) {

    // D2 recovery: one click on the shell banner re-logs-in with the saved
    // token and runs the full online entry over the live offline session.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let settings_ctx = settings_ctx.clone();
        let login_task = login_task.clone();
        window.on_recovery_login(move || {
            // Logged BEFORE the spawn: records the click arriving from the
            // UI chain even if the async attempt below stalls or fails.
            log::info!("[qbz-slint] recovery sign-in requested");
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            let settings_ctx = settings_ctx.clone();
            let task = handle.spawn(async move {
                // No pre-lift anywhere: the auth endpoints are EXEMPT from
                // the offline gate (qbz-qobuz client), so the token login and
                // the OAuth exchange pass the closed gate — and
                // login_via_system_browser no longer clears offline_session
                // up front either. The flag ends up false only on SUCCESS
                // paths (restore_saved_session / login_via_system_browser
                // clear it after the login completes), so the shell never
                // sits unlocked-and-empty while an attempt is pending, and a
                // failed attempt leaves the live offline session intact.
                match auth::restore_saved_session(&runtime).await {
                    Ok(Some(session)) => {
                        log::info!(
                            "[qbz-slint] recovery login succeeded for user {}",
                            session.user_id
                        );
                        enter_shell(runtime, weak, image_cache, settings_ctx, session).await;
                    }
                    Ok(None) => {
                        // No saved token, or the token was explicitly
                        // rejected (and cleared). The user asked to sign in —
                        // fall back to the full system-browser OAuth. Show
                        // the LOGIN screen FIRST: its UX narrates the
                        // browser flow (the user shouldn't have to notice
                        // the opened browser on their own), and it replaces
                        // the offline shell instead of leaving it on screen
                        // while the attempt runs.
                        log::warn!(
                            "[qbz-slint] recovery login: saved session unusable — falling back to browser OAuth"
                        );
                        let _ = weak.upgrade_in_event_loop(|w| {
                            // Seed the waiting narration before the browser
                            // opens so the screen never shows an idle button
                            // while the flow is already running.
                            let login_state = w.global::<LoginState>();
                            login_state.set_error("".into());
                            login_state.set_phase(1);
                            w.set_screen(AppScreen::Login);
                        });
                        let phase_weak = weak.clone();
                        let login_result =
                            auth::login_via_system_browser(&runtime, move |phase| {
                                let value = match phase {
                                    auth::LoginPhase::WaitingForBrowser => 1,
                                    auth::LoginPhase::Authenticating => 2,
                                };
                                let _ = phase_weak.upgrade_in_event_loop(move |w| {
                                    w.global::<LoginState>().set_phase(value);
                                });
                            })
                            .await;
                        match login_result {
                            Ok(session) => {
                                log::info!(
                                    "[qbz-slint] recovery browser sign-in succeeded for user {}",
                                    session.user_id
                                );
                                enter_shell(runtime, weak, image_cache, settings_ctx, session)
                                    .await;
                            }
                            Err(e) => {
                                log::error!("[qbz-slint] recovery browser sign-in failed: {e}");
                                // The offline session was never lifted, so
                                // there is nothing to restore. Stay on the
                                // Login screen: the error box explains the
                                // failure, and the "Start offline" link
                                // (has-previous-session) leads back into
                                // the offline shell.
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    toast::error(&w, format!("Sign-in failed: {e}"));
                                    let login_state = w.global::<LoginState>();
                                    login_state.set_phase(0);
                                    login_state.set_error(e.into());
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Init-class failure (gated/unreachable cold bundle
                        // fetch): any transient flag lift was already undone
                        // inside auth, so the offline shell state is intact —
                        // just surface the error.
                        log::error!("[qbz-slint] recovery login failed: {e}");
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            toast::error(&w, format!("Sign-in failed: {e}"));
                            w.global::<OfflineState>().set_login_error(e.into());
                        });
                    }
                }
            });
            // Same slot the login screen's Cancel link aborts — the browser
            // leg of this recovery flow is cancellable like a normal sign-in.
            *login_task.lock().unwrap() = Some(task);
        });
    }
}
