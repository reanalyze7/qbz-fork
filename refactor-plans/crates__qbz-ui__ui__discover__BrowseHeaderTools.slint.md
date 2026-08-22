# crates/qbz-ui/ui/discover/BrowseHeaderTools.slint (173 lines)

## Summary
Three independent, reusable header-tool components shared by the browse
pages (DiscoverBrowseView, PlaylistBrowseView): `BrowseSearch` (search box
with clear-X), `GenreButton` (genre-filter trigger reading the shared
`GenreFilterState`), and `ViewModeToggle` (grid/list two-button toggle).

## Proposed split
Each component is already fully self-contained (no shared local helpers
between them) — split one-component-per-file, which is the cleanest and
lowest-risk cut available.

- `discover/BrowseHeaderTools.slint` (~15 lines) — becomes a pure re-export
  barrel: imports `Theme`, and re-exports the three components via
  `export { BrowseSearch } from "BrowseSearchField.slint";` /
  `export { GenreButton } from "GenreButton.slint";` /
  `export { ViewModeToggle } from "ViewModeToggle.slint";` (or Slint's
  equivalent multi-export syntax) so every existing
  `import { BrowseSearch, GenreButton, ViewModeToggle } from
  "discover/BrowseHeaderTools.slint";` call site keeps working unchanged.
- `discover/BrowseSearchField.slint` (~80 lines) — `BrowseSearch` component
  in full (lines 19-97): the leading magnifier icon, `TextInput`, placeholder
  text, and trailing clear-X.
- `discover/GenreButton.slint` (~45 lines) — `GenreButton` component
  (lines 102-145): accent-fill active state + genre-count text, reading
  `GenreFilterState.selected-count`.
- `discover/ViewModeToggle.slint` (~26 lines) — `ViewModeToggle` component
  (lines 149-173): the two-`ToggleButton` grid/list pair.

## Re-export surface
`discover/BrowseHeaderTools.slint` stays the single import path every
caller uses today — turning it into a re-export barrel means
`DiscoverBrowseView.slint` / `PlaylistBrowseView.slint` need zero changes to
their `import` statements.

## Coupling / watch out
- All three components read `ShellState.app-background-active` and
  `AppearanceState.app-background-surface-alpha` for the translucent
  background variant — each new file needs both imports from `../state.slint`
  (currently imported once at the top of the shared file).
- `BrowseSearch` also sets `UiFocusState.text-input-focused` on
  focus-change — a global side effect other components may depend on
  (e.g. keyboard-shortcut suppression while typing); keep that line exactly
  as-is when moving the component.
- `GenreButton` has no `clicked` internal logic — it only emits `clicked()`
  and lets the caller decide what to open; verify no caller relies on
  `BrowseHeaderTools.slint` exporting anything besides these 3 components
  (e.g. a shared color constant) before turning it into a pure barrel.
- If Slint's import system does not support the `export { X } from "file"`
  re-export barrel pattern (verify against the Slint version in use), fall
  back to keeping `BrowseHeaderTools.slint` as a real file that itself does
  `import { BrowseSearch } from "BrowseSearchField.slint"; export {
  BrowseSearch };` per component — check existing plans in this repo (e.g.
  other split `.slint` files) for the pattern this project has already
  settled on.

## Verify after split
- Slint compile check on all four files.
- `cargo build -p qbz-ui`.
- Smoke-test both browse pages (Discover Browse, Playlist Browse): search
  box typing/clearing, genre-filter button opening the genre picker, and the
  grid/list toggle switching layouts.
