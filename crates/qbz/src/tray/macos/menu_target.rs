//! The `QbzTrayMenuTarget` AppKit action-target class.

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_app_kit::NSMenuItem;
use objc2_foundation::MainThreadMarker;

use super::dispatch::{dispatch_tag, handle_status_click};

declare_class!(
    pub(super) struct QbzTrayMenuTarget;

    unsafe impl ClassType for QbzTrayMenuTarget {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "QbzTrayMenuTarget";
    }

    impl DeclaredClass for QbzTrayMenuTarget {
        type Ivars = ();
    }

    unsafe impl QbzTrayMenuTarget {
        #[method(onMenuItem:)]
        fn on_menu_item(&self, sender: Option<&NSMenuItem>) {
            let tag = sender.map(|s| unsafe { s.tag() }).unwrap_or(0);
            dispatch_tag(tag);
        }

        // Fires on left AND right mouse-up of the status-bar button (see
        // `sendActionOn` in `create`). We inspect the current event to route:
        // right-click / control-click → pop the menu; plain left-click → toggle.
        #[method(onStatusButton:)]
        fn on_status_button(&self, _sender: Option<&AnyObject>) {
            handle_status_click();
        }
    }
);

impl QbzTrayMenuTarget {
    pub(super) fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(());
        unsafe { msg_send_id![super(this), init] }
    }
}
