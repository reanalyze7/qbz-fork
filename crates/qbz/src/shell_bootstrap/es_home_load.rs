use crate::*;

/// Load Audio + Playback settings into the Settings page in the background,
/// then load the genre-filter parents + persisted selection (before the
/// discover load, so the first fetch honors a remembered genre selection),
/// then load Home.
pub(crate) async fn es_home_load(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    settings_ctx: Arc<settings::SettingsCtx>,
) {
    // Load Audio + Playback settings into the Settings page in the
    // background — store reads and device enumeration are blocking.
    spawn_settings_snapshot_load(runtime.clone(), weak.clone(), settings_ctx.clone());

    // Load the genre-filter parents + persisted selection, then seed
    // the popup state. Done before the discover load so the first
    // fetch honors a remembered genre selection.
    genre_filter::load_parents(&runtime).await;
    let _ = weak.upgrade_in_event_loop(|w| {
        genre_filter::apply_state(&w);
    });

    reload_home(&runtime, &weak, &image_cache, "home".to_string()).await;
}
