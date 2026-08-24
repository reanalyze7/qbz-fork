use crate::*;

/// Registers the single `on_appearance_select` callback; its match arms
/// live in `handle_appearance_select_a`/`_b` (appearance_select_a.rs /
/// appearance_select_b.rs), split out to stay under the 130-line file cap.
pub(crate) fn wire_appearance_select(window: &AppWindow) {
    let appearance = window.global::<AppearanceState>();
    let theme_weak = window.as_weak();
    let theme_handle = tokio::runtime::Handle::current();
    appearance.on_appearance_select(move |key, index| {
        handle_appearance_select_a(key.as_str(), index, &theme_weak);
        handle_appearance_select_b(key.as_str(), index, &theme_weak, &theme_handle);
    });
}
