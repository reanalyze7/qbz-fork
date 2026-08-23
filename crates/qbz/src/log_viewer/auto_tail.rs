//! The auto-tail timer: stateful, UI-thread-only machinery for the
//! `toggle-auto-tail` callback.

use crate::AppWindow;

use super::refresh::rebuild;

/// Auto-tail refresh cadence.
pub(super) const AUTO_TAIL_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(1500);

thread_local! {
    /// The auto-tail timer. UI-thread only (slint::Timer requirement); started /
    /// stopped from the `toggle-auto-tail` callback (always on the UI thread).
    static AUTO_TAIL_TIMER: slint::Timer = slint::Timer::default();
}

/// Body of the `toggle-auto-tail` callback: start/stop the timer, mirroring
/// `on` onto the `auto-tail` property.
pub(super) fn toggle(weak: &slint::Weak<AppWindow>, on: bool) {
    use slint::ComponentHandle;
    if let Some(w) = weak.upgrade() {
        w.global::<crate::LogViewerState>().set_auto_tail(on);
    }
    AUTO_TAIL_TIMER.with(|timer| {
        if on {
            let weak = weak.clone();
            timer.start(slint::TimerMode::Repeated, AUTO_TAIL_INTERVAL, move || {
                rebuild(&weak);
            });
        } else {
            timer.stop();
        }
    });
}
