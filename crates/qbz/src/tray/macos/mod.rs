//! macOS menu-bar tray (`NSStatusItem`), hand-rolled on objc2 0.5.
//!
//! Why not `tray-icon`/`muda`: Slint's `i-slint-backend-winit` bundles its own
//! `muda` and registers the `MudaMenuItem` objc class. A second `muda` (from
//! tray-icon) registering the same class either silently fails to dispatch the
//! menu item target-action (objc2 0.6) or panics at startup ("could not create
//! new class MudaMenuItem", objc2 0.5). So we build the `NSStatusItem` + its
//! `NSMenu` directly on objc2 0.5 / objc2-app-kit 0.2 — the SAME objc2 era
//! winit 0.30 uses — so the menu lives in winit's `NSApplication` runtime and
//! `[NSApp sendAction:to:from:]` routes our items' target-action.
//!
//! Everything here is main-thread only: the `NSStatusItem`, the menu, and the
//! `QbzTrayMenuTarget` instance are `!Send` (`thread_local!`). `create` is
//! invoked via `slint::invoke_from_event_loop` (main thread). The action
//! callback reads the clicked item's `tag` and routes to the shared dispatch
//! helpers in the parent `tray` module (those marshal back onto the Slint loop
//! / tokio runtime themselves; the captured `slint::Weak`, `Runtime`, and
//! `tokio::runtime::Handle` are all `Send + Sync`, kept in a process-global
//! `OnceLock`).
//!
//! Click behavior matches the Tauri tray (`show_menu_on_left_click(false)`):
//! the menu is NOT permanently attached to the status item. Instead the status
//! button gets its own target-action firing on both left and right mouse-up.
//! LEFT-click toggles the window; RIGHT-click (or control-click) pops the menu
//! up transiently (set menu → `performClick` → clear menu, the non-deprecated
//! replacement for `popUpStatusItemMenu:`).

use std::cell::RefCell;
use std::sync::OnceLock;

use objc2::rc::Retained;
use objc2_app_kit::NSMenu;
use objc2_app_kit::NSStatusItem;

use super::Runtime;
use crate::AppWindow;

mod activation;
mod create;
mod dispatch;
mod icon;
mod menu_target;

use menu_target::QbzTrayMenuTarget;

/// Process-global dispatch context. Set once by `create`. The captured types
/// (`Runtime` = `Arc<..>`, `slint::Weak`, `tokio::runtime::Handle`) are all
/// `Send + Sync`, so reading this from the AppKit action callback (main thread)
/// is sound.
static CTX: OnceLock<(Runtime, slint::Weak<AppWindow>, tokio::runtime::Handle)> = OnceLock::new();

thread_local! {
    // Kept alive for the tray's lifetime; dropping the status item removes it
    // from the menu bar. Both are `!Send`, main-thread only.
    static STATUS_ITEM: RefCell<Option<Retained<NSStatusItem>>> = const { RefCell::new(None) };
    static MENU_TARGET: RefCell<Option<Retained<QbzTrayMenuTarget>>> = const { RefCell::new(None) };
    // The menu is NOT permanently attached to the status item (that would make
    // a left-click pop it). It lives here and is only flashed onto the status
    // item for the duration of a right/control-click pop-up.
    static MENU: RefCell<Option<Retained<NSMenu>>> = const { RefCell::new(None) };
}

pub use create::create;
pub use icon::set_icon_theme;
pub use activation::set_dock_icon_hidden;
