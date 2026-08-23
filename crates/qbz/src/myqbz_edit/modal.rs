//! Shared "on success/failure" bookkeeping every action calls.

use slint::ComponentHandle;

use crate::artwork::ImageCache;
use crate::{AppWindow, MyQbzEditState};

use super::reload::reload;

/// On a successful mutation: close the modal (if any), reload, and (when given)
/// toast a success message. On failure: clear busy + toast `err_msg`.
pub(super) fn finish(
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &ImageCache,
    id: String,
    result: Result<(), String>,
    success_toast: Option<String>,
    err_msg: String,
) {
    match result {
        Ok(()) => {
            if let Some(msg) = success_toast {
                crate::toast::success_weak(weak, msg);
            }
            close_modal(weak);
            reload(weak, handle, image_cache, id);
        }
        Err(e) => {
            log::warn!("[qbz-slint] myqbz_edit mutation failed: {e}");
            set_busy(weak, false);
            crate::toast::error_weak(weak, err_msg);
        }
    }
}

/// Close the edit modal (clears mode + busy). UI thread hop.
pub(super) fn close_modal(weak: &slint::Weak<AppWindow>) {
    let _ = weak.upgrade_in_event_loop(|w| {
        let es = w.global::<MyQbzEditState>();
        es.set_open(false);
        es.set_mode("".into());
        es.set_busy(false);
    });
}

/// Set the modal busy flag (disables submit) from any thread.
pub(super) fn set_busy(weak: &slint::Weak<AppWindow>, busy: bool) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        w.global::<MyQbzEditState>().set_busy(busy);
    });
}
