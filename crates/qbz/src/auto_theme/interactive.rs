//! Interactive auto-theme flows: regeneration, image picking, and source
//! switching. These spawn async work off the event loop and push results back
//! via `upgrade_in_event_loop`.

use super::source_from_prefs;
use crate::AppWindow;
use crate::AppearanceState;
use slint::ComponentHandle;

/// Regenerate the auto theme off-thread and push the result on the event loop.
/// Toggles `auto-theme-generating` around the work and toasts on failure.
pub fn regenerate(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    // Through the event loop, NOT a direct upgrade: regenerate() is also
    // called from tokio contexts (select_image), where a plain upgrade()
    // returns None and the generating indicator would silently be skipped.
    let weak_flag = weak.clone();
    let _ = weak_flag.upgrade_in_event_loop(move |w| {
        w.global::<AppearanceState>().set_auto_theme_generating(true);
    });
    handle.spawn(async move {
        let prefs = crate::ui_prefs::load();
        let source = source_from_prefs(&prefs);
        let result = tokio::task::spawn_blocking(move || qbz_theme::generate_auto_theme(&source))
            .await
            .unwrap_or_else(|e| Err(format!("auto theme task panicked: {e}")));

        let _ = weak.upgrade_in_event_loop(move |w| {
            w.global::<AppearanceState>()
                .set_auto_theme_generating(false);
            match result {
                Ok(colors) => crate::theme::push_colors(&w, &colors, false, false),
                Err(e) => {
                    log::warn!("[qbz-slint] auto theme regeneration failed: {e}");
                    crate::toast::error(&w, qbz_i18n::t("Auto theme generation failed"));
                }
            }
        });
    });
}

/// Open the native image picker; on selection persist it as the `image` source
/// and regenerate. Cancel is a no-op (no toast).
pub fn select_image(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let regen_handle = handle.clone();
    handle.spawn(async move {
        let Some(file) = rfd::AsyncFileDialog::new()
            .set_title(&qbz_i18n::t("Select Image..."))
            .add_filter(
                &qbz_i18n::t("Image"),
                &["png", "jpg", "jpeg", "webp", "bmp", "tiff"],
            )
            .pick_file()
            .await
        else {
            return; // user cancelled
        };
        let path = file.path().to_string_lossy().to_string();

        // Persist source=image + path before regenerating (regenerate re-reads).
        let mut prefs = crate::ui_prefs::load();
        prefs.auto_theme_source = "image".to_string();
        prefs.auto_theme_image_path = path.clone();
        crate::ui_prefs::save(&prefs);

        // Reflect the new source into the Settings controls.
        let ui_path = path.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let st = w.global::<AppearanceState>();
            st.set_auto_theme_custom_path(ui_path.into());
            st.set_auto_theme_source_index(crate::ui_prefs::auto_theme_source_index("image"));
        });

        regenerate(weak, regen_handle);
    });
}

/// Persist a new auto-theme source (from the source dropdown) and regenerate.
pub fn set_source(index: i32, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let key = crate::ui_prefs::auto_theme_source_for_index(index);
    let mut prefs = crate::ui_prefs::load();
    prefs.auto_theme_source = key.to_string();
    crate::ui_prefs::save(&prefs);
    regenerate(weak, handle);
}
