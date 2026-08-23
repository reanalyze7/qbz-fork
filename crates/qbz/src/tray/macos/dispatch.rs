//! `dispatch_tag`, `handle_status_click`, `pop_up_menu` — routing clicked
//! items / clicks to the shared `tray` module dispatch helpers.

use objc2_app_kit::{NSApplication, NSEventModifierFlags, NSEventType, NSMenu, NSStatusItem};
use objc2_foundation::{MainThreadMarker, NSInteger};

use super::{CTX, MENU, STATUS_ITEM};

// Menu item tags → actions.
pub(super) const TAG_PLAY_PAUSE: NSInteger = 1;
pub(super) const TAG_NEXT: NSInteger = 2;
pub(super) const TAG_PREVIOUS: NSInteger = 3;
pub(super) const TAG_SHOW_HIDE: NSInteger = 4;
pub(super) const TAG_QUIT: NSInteger = 5;

/// Route a clicked menu item's tag to the shared dispatch helpers.
pub(super) fn dispatch_tag(tag: NSInteger) {
    log::info!("[tray] menu item activated: tag={tag}");
    let Some((runtime, weak, handle)) = CTX.get() else {
        log::warn!("[tray] dispatch_tag: context not initialized");
        return;
    };
    match tag {
        TAG_PLAY_PAUSE => {
            super::super::dispatch_play_pause(runtime.clone(), weak.clone(), handle.clone())
        }
        TAG_NEXT => super::super::dispatch_next(runtime.clone(), weak.clone(), handle.clone()),
        TAG_PREVIOUS => super::super::dispatch_previous(runtime.clone(), weak.clone(), handle.clone()),
        TAG_SHOW_HIDE => super::super::toggle_window(weak),
        TAG_QUIT => super::super::quit(),
        other => log::debug!("[tray] unhandled menu tag {other}"),
    }
}

/// Status-bar button click router. Reads the current AppKit event: a
/// right-click or control-click pops the menu, a plain left-click toggles the
/// window. Main thread only (it's an AppKit action callback).
pub(super) fn handle_status_click() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let (is_right, is_ctrl) = match app.currentEvent() {
        Some(ev) => {
            let ty = unsafe { ev.r#type() };
            let mods = unsafe { ev.modifierFlags() };
            (
                ty == NSEventType::RightMouseUp,
                mods.contains(NSEventModifierFlags::NSEventModifierFlagControl),
            )
        }
        None => (false, false),
    };

    if is_right || is_ctrl {
        pop_up_menu(mtm);
    } else if let Some((_, weak, _)) = CTX.get() {
        super::super::toggle_window(weak);
    }
}

/// Pop the tray menu transiently. Non-deprecated replacement for
/// `popUpStatusItemMenu:`: flash the menu onto the status item, simulate a
/// click (which opens it modally), then detach it so a left-click doesn't open
/// it. Main thread only.
fn pop_up_menu(mtm: MainThreadMarker) {
    STATUS_ITEM.with(|s| {
        let Some(status_item) = s.borrow().as_ref().cloned() else {
            return;
        };
        MENU.with(|m| {
            if let Some(menu) = m.borrow().as_ref() {
                apply_menu(&status_item, menu, mtm);
            }
        });
    });
}

fn apply_menu(status_item: &NSStatusItem, menu: &NSMenu, mtm: MainThreadMarker) {
    unsafe { status_item.setMenu(Some(menu)) };
    if let Some(button) = unsafe { status_item.button(mtm) } {
        unsafe { button.performClick(None) };
    }
    unsafe { status_item.setMenu(None) };
}
