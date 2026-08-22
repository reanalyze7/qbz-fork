# crates/qbz/src/log_viewer.rs (277 lines)

## Summary
Log viewer controller (developer log overlay): wires the `LogViewerState`
Slint global to the `qbz_log` in-memory ring — refresh/filter/clear,
auto-tail timer, copy-all, copy-diagnostics-bundle, upload-to-paste.rs, and
open-log-file — with redaction applied at multiple layers.

## Proposed split
147 lines over budget. The file cleanly separates into "wire the callbacks"
(imperative, stateful, UI-thread) vs. "compute rows/text from the ring"
(pure-ish, testable) vs. "build the shareable bundle" (async, calls into
diagnostics).

- `log_viewer/mod.rs` (~55 lines) — `install()` only: builds the `Runtime`
  type alias, registers all `LogViewerState` callbacks by delegating each
  body to a named function in the sibling files below. Re-exports `install`.
- `log_viewer/filter.rs` (~40 lines) — `line_matches`, `MAX_VIEW_ROWS`
  constant: the pure level/search predicate, easy to unit test in isolation.
- `log_viewer/refresh.rs` (~75 lines) — `rebuild` and `filtered_text`: the
  ring-snapshot + filter + cap-to-1000 + push-to-`LogViewerState` logic, and
  the plain-text join used by `copy-all`.
- `log_viewer/auto_tail.rs` (~35 lines) — the `AUTO_TAIL_TIMER` thread_local,
  `AUTO_TAIL_INTERVAL` constant, and the `on_toggle_auto_tail` callback body
  (timer start/stop), since it's the one piece of genuinely stateful
  UI-thread-only machinery.
- `log_viewer/share.rs` (~85 lines) — `flash_copied`, `build_share_text`, and
  the `on_copy_bundle`/`on_upload`/`on_copy_url` callback bodies (clipboard +
  paste.rs POST + the markdown-report-plus-last-200-lines assembly).

## Re-export surface
`log_viewer/mod.rs` re-exports `pub fn install(...)` unchanged — the only
thing callers (`crate::log_viewer::install` from shell setup) use.

## Coupling / watch out
- `AUTO_TAIL_TIMER` is a `thread_local!` — it MUST stay reachable only from
  the UI thread; moving it to `auto_tail.rs` doesn't change that constraint
  but make sure no other file tries to touch it.
- `build_share_text` in `share.rs` calls `crate::diagnostics::build_full_report`
  — this is the direct coupling point with `diagnostics.rs` (also being
  split by another agent in this run); if that file's `build_full_report`
  moves to a new path, `share.rs` needs the updated `use`.
- Redaction happens in THREE places (ring write choke point, `filtered_text`,
  `build_share_text`) — when splitting, keep all three `qbz_log::redact`
  call sites; don't accidentally dedupe them away as "redundant".
- `MAX_VIEW_ROWS` (1000) is used both by `refresh.rs`'s `rebuild` and
  `filtered_text` — keep it as one shared `pub(super) const` in `mod.rs` or
  `filter.rs`, not duplicated.

## Verify after split
- `cargo check -p qbz` (or wherever this Slint frontend crate is)
- Manual/smoke test: open the developer log overlay, toggle level/search
  filters, toggle auto-tail, hit Copy All / Copy diagnostics bundle /
  Upload, confirm clipboard and uploaded-URL flows still work.
- Grep for `log_viewer::install` call site in shell setup to confirm the
  signature (`&AppWindow, Runtime, tokio::runtime::Handle`) is unchanged.
