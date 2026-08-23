//! Icon resolution, `NSImage` building, and applying/re-theming the status
//! item's icon.

use objc2::rc::Retained;
use objc2_app_kit::{NSImage, NSStatusItem};
use objc2_foundation::{MainThreadMarker, NSData, NSSize};

use super::STATUS_ITEM;

// 44px assets (= 22pt @2x menu bar). Filename trap (shared with Linux):
// `tray-dark-*` holds the WHITE glyph, `tray-light-*` holds the BLACK glyph.
const ICON_COLOR: &[u8] = include_bytes!("../../../icons/tray-color-44.png");
const ICON_WHITE: &[u8] = include_bytes!("../../../icons/tray-dark-44.png");
const ICON_BLACK: &[u8] = include_bytes!("../../../icons/tray-light-44.png");

/// Resolve the icon bytes + whether to render it as a macOS template image
/// (template = adapts to the light/dark menu bar automatically).
/// - "color"      → full vinyl, not a template
/// - "mono-light" → white glyph (`tray-dark`), not a template
/// - "mono-dark"  → black glyph (`tray-light`), not a template
/// - "auto"/other → black glyph as a template, so macOS adapts it
fn icon_for(theme: &str) -> (&'static [u8], bool) {
    match theme {
        "color" => (ICON_COLOR, false),
        "mono-light" => (ICON_WHITE, false),
        "mono-dark" => (ICON_BLACK, false),
        _ => (ICON_BLACK, true),
    }
}

/// Build an `NSImage` from PNG bytes, marking it a template image when asked.
fn make_image(bytes: &[u8], is_template: bool) -> Option<Retained<NSImage>> {
    let data = NSData::with_bytes(bytes);
    let image = NSImage::initWithData(NSImage::alloc(), &data)?;
    unsafe { image.setTemplate(is_template) };
    // The PNG assets are 44px (22pt @2x). Without an explicit point size the
    // menu bar renders them at native pixel size → a giant icon. Pin to the
    // standard menu-bar glyph box (18pt; the bar is 22pt tall).
    unsafe { image.setSize(NSSize::new(18.0, 18.0)) };
    Some(image)
}

/// Apply the resolved icon to the status item's button.
pub(super) fn apply_icon(status_item: &NSStatusItem, theme: &str, mtm: MainThreadMarker) {
    let (bytes, is_template) = icon_for(theme);
    let Some(image) = make_image(bytes, is_template) else {
        log::error!("[tray] failed to decode menu-bar icon");
        return;
    };
    if let Some(button) = unsafe { status_item.button(mtm) } {
        unsafe { button.setImage(Some(&image)) };
    }
}

/// Re-theme the live menu-bar icon (called on the main thread).
pub fn set_icon_theme(theme: &str) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    STATUS_ITEM.with(|s| {
        if let Some(status_item) = s.borrow().as_ref() {
            apply_icon(status_item, theme, mtm);
        }
    });
}
