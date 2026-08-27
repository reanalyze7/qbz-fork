//! Small business-logic layer over `store.rs`'s `read`/`write`.

use super::store::{read, write};
use super::DEFAULT_LABEL;

/// Coerce a raw label input to the persisted value: trimmed-empty → the
/// default "My Qoqobuz" (and the default string is what's stored).
pub(super) fn coerce_label(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        DEFAULT_LABEL.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Persist a new label (empty/whitespace coerces to the default) and return
/// the coerced value the sidebar should display.
pub fn set_label(label: &str) -> String {
    let mut b = read();
    b.label = coerce_label(label);
    write(&b);
    b.label
}

/// Persist a custom icon path. An empty / whitespace path clears the custom
/// icon (reset to default), mirroring `setMyQbzIconPath(null)`.
pub fn set_icon_path(path: &str) {
    let mut b = read();
    b.icon_path = path.trim().to_string();
    write(&b);
}

/// Reset the icon to the default branded glyph (clears the persisted path).
pub fn reset_icon() {
    set_icon_path("");
}
