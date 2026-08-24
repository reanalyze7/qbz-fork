use crate::*;

/// Auto-theme actions (image picker + explicit Regenerate) and the
/// custom-theme editor callbacks (per-token live edits, polarity toggle,
/// "start from current theme"). Each re-derives the whole palette in Rust
/// and pushes it live (derivation is cheap).
pub(crate) fn wire_appearance_action_custom(window: &AppWindow) {
    let appearance = window.global::<AppearanceState>();
        // Auto-theme actions: image picker + explicit Regenerate button. Both run
        // generation off the event loop and push the palette back on it.
        let action_weak = window.as_weak();
        let action_handle = tokio::runtime::Handle::current();
        appearance.on_appearance_action(move |key| match key.as_str() {
            "auto-theme-select-image" => {
                crate::auto_theme::select_image(action_weak.clone(), action_handle.clone());
            }
            "auto-theme-regenerate" => {
                crate::auto_theme::regenerate(action_weak.clone(), action_handle.clone());
            }
            other => log::debug!("[qbz-slint] unhandled appearance-action '{other}'"),
        });

        // Custom-theme editor callbacks: per-token live edits (drag + hex),
        // polarity toggle, and "start from current theme". Each re-derives the
        // whole palette in Rust and pushes it live (derivation is cheap).
        let ct_weak = window.as_weak();
        appearance.on_custom_set_token(move |key, color| {
            if let Some(w) = ct_weak.upgrade() {
                crate::custom_theme::set_token(&w, key.as_str(), color);
            }
        });
        let ct_hex_weak = window.as_weak();
        appearance.on_custom_set_token_hex(move |key, hex| {
            if let Some(w) = ct_hex_weak.upgrade() {
                crate::custom_theme::set_token_hex(&w, key.as_str(), hex.as_str());
            }
        });
        let ct_dark_weak = window.as_weak();
        appearance.on_custom_toggle_dark(move |is_dark| {
            if let Some(w) = ct_dark_weak.upgrade() {
                crate::custom_theme::toggle_dark(&w, is_dark);
            }
        });
        let ct_seed_weak = window.as_weak();
        appearance.on_custom_seed_from_current(move || {
            if let Some(w) = ct_seed_weak.upgrade() {
                crate::custom_theme::seed_from_current(&w);
            }
        });
}
