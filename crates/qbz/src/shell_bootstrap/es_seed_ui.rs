use crate::*;

/// Init per-user shell wiring + bind the deep-link context, then seed the
/// session/appearance/pinned state and switch to the shell screen.
pub(crate) fn es_seed_ui(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    session: auth::SessionInfo,
) {
    let tray = init_shell_for_user(&runtime, &weak, session.user_id);

    // Deep links (argv capture / warm D-Bus OpenUrl) may now drain: the
    // session is active and the AppWindow exists. Bound here at the top; the
    // pending URL itself is dispatched at the very END of this function so
    // the startup-page/view restore below can't re-root over the deep link.
    deep_link::bind_shell_ctx(
        runtime.clone(),
        weak.clone(),
        tokio::runtime::Handle::current(),
        image_cache.clone(),
    );

    let _ = weak.upgrade_in_event_loop(move |w| {
        let state = w.global::<SessionState>();
        state.set_user_name(session.display_name.into());
        state.set_subscription(session.subscription.into());
        // A successful login means a previous session now exists; clear any
        // stale boot restore error from the login screen.
        let offline_state = w.global::<OfflineState>();
        offline_state.set_has_previous_session(true);
        offline_state.set_login_error("".into());
        // Reset the browser sign-in narration for the next visit to the
        // login screen (logout → login).
        let login_state = w.global::<LoginState>();
        login_state.set_phase(0);
        login_state.set_error("".into());
        seed_tray_appearance(&w, &tray);
        // Seed the My QBZ branding (label + icon) from the per-user store so
        // the sidebar row + Settings row paint the custom values immediately.
        myqbz_prefs::seed(&w);
        // Seed the Discover configurator descriptor lists so the prefs-driven
        // render loop has order/visibility data before the first apply_home.
        discover_prefs::seed(&w);
        // Seed the Pinned section (Home / For You) from the per-user pinned
        // store — bound by perform_login / restore before this closure runs.
        pinned_section::rebuild_pinned(&w);
        w.global::<HomeState>().set_loading(true);
        w.set_screen(AppScreen::Shell);
    });
}
