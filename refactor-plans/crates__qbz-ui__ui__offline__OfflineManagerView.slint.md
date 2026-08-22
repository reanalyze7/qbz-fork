# crates/qbz-ui/ui/offline/OfflineManagerView.slint (576 lines)

## Summary
Standalone "manage downloads" view: header, a stats bar (track count / size
+ **size-limit GB slider/input** / open-folder / clear-all), a toolbar
(sort dropdown + failed-only filter + bulk actions), and a two-pane body
(A-Z artist rail + scrolling album/track row list with per-row remove/
re-download and a cache-status glyph).

## Proposed split
Split into sibling `.slint` files under `ui/offline/` (flat, matching
crate convention):

- `OfflineManagerView.slint` (~110 lines) — KEEP as the main file: imports,
  `export component OfflineManagerView`, and the top-level
  `VerticalLayout` composition wiring header + stats bar + toolbar + body
  together. This stays the public import surface.
- `offline/IconBtn.slint` (~25 lines) — lines 23-45: the small ghost icon
  button.
- `offline/StatusIcon.slint` (~30 lines) — lines 48-75: the cache-status
  glyph (spinning while downloading).
- `offline/ArtistRailRow.slint` (~40 lines) — lines 78-113: one A-Z rail
  row.
- `offline/BulkIconBtn.slint` (~45 lines) — lines 119-158: the bulk-action
  icon button with tooltip.
- `offline/OfflineStatsBar.slint` (~130 lines) — lines 184-288: the whole
  stats-bar `Rectangle` INCLUDING the size-limit GB block (track/size text,
  usage progress bar, the limit `TextInput` + GB label + confirm `IconBtn`,
  open-folder button, clear-all button). **Keep this entire block intact as
  one component/file per the task's explicit instruction — do not further
  split the limit-input sub-block out of it.** Takes no props beyond
  reading `OfflineManagerState`/calling `OfflineManagerActions` directly
  (matches the file's existing pattern of components touching globals
  directly).
- `offline/OfflineToolbar.slint` (~90 lines) — lines 291-369: sort
  dropdown, "Failed only" toggle, spacer, bulk-action buttons (uses
  `BulkIconBtn`), select-all/clear toggle.
- `offline/ArtistRail.slint` (~35 lines) — lines 377-403: the left A-Z
  rail `ScrollView` + "All artists" row + `for a in ... ArtistRailRow`.
- `offline/OfflineRowsList.slint` (~170 lines) — lines 406-573: the right
  pane's loading/empty states plus the scroll-restore `ScrollView` and the
  `for row in OfflineManagerState.rows` album/track row rendering
  (selection checkbox, cover/track-number, title/subtitle, meta,
  re-download, remove, `StatusIcon`).

## Re-export surface
`OfflineManagerView.slint`'s `export component OfflineManagerView` stays
the only thing other `.slint` files import (`import { OfflineManagerView }
from "offline/OfflineManagerView.slint";`). All new sub-files are imported
by `OfflineManagerView.slint` and wired together in its body.

## Coupling / watch out
- **The GB size-limit slider/input block (inside `OfflineStatsBar.slint`)
  was recently discussed — do not split it further or change its behavior;
  keep the `TextInput` + `UiFocusState.text-input-focused` wiring, the
  `.to-float()` parse-on-accept/confirm-click pattern, and the "GB" label
  exactly as-is.**
- `IconBtn`/`BulkIconBtn` are two DIFFERENT components (a tinted toolbar
  glyph vs a bordered-danger bulk-action button) — don't merge them when
  extracting; they're named distinctly in the source for a reason (per the
  file's own comment at line 115-118).
- `ArtistRailRow`'s `active` state and `ArtistRailRow`/rail-row list both
  depend on `OfflineManagerState.selected-artist`/`.artists` — no local
  component state beyond what's passed in, so extraction is low-risk here.
- The rows list's scroll-restore block (`sr-armed`/`sr-restore`,
  `NavState.restore-scope == "offline-manager"`) is `ScrollView`-local and
  must move together with `OfflineRowsList.slint` — don't strand it in the
  main file.
- `StatusIcon`'s spin animation reads `ShellState.reduce-motion` /
  `ShellState.coarse-tick-ms` / `animation-tick()` — a global-state
  dependency, unaffected by which file it lives in.

## Verify after split
- Slint compile check (`cargo build` triggers slint-build, or
  `slint-viewer`/`slint-lsp` if available in CI).
- Manually smoke-test: open Offline Cache Manager, verify the stats bar
  (especially the GB limit input — type a value, click confirm, verify it
  persists; check the usage bar color flips to red-ish over 100%), the
  artist rail filter, sort dropdown, "Failed only" toggle, bulk select/
  bulk re-download/bulk remove, and per-row re-download/remove/status
  glyph (including the spinning "downloading" state).
