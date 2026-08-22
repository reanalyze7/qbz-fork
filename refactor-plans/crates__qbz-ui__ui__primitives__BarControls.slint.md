# crates/qbz-ui/ui/primitives/BarControls.slint (147 lines)

## Summary
Two shared now-playing-bar control primitives extracted from `PlayerBar`:
`IconButton` (square icon button with hover tooltip) and `PlayButton`
(the primary play/pause control, circular "New" layout or plain "Classic"
glyph), both used by `TransportControls` and the bar's right-side cluster.

## Proposed split
Only 17 lines over budget — the cleanest split is simply one component per
file, which also matches the file's own framing ("two... primitives... so
[callers] can both use them without duplicating definitions"):

- `primitives/IconButton.slint` (~72 lines) — the `IconButton` component,
  verbatim, with its own `import { Theme } from "../foundation/semantic-colors.slint";
  import { Radius } from "../foundation/radius.slint"; import { QbzIcon }
  from "QbzIcon.slint"; import { TooltipState } from "../state.slint";`.
- `primitives/PlayButton.slint` (~85 lines) — the `PlayButton` component,
  verbatim, with its own imports (same as above plus `import {
  LoadingSpinner } from "LoadingSpinner.slint";`).
- `primitives/BarControls.slint` (~10 lines) — becomes a thin re-export
  shim: `import { IconButton } from "IconButton.slint"; import {
  PlayButton } from "PlayButton.slint"; export { IconButton, PlayButton };`
  so existing importers are completely unaffected (Slint supports
  re-exporting an imported component via `export { Name };`).

## Re-export surface
`primitives/BarControls.slint` stays the public import path — every
existing `import { IconButton, PlayButton } from
"../primitives/BarControls.slint";` (in `TransportControls` and the bar's
right-cluster component) keeps working unchanged via the re-export shim
above. New code added later is free to import directly from
`IconButton.slint`/`PlayButton.slint` instead, but nothing is required to
change.

## Coupling / watch out
- Both components independently duplicate an identical tooltip-on-hover
  `TouchArea { changed has-hover => { ... TooltipState... } }` block — this
  is pre-existing duplication, not something the split should "fix" (the
  task is only to split files, not refactor behavior), but worth flagging:
  if a THIRD file/agent later notices this and wants to extract a shared
  `Tooltipped` mixin, that would touch both new files — not this agent's
  job today.
- Both components' `TouchArea` also duplicates the exact same
  `clicked => { if (root.enabled) { root.clicked(); } }` pattern — same
  note as above, pre-existing, out of scope for a pure file-split.
- `PlayButton`'s `circle` boolean drives THREE different properties
  (`width`/`height`, `border-radius`, `opacity`, `background`, `border-width`)
  simultaneously — this is all self-contained within `PlayButton.slint`,
  no cross-file coupling risk.
- Neither component references the other — this is true independent-file
  territory, the safest kind of split in this whole batch.

## Verify after split
- Run the project's Slint compile check (`cargo build -p qbz-ui` / `cargo
  build -p qbz`, whichever triggers `.slint` codegen) to confirm the
  re-export shim resolves and no importer's path needs updating.
- Manually verify the now-playing bar in the running app: transport
  buttons (prev/play-pause/next/shuffle/repeat) still render with correct
  hover tooltips, active/tint states, and the play/pause button still
  toggles between its icon and the loading spinner during a resolving
  play action, in both New (circular) and Classic (plain-glyph) layouts.
