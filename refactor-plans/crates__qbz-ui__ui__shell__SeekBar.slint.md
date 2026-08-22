# crates/qbz-ui/ui/shell/SeekBar.slint (154 lines)

## Summary
`SeekBar` — the player's elapsed/remaining time labels around a three-layer
progress track (buffered/cache line, playback progress line, hover thumb)
with a hover time-tooltip (bubble + downward caret) showing the seek target.

## Proposed split
Only ~24 lines over budget; the cleanest split is to extract the hover
tooltip (the most self-contained visual sub-piece, with its own internal
`at-secs` property) into its own component, since the track/thumb/TouchArea
logic is one cohesive interactive unit that shouldn't be split mid-gesture:

- `shell/SeekBar.slint` (~110 lines after extraction) — keeps the
  `export component SeekBar`: elapsed/remaining time labels, the `clamp01`/
  `fmt` helper functions, the `track` Rectangle with cache line + progress
  line + hover thumb + the `ta := TouchArea` (click-to-seek). Instantiates
  the new tooltip component instead of the inline `tip := Rectangle { ... }`.
- `shell/SeekBarTooltip.slint` (~55 lines) — a new `component
  SeekBarTooltip` covering the current `tip := Rectangle { ... }` block
  (lines 103-140): the bubble with the formatted time text and the caret
  `Path`. Takes `in property <bool> visible-hover`, `in property <length>
  mouse-x`, `in property <length> track-width`, `in property <int>
  duration-secs` as inputs (computed by the parent from `ta.mouse-x`/
  `ta.has-hover`/`track.width`/`root.duration-secs`) so it stays a small,
  presentation-only component with no `TouchArea` of its own.
  - The `at-secs` computation (`Math.round(root.clamp01(ta.mouse-x /
    track.width) * root.duration-secs)`) and the `fmt()` call move into this
    new component too, so it needs its own `fmt(secs: int) -> string`
    function (duplicated) OR — cleaner — `SeekBar.slint` computes `at-secs`
    itself (it already has `clamp01` and `fmt`) and passes a plain `in
    property <string> time-label: root.fmt(...)` down to
    `SeekBarTooltip`, avoiding logic duplication entirely. Prefer this
    second approach: `SeekBarTooltip` becomes purely presentational (an
    `in property <string> time-label` + `in property <length> mouse-x` +
    `in property <length> track-width`), and `SeekBar.slint` keeps ALL the
    math (`clamp01`, `fmt`, `at-secs`).

## Re-export surface
`shell/SeekBar.slint`'s `export component SeekBar` stays the only symbol
other `.slint` files import (`import { SeekBar } from "./SeekBar.slint";` or
similar, from the player bar / now-playing components per the file's own
doc comment about "New/Classic bars" and the "Small bar" caller passing
`show-times:false`). `SeekBarTooltip` is internal to `shell/` and does not
need exporting beyond this directory unless another file already reaches
into `SeekBar.slint` for it directly (unlikely — grep to confirm before
finalizing).

## Coupling / watch out
- The tooltip's positioning (`x: Math.min(Math.max(ta.mouse-x - self.width /
  2, 0px), track.width - self.width);`) depends on BOTH `ta.mouse-x` (the
  TouchArea inside `track`, which stays in `SeekBar.slint`) and `self.width`
  (the tooltip's own `bubble.preferred-width`) — after extraction this
  becomes `x: Math.min(Math.max(root.mouse-x - self.width / 2, 0px),
  root.track-width - self.width);` inside `SeekBarTooltip`, reading the two
  new `in property`s instead of reaching into a sibling directly. Slint
  components CANNOT reach into a sibling's named element (`ta.mouse-x`)
  across a component boundary — this is the one real mechanical change the
  split requires, not just a file move.
- Tooltip visibility (`opacity: ta.has-hover ? 1.0 : 0.0;`) similarly needs
  to become `opacity: root.visible-hover ? 1.0 : 0.0;` bound from
  `SeekBar.slint` as `visible-hover: ta.has-hover;`.
- The hover thumb Rectangle (kept in `SeekBar.slint`) ALSO reads
  `ta.has-hover` for its own opacity — no coupling issue, it's not moving.
- `Theme` and `Typography` imports are used in both the retained `SeekBar`
  code and the extracted tooltip — `SeekBarTooltip.slint` needs its own
  `import { Theme } from "../foundation/semantic-colors.slint"; import {
  Typography } from "../foundation/typography.slint";` lines.
- `show-times` (the New/Classic vs. Small-bar toggle) only affects the
  leading/trailing time `VerticalLayout`s in `SeekBar.slint` — it does not
  touch the tooltip at all, so no new prop threading needed there.

## Verify after split
- Build the Slint UI (`cargo build -p qbz-ui` or however `.slint` files are
  compiled in this repo — check for a codegen `build.rs` step) and confirm
  no import/property-binding errors, in particular around the
  `mouse-x`/`track-width`/`time-label` hand-off described above.
- Smoke-test in the running app: hover the seek bar in BOTH the New/Classic
  now-playing bar (show-times:true) and the Small bar (show-times:false),
  confirm the time-bubble tooltip tracks the cursor correctly, the caret
  stays centered under the bubble, and click-to-seek still respects the
  `seekable-max` not-allowed-cursor gate.
