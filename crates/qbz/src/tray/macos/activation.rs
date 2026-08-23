//! macOS activation-policy helpers (Dock icon visibility, active-app status).

use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

/// Force the app to a Regular, active application so macOS dispatches the
/// `NSStatusItem` menu-item actions. Main thread only.
pub(super) fn ensure_regular_active_app(mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
}

/// Switch the macOS activation policy: `.accessory` hides the Dock icon
/// (menu-bar-only), `.regular` keeps it (Spotify default). Must run on the
/// main thread.
pub fn set_dock_icon_hidden(hidden: bool) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let policy = if hidden {
        NSApplicationActivationPolicy::Accessory
    } else {
        NSApplicationActivationPolicy::Regular
    };
    app.setActivationPolicy(policy);
}
