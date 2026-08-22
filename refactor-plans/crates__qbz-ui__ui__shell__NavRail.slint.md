# crates/qbz-ui/ui/shell/NavRail.slint (168 lines)

## Summary
A bottom navigation rail (icon+label tiles, 7 fixed absolute-positioned
tiles: Discover/Library/Local Library/MyQBZ/Now Playing/Visualizer/Settings)
intended as a small-panel/touch alternative to the desktop `Sidebar`,
driving the same `NavState` routes.

## ORPHANED — likely dead code
`grep -rn "import.*NavRail" crates/qbz-ui/ui/` returns **zero matches**
anywhere in the `.slint` tree. `Sidebar.slint` appears to be the actual
mounted navigation component (referenced from AppShell/KioskShell). This file
exports `NavRail` and defines a local `NavTile` component, but nothing
imports either. Before spending effort on the split below, confirm with a
repo-wide grep across non-`.slint` files too (in case it's referenced from
generated Rust bindings or a `.slint` file outside `crates/qbz-ui/ui/`) and,
if genuinely unused, **delete the file** instead of splitting it — that's
strictly simpler than maintaining a multi-file split of dead code. This plan
is written per the task instructions in case it turns out to be used
somewhere not covered by the grep above (e.g. a kiosk-mode build variant not
present in this checkout).

## Proposed split (if kept)
- `shell/nav_rail/nav_tile.slint` (~50 lines) — the `NavTile` component
  (lines 13-55): icon+label tile with active/hover styling.
- `shell/NavRail.slint` (~120 lines, stays at this path) — the exported
  `NavRail` component (lines 57-168): the 7 hand-positioned `NavTile`
  instances + the `tw`/`th` sizing properties + the 7 click callbacks.
  Imports `NavTile` from the new sibling file. Note the 7 tile blocks are
  extremely repetitive (only differing in icon/label/active-condition/
  callback) — a further improvement (not a line-count-driven split, but
  worth flagging) would be a `for` loop over a small struct-array of tile
  specs, which would shrink this file well under 130 lines on its own and
  remove the manual `x: 8px + N * (tw + 6px)` arithmetic repeated 7 times.

## Re-export surface
`NavRail.slint` (same filename/path) stays the only import path, if any
caller exists. `NavTile` becomes a private sub-component imported only by
`NavRail.slint` itself.

## Coupling / watch out
- `NavTile` has zero coupling to global state (`icon`/`label`/`active` are
  all `in` properties, `clicked` is a plain callback) — trivial to extract.
- `NavRail` itself reads `NavState`, `ContentView`, `ImmersiveState`,
  `MyQbzBrandingState` from `../foundation`/`../state.slint` — these imports
  are unaffected by extracting `NavTile` since `NavRail` still needs them for
  its `active:` bindings.
- `ImmersiveState` is imported (line 11) but never referenced in the body —
  worth flagging as dead-import cleanup regardless of whether the file
  itself is deleted or split.

## Verify after split (or after deletion)
- If deleting: re-run `grep -rn "NavRail" crates/` (not just `crates/qbz-ui/ui/`)
  and `grep -rn "NavRail" crates/**/*.rs` to be certain no Rust code
  references a generated `NavRail` binding before removing the file; then
  confirm `cargo build` / the Slint compile step still succeeds with the
  file gone.
- If splitting instead: Slint compile check succeeds with no unresolved
  imports; since the component appears unmounted, there is no user-visible
  smoke test available — rely on the compile check only.
