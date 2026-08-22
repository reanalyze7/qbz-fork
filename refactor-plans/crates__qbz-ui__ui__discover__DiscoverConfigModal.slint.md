# crates/qbz-ui/ui/discover/DiscoverConfigModal.slint (370 lines)

## Summary
The Discover "Customize" modal: a section reorder/show-hide list (perf-warning
banner, enabled/total count, per-row checkbox+reorder buttons, reset-to-
defaults footer) for the active Discover tab, PLUS a separate
Recommendations-tab explainer + cache-window/refresh-now control panel shown
instead of the section list when `active-tab == "recommendations"`.

## Proposed split
By the two mutually-exclusive modes this modal already branches on (section-
config vs recommendations-config), plus extracting the two small row helper
components:

- `discover/DiscoverConfigModal.slint` (~110 lines) — stays the re-export
  surface: module doc, imports, `export component DiscoverConfigModal` shell
  (scrim + panel sizing, title row + close button), delegating the body to the
  two mode components below based on `DiscoverState.active-tab`.
- `discover/DiscoverConfigHelpers.slint` (~75 lines) — `ReorderButton` (lines
  26–53) and `ConfigRowView` (lines 56–120), the two small standalone row
  components.
- `discover/DiscoverSectionConfig.slint` (~110 lines) — the section-config
  body (lines 183–246 + 325–366: perf-warning banner, count line, scrollable
  section list using `ConfigRowView`, reset-to-defaults footer) — everything
  gated on `active-tab != "recommendations"`.
- `discover/DiscoverRecoConfig.slint` (~65 lines) — the Recommendations-tab
  body (lines 250–323: explainer text, cache-window `QbzSelect` + refresh-now
  button) — everything gated on `active-tab == "recommendations"`.

## Re-export surface
`discover/DiscoverConfigModal.slint` remains the only file imported by callers
(likely `DiscoverView.slint` or similar, opened via the "Customize" button) —
`export component DiscoverConfigModal` keeps its current (implicit) signature.

## Coupling / watch out
- The panel's `height:` binding (line 138: `Math.min(panel.preferred-height,
  root.height * 0.78)`) references `panel` by id — `panel` is the outer
  `VerticalLayout` that would still live in the main file (it wraps both the
  header AND the extracted body components), so this stays intact as long as
  `panel` itself isn't moved.
- `DiscoverSectionConfig`'s list's height cap (line 228:
  `root.height * 0.78 - 260px`) is a magic constant assuming a specific header
  height — when splitting, keep this computed relative to `root` (the modal's
  own root, still reachable since `root` refers to the innermost component
  root in Slint, so this constant would need re-deriving inside the new
  child component using ITS OWN root, which is a different `Rectangle` than
  today — verify this doesn't silently change behavior; may need to pass the
  available height down as an `in property` instead of recomputing).
- `ConfigRowView`'s `entry` property is named `entry` specifically to avoid
  colliding with Slint's reserved `row` grid attached-property — preserve this
  naming when moved to `DiscoverConfigHelpers.slint`.
- Both mode bodies read `DiscoverState`/`DiscoverActions`/`ExternalRecoState`/
  `ExternalRecoActions` directly — no prop drilling needed, just re-import the
  same globals in each new file.

## Verify after split
- `cargo build -p qbz-ui`.
- Manual smoke test: open Discover, click Customize on a normal tab (verify
  reorder/toggle/reset all work and persist), switch to the Recommendations
  tab's Customize (verify cache-window select + refresh-now still work), and
  confirm the panel's height/scroll behavior is unchanged in both modes.
