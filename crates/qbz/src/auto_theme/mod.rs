//! Auto-theme controller: wires the Settings "Auto (dynamic)" theme option to
//! `qbz_theme::auto` generation.
//!
//! Generation (DE probing + k-means over the wallpaper/image) runs off the event
//! loop via `spawn_blocking`; the palette is pushed back through
//! `crate::theme::push_colors` (the same path static themes use) on the event
//! loop. On failure the app logs, toasts, and (at startup) falls back to the
//! default OLED theme.
//!
//! Deviation vs Tauri: Tauri regenerated the wallpaper theme reactively; here v1
//! regenerates on activation, on source change, on image pick, and via the
//! explicit "Regenerate" button — there is no live wallpaper file-watcher.

mod interactive;

pub use interactive::{regenerate, select_image, set_source};

use crate::AppWindow;
use crate::AppearanceState;
use qbz_theme::AutoSource;
use slint::ComponentHandle;

/// Build an [`AutoSource`] from the persisted preferences.
pub(super) fn source_from_prefs(prefs: &crate::ui_prefs::UiPrefs) -> AutoSource {
    match prefs.auto_theme_source.as_str() {
        "wallpaper" => AutoSource::Wallpaper,
        "image" => AutoSource::Image(prefs.auto_theme_image_path.clone()),
        _ => AutoSource::System,
    }
}

/// Human-readable detected desktop environment (for the Settings "Detected: …"
/// hint row).
pub fn detected_de() -> String {
    qbz_theme::auto::detect_desktop_environment()
        .display_name()
        .to_string()
}

/// Seed the auto-theme Settings state (source index, custom path, detected DE)
/// from the persisted prefs. Called at startup so the controls reflect the saved
/// source when the user opens Settings.
pub fn seed_state(window: &AppWindow) {
    let prefs = crate::ui_prefs::load();
    let state = window.global::<AppearanceState>();
    state.set_auto_theme_source_index(crate::ui_prefs::auto_theme_source_index(
        &prefs.auto_theme_source,
    ));
    state.set_auto_theme_custom_path(prefs.auto_theme_image_path.clone().into());
    state.set_auto_theme_detected_de(detected_de().into());
    state.set_auto_theme_generating(false);
}

/// Synchronous startup apply: generate from the persisted source and push the
/// palette, or fall back to the default (OLED) theme on failure. Runs inline on
/// the event-loop thread during window init so the first paint is already the
/// generated palette.
pub fn apply_startup(window: &AppWindow) {
    let prefs = crate::ui_prefs::load();
    let source = source_from_prefs(&prefs);
    match qbz_theme::generate_auto_theme(&source) {
        Ok(colors) => {
            crate::theme::push_colors(window, &colors, false, false);
            log::info!(
                "[qbz-slint] applied auto theme (source={})",
                prefs.auto_theme_source
            );
        }
        Err(e) => {
            log::warn!(
                "[qbz-slint] auto theme generation failed at startup: {e}; falling back to default"
            );
            crate::theme::apply_theme(window, qbz_theme::default_theme_id());
            crate::toast::error(window, qbz_i18n::t("Auto theme generation failed"));
        }
    }
}
