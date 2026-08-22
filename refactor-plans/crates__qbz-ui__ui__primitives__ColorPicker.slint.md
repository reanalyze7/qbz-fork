# crates/qbz-ui/ui/primitives/ColorPicker.slint (249 lines)

## Summary
An inline (non-popup) HSV color-picker primitive: owns its own hue/sat/val
state seeded from an incoming `color`, renders a saturation/value square + a
hue strip + a preview swatch + an editable HEX field, and emits both live
`changed(color)` drag updates and a `hex-committed(string)` event for the
parent to resolve.

## Proposed split
This is a single exported component with pure-function helpers and one
render tree — Slint doesn't support splitting one component's body across
files, so the practical split is: extract the two draggable sub-areas (SV
square, hue strip) into their own leaf components, leaving `ColorPicker`
itself as the thin owner of the HSV state + swatch/hex row.

- `primitives/color_picker/sv_square.slint` (~75 lines) — a new
  `SvSquare` component: the saturation/value square (lines 112-168), taking
  `hue: float`, `sat: float`, `val: float` as `in` properties and emitting
  `changed(float, float)` (new sat, new val) + `drag-started()`/`drag-ended()`
  callbacks so the parent can still gate `dragging` for the reseed guard.
- `primitives/color_picker/hue_strip.slint` (~45 lines) — a new `HueStrip`
  component: the hue gradient strip (lines 171-208), taking `hue: float` as
  `in` property, emitting `changed(float)` (new hue) +
  `drag-started()`/`drag-ended()`.
- `primitives/ColorPicker.slint` (~130 lines, stays at this path) — the
  exported `ColorPicker` component: owned HSV state (`hue`/`sat`/`val`/
  `dragging`), the RGB->HSV decompose function + `changed value` reseed
  guard, the int->hex helpers + `current-hex`, and the layout wiring
  `SvSquare`/`HueStrip` + the preview swatch + `LineEdit` hex field. Imports
  `SvSquare` and `HueStrip` from the two new sibling files.

## Re-export surface
`ColorPicker.slint` (same filename/path) stays the only import other
`.slint` files use (`import { ColorPicker } from "../primitives/ColorPicker.slint";`
from the custom-theme-editor view). `SvSquare`/`HueStrip` are new,
picker-internal components not imported anywhere else today.

## Coupling / watch out
- The trickiest part of this split is the `dragging` flag: today the SV
  square's and hue strip's `TouchArea`s each set `root.dragging = true` on
  pointer-down and `false` on pointer-up directly on the OWNING
  `ColorPicker`'s property. After extraction, `SvSquare`/`HueStrip` must
  instead emit `drag-started`/`drag-ended` callbacks that `ColorPicker`
  handles by setting its own `dragging` — get this wrong and the "don't
  reseed mid-drag" guard (the `changed value` handler, lines 74-84) breaks,
  which was explicitly called out as a hard-won fix in the file's own
  comments (a fully-derived model resets hue at s=0/v=0).
- `current` (`Colors.hsv(root.hue, root.sat, root.val)`) is computed on
  `ColorPicker` and must stay there — it's read by the swatch, the hex
  field, AND emitted via `changed(color)` — do not duplicate this
  computation inside `SvSquare`/`HueStrip`.
- The pure `compute-hue`/`decompose`/`hex-digit`/`byte-hex` functions all
  stay on `ColorPicker` (they operate on the incoming `value`, not on the
  drag areas) — no need to move them.
- No Timer is used anywhere (the file's own comment explicitly warns
  Timer-in-`if` + restart() panics the Slint compiler) — don't introduce one
  during the split.

## Verify after split
- Slint compile check succeeds with no unresolved imports/callbacks.
- Smoke-test in the running app: open the custom-theme editor, drag the SV
  square to each corner (verify hue survives at s=0 and v=0/v=1 edges — the
  documented regression risk), drag the hue strip, type a hex value and
  confirm it round-trips into the picker without jumping.
- Grep `crates/qbz-ui/ui/` for `ColorPicker` to confirm only the top-level
  file's import path is referenced by the theme-editor view, so the internal
  `SvSquare`/`HueStrip` split is invisible to importers.
