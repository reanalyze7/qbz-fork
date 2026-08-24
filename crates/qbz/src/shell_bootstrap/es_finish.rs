use crate::*;

/// Seed the favorites tab counts, then drain a pending XDG deep-link URL
/// LAST — after the startup-page / view-restore, so the restore can't
/// re-root over the deep link.
pub(crate) async fn es_finish(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
) {
    // Seed the favorites tab counts so the badges are ready before the
    // user opens each tab (they otherwise only count on first visit).
    let counts = favorites::load_counts(&runtime).await;
    let _ = weak.upgrade_in_event_loop(move |w| {
        favorites::apply_counts(&w, counts);
    });

    // XDG deep link: drain a pending Qobuz URL LAST — after the startup-page
    // / view-restore block above, so the restore can't re-root over the deep
    // link (the navigation lands on top of whatever was restored). Session
    // active, AppWindow alive: no readiness sleep needed. Nothing pending =>
    // no-op. (Offline entries never reach here — enter_shell_offline keeps
    // the URL pending; navigation needs the API.)
    deep_link::drain_pending();
}
