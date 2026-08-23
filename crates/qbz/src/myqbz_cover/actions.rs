//! Entry points wired from the My QBZ hero overflow menu.

use slint::ComponentHandle;

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::file_ops::{do_remove, do_upload};

/// Hero overflow "Set custom cover": open the native image picker, then upload
/// + resize + persist + reload. Toasts success/failure (spec §10 item 3).
pub fn upload(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
) {
    handle.clone().spawn(async move {
        let Some(file) = rfd::AsyncFileDialog::new()
            .set_title(&qbz_i18n::t("Choose a cover image"))
            .add_filter(&qbz_i18n::t("Image"), &["png", "jpg", "jpeg", "webp"])
            .pick_file()
            .await
        else {
            return; // user cancelled — no toast.
        };
        let source = file.path().to_string_lossy().to_string();

        let upload_id = id.clone();
        let result = tokio::task::spawn_blocking(move || do_upload(&upload_id, &source))
            .await
            .unwrap_or_else(|e| Err(format!("upload task panicked: {e}")));

        match result {
            Ok(_) => {
                crate::toast::success_weak(&weak, qbz_i18n::t("Cover updated"));
                reload(weak.clone(), handle.clone(), image_cache.clone(), id);
            }
            Err(e) => {
                log::warn!("[qbz-slint] myqbz_cover upload failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Failed to upload cover"));
            }
        }
    });
}

/// Hero overflow "Remove custom cover": clear + delete the file + reload.
pub fn remove(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
) {
    handle.clone().spawn(async move {
        let remove_id = id.clone();
        let result = tokio::task::spawn_blocking(move || do_remove(&remove_id))
            .await
            .unwrap_or_else(|e| Err(format!("remove task panicked: {e}")));

        match result {
            Ok(()) => {
                crate::toast::success_weak(&weak, qbz_i18n::t("Cover removed"));
                reload(weak.clone(), handle.clone(), image_cache.clone(), id);
            }
            Err(e) => {
                log::warn!("[qbz-slint] myqbz_cover remove failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Failed to remove cover"));
            }
        }
    });
}

/// Reload the open detail view so the hero reflects the new cover. Re-runs the
/// detail navigator's load/apply/artwork path for the same id (Tauri's
/// "-> reload"). The `set_view` inside `navigate` is harmless (we're already on
/// the detail view).
fn reload(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
) {
    let Some(runtime) = crate::myqbz_detail::global_runtime() else { return };
    let _ = weak.upgrade_in_event_loop(move |w| {
        let _ = &w; // keep the closure's capture explicit.
        crate::myqbz_detail::navigate(runtime, w.as_weak(), handle.clone(), image_cache.clone(), id);
    });
}
