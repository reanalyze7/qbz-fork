//! `create` — the main menu/status-item builder. Kept whole since it's one
//! linear build sequence with real internal ordering dependencies.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::sel;
use objc2_app_kit::{
    NSEventMask, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{MainThreadMarker, NSInteger, NSString};

use super::activation::ensure_regular_active_app;
use super::dispatch::{TAG_NEXT, TAG_PLAY_PAUSE, TAG_PREVIOUS, TAG_QUIT, TAG_SHOW_HIDE};
use super::icon::apply_icon;
use super::menu_target::QbzTrayMenuTarget;
use super::{AppWindow, Runtime, CTX, MENU, MENU_TARGET, STATUS_ITEM};

/// Build the menu-bar item + menu. MUST run on the main thread (call via
/// `slint::invoke_from_event_loop`).
pub fn create(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    theme_override: &str,
) {
    let Some(mtm) = MainThreadMarker::new() else {
        log::error!("[tray] create called off the main thread");
        return;
    };

    // Store the dispatch context (only the first init wins).
    let _ = CTX.set((runtime, weak, handle));

    // The action target. Held alive in a thread_local so the menu's weak
    // target reference stays valid.
    let target = QbzTrayMenuTarget::new(mtm);
    let target_obj: &AnyObject = &target;
    let action = sel!(onMenuItem:);

    // Build the menu: 3 transport items, separator, show/hide, separator, quit.
    let menu = NSMenu::new(mtm);
    let empty_key = NSString::from_str("");
    let make_item = |title: &str, tag: NSInteger| -> Retained<NSMenuItem> {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &NSString::from_str(title),
                Some(action),
                &empty_key,
            )
        };
        unsafe {
            item.setTarget(Some(target_obj));
            item.setTag(tag);
            item.setEnabled(true);
        }
        item
    };

    menu.addItem(&make_item(&qbz_i18n::t("Play/Pause"), TAG_PLAY_PAUSE));
    menu.addItem(&make_item(&qbz_i18n::t("Next Track"), TAG_NEXT));
    menu.addItem(&make_item(&qbz_i18n::t("Previous Track"), TAG_PREVIOUS));
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    menu.addItem(&make_item(&qbz_i18n::t("Show/Hide Window"), TAG_SHOW_HIDE));
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    menu.addItem(&make_item(&qbz_i18n::t("Quit Qoqobuz"), TAG_QUIT));

    // Build the status item and wire the icon.
    let status_bar = unsafe { NSStatusBar::systemStatusBar() };
    let status_item = unsafe { status_bar.statusItemWithLength(NSVariableStatusItemLength) };
    apply_icon(&status_item, theme_override, mtm);

    // Do NOT attach the menu permanently (that makes any click open it). Give
    // the status button its own action that fires on left AND right mouse-up;
    // `handle_status_click` decides toggle-vs-menu. The menu is kept in the
    // MENU thread_local and only flashed on for a right-click pop-up.
    if let Some(button) = unsafe { status_item.button(mtm) } {
        unsafe {
            button.setTarget(Some(target_obj));
            button.setAction(Some(sel!(onStatusButton:)));
            button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
        }
    }

    STATUS_ITEM.with(|s| *s.borrow_mut() = Some(status_item));
    MENU_TARGET.with(|t| *t.borrow_mut() = Some(target));
    MENU.with(|m| *m.borrow_mut() = Some(menu));

    // A bare `cargo run` binary is NOT a bundled .app. Without an explicit
    // Regular activation policy + activation, macOS treats the app as a
    // background process and `[NSApp sendAction:]` may not route the menu item
    // target-action. Force Regular + active.
    ensure_regular_active_app(mtm);

    log::info!("[tray] menu-bar item initialized (theme={theme_override})");
}
