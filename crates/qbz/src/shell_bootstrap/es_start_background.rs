use crate::*;

/// Start the playback poll loop + bind the exit-flush context, then load the
/// sidebar playlists list. Runs for the app lifetime; safe to start once per
/// shell entry.
pub(crate) fn es_start_background(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
) {
    // Start the playback poll loop — it runs for the app lifetime,
    // ticking position/progress onto NowPlayingState and auto-advancing
    // the queue on track end. Safe to start once per shell entry.
    playback::start_poll_loop(runtime.clone(), weak.clone(), tokio::runtime::Handle::current());
    // Bind the exit context so the window close handlers can flush a final
    // session snapshot before the loop quits (idempotent).
    session_persist::bind_exit_ctx(runtime.clone(), tokio::runtime::Handle::current());

    // Load the sidebar playlists list.
    load_sidebar_playlists(runtime.clone(), weak.clone(), &tokio::runtime::Handle::current());
}
