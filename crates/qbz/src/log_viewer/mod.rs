//! Log viewer controller (developer log overlay).
//!
//! Wires the `LogViewerState` Slint global to the `qbz_log` in-memory ring. The
//! viewer is a thin read surface over `qbz_log::ring`: `refresh` snapshots the
//! ring, applies the level + search filters, caps to the last 1000 rows, and
//! pushes `[LogRow]`. `clear` empties the ring; `set-level` / `set-search`
//! re-filter; `auto-tail` re-runs `refresh` every 1.5s via a `slint::Timer`.
//! `copy-all` copies the currently-filtered rows; `copy-bundle` builds a
//! GitHub-ready diagnostics bundle; `upload` POSTs that bundle to paste.rs and
//! surfaces the returned URL; `open-log-file` opens the on-disk log.
//!
//! All log text is redacted at the ring's write choke point; clipboard/upload
//! paths redact again defensively.

use std::sync::Arc;

use slint::ComponentHandle;

use crate::adapter::SlintAdapter;
use crate::{AppWindow, LogViewerState};

mod auto_tail;
mod filter;
mod refresh;
mod share;

use refresh::{filtered_text, rebuild};
use share::{build_share_text, flash_copied};

type Runtime = Arc<qbz_app::shell::AppRuntime<SlintAdapter>>;

/// Wire every `LogViewerState` callback. Call once at shell setup. `runtime` is
/// used by the "Copy diagnostics bundle" / "Upload" paths to gather the COMPLETE
/// diagnostics report (system + live audio + graphics + playback + qconnect).
pub fn install(window: &AppWindow, runtime: Runtime, handle: tokio::runtime::Handle) {
    let state = window.global::<LogViewerState>();

    {
        let weak = window.as_weak();
        state.on_refresh(move || rebuild(&weak));
    }
    {
        let weak = window.as_weak();
        state.on_clear(move || {
            qbz_log::ring::clear();
            rebuild(&weak);
        });
    }
    {
        let weak = window.as_weak();
        // The new value is already stored in the in-out `filter-level`; rebuild
        // reads it back. Same for search.
        state.on_set_level(move |_level| rebuild(&weak));
    }
    {
        let weak = window.as_weak();
        state.on_set_search(move |_search| rebuild(&weak));
    }
    {
        let weak = window.as_weak();
        state.on_toggle_auto_tail(move |on| auto_tail::toggle(&weak, on));
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        state.on_copy_all(move || {
            let text = filtered_text(&weak).join("\n");
            crate::share::copy_to_clipboard(text);
            flash_copied(&weak, &handle);
        });
    }
    share::wire_bundle_callbacks(window, runtime, handle);
}
