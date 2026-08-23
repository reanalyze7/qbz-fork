//! Slint-facing glue: decoding the custom icon image, pushing to
//! `MyQbzBrandingState`, and the async native file-picker flow.

use slint::ComponentHandle;

use crate::{AppWindow, MyQbzBrandingState};

use super::actions::set_icon_path;
use super::store::read;
use super::DEFAULT_LABEL;

/// Resolve a (label, custom_icon) pair for the UI from the persisted
/// branding. `custom_icon` is `Some` only when a custom path is set AND the
/// file loads; a missing path or a stale / deleted file yields `None` (the
/// markup then falls back to the default branded glyph). A load failure does
/// NOT mutate the store — the user can re-pick, and the path is preserved in
/// case the file returns.
fn resolve() -> (String, Option<slint::Image>) {
    let b = read();
    if b.icon_path.is_empty() {
        return (b.label, None);
    }
    match slint::Image::load_from_path(std::path::Path::new(&b.icon_path)) {
        Ok(img) => (b.label, Some(img)),
        Err(e) => {
            log::warn!(
                "[qbz-slint] myqbz branding: custom icon '{}' failed to load, using default: {e}",
                b.icon_path
            );
            (b.label, None)
        }
    }
}

/// Push the persisted branding onto `MyQbzBrandingState`. Runs on the UI
/// thread (it touches the Slint global + decodes an image). Call on shell
/// entry and after every set/reset so the sidebar row reflects the change.
///
/// The default glyph stays a compile-time `@image-url` in the markup; Rust
/// only supplies the custom image (and the flag that selects it).
pub fn seed(window: &AppWindow) {
    let (label, custom_icon) = resolve();
    let st = window.global::<MyQbzBrandingState>();
    st.set_label(label.into());
    match custom_icon {
        Some(img) => {
            st.set_custom_icon(img);
            st.set_has_custom_icon(true);
        }
        None => {
            // Clear any stale custom image and fall back to the default glyph.
            st.set_custom_icon(slint::Image::default());
            st.set_has_custom_icon(false);
        }
    }
}

/// Re-seed the branding state on the UI thread via a weak handle. Used by the
/// async icon picker once it has persisted a new path.
pub fn reseed_weak(weak: &slint::Weak<AppWindow>) {
    let _ = weak.upgrade_in_event_loop(|w| seed(&w));
}

/// Open the native image picker; on pick, persist the chosen path and re-seed
/// the branding state (sidebar row + Settings preview reflect it). No-op on
/// cancel. The filter matches the Tauri modal's set (`svg, png, jpg, jpeg,
/// webp`).
pub fn pick_icon(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let Some(file) = rfd::AsyncFileDialog::new()
            .set_title(&qbz_i18n::t_args("Choose a {} icon", &[DEFAULT_LABEL]))
            .add_filter(&qbz_i18n::t("Image"), &["svg", "png", "jpg", "jpeg", "webp"])
            .pick_file()
            .await
        else {
            return; // cancelled — leave branding unchanged.
        };
        let path = file.path().to_string_lossy().to_string();
        set_icon_path(&path);
        reseed_weak(&weak);
    });
}
