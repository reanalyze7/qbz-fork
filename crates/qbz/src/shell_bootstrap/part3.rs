use crate::*;

/// Reveal the shell and load the Discover / Home view with real data, then
/// kick off cached artwork downloads. Split into `es_*` sub-functions (this
/// dir's `es_*.rs`), awaited in original sequence — since each step's side
/// effects (spawns, `upgrade_in_event_loop` closures, state mutations) only
/// ever depend on what already ran before it, awaiting them one after
/// another here is exactly equivalent to the original inline body.
pub(crate) async fn enter_shell(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    settings_ctx: Arc<settings::SettingsCtx>,
    session: auth::SessionInfo,
) {
    es_seed_ui(
        runtime.clone(),
        weak.clone(),
        image_cache.clone(),
        session,
    );
    es_start_background(runtime.clone(), weak.clone());
    es_warm_caches(runtime.clone(), weak.clone());
    es_home_load(
        runtime.clone(),
        weak.clone(),
        image_cache.clone(),
        settings_ctx,
    )
    .await;
    es_restore_session(runtime.clone(), weak.clone()).await;
    es_restore_startup_page(runtime.clone(), weak.clone(), image_cache).await;
    es_finish(runtime, weak).await;
}
