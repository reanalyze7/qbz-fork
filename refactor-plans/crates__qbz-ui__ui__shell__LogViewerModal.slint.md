# `crates/qbz-ui/ui/shell/LogViewerModal.slint` (516 lines)

In-app log viewer modal: header (title + count + advanced-toggle + close), advanced
controls (level filter/search/refresh/auto-tail), column header, scrolling per-level-
colored log list, uploaded-URL row, advanced actions (bundle/open/clear), footer
(copy-all/upload).

## Proposed split

- `LogViewerModal.slint` (~110 lines) — stays the public surface: `export component
  LogViewerModal`, backdrop + centered card shell, composes the sub-blocks below.
- `shell/IconTextButton.slint` (~50 lines) — extract the internal `IconTextButton`
  helper component (lines 29-75) — small, fully self-contained, reused by both the
  advanced-actions row and the footer.
- `shell/LogViewerHeader.slint` (~60 lines) — title + shown/total count + advanced
  `[+]/[−]` toggle + close X (lines ~120-173), as an `in-out property <bool>
  advanced-open` two-way bound from the parent.
- `shell/LogViewerControls.slint` (~130 lines) — the advanced controls row (level
  `QbzSelect` + search box + refresh + auto-tail toggle, lines ~176-298).
- `shell/LogViewerList.slint` (~70 lines) — the column header + scrolling `ListView` body
  + empty state (lines ~300-401).
- `shell/LogViewerFooter.slint` (~90 lines) — uploaded-URL row + advanced actions
  (bundle/open/clear) + always-visible footer (copy-all/upload) (lines ~403-512).

## Coupling to flag

- All sub-components bind directly to the `LogViewerState`/`LogViewerActions` /
  `UiFocusState` globals — no prop-threading needed except `advanced-open`, which is
  local state on the root component and must be passed to/from `LogViewerHeader` (toggle)
  and read by `LogViewerControls`/`LogViewerFooter` (visibility gates).
- The search box's `changed has-focus => { UiFocusState.text-input-focused = ... }`
  pattern (hotkey gating) must be preserved verbatim wherever the search input ends up.

## Verify after split

- Slint compile check.
- Visual smoke test: open modal, toggle advanced controls, level filter + search filter
  (Rust-side), refresh, auto-tail, copy-all/upload footer buttons, uploaded-URL copy,
  bundle/open-file/clear actions, empty state.
