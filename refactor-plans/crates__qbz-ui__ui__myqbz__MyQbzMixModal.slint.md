# crates/qbz-ui/ui/myqbz/MyQbzMixModal.slint (184 lines)

## Summary
The "Random queue" DJ-mix sampler modal: a backdrop + centered panel with a
title, body copy, a loading state, a size-picker (slider + "N songs available"
label), and a Cancel/"Add to queue" footer, driven by `MyQbzMixState`/
`MyQbzMixActions`.

## Proposed split
Modestly over budget (184 lines); the natural cut is the size-picker field,
which is the most self-contained visual chunk (its own loading/loaded
conditional, slider, and helper text) and could be reused by any future
"pick a sample size" modal:

- `myqbz/MyQbzMixModal.slint` (~120 lines) — kept as the main file / export
  surface: the backdrop, panel shell, title, body copy, footer
  (Cancel/"Add to queue"), and the `can-shuffle`/`size-label` properties (these
  drive the footer button state, so they stay with the shell). Composes the new
  `MyQbzMixSizePicker` sub-component in place of the inline loading/slider
  block.
- `myqbz/MyQbzMixSizePicker.slint` (~75 lines) — new component wrapping the
  "Loading…" spinner state AND the loaded slider + "Number of songs" label +
  "{n} songs available" helper text as one unit, taking `loading`,
  `size-options.length`, `selected-index`, `selected-size`, `selected-is-all`,
  `unique-count` as in-properties and firing a `set-index(int)` callback (a
  thin proxy to `MyQbzMixActions.set-index`, or the modal can still call
  `MyQbzMixActions` directly from a callback the sub-component forwards).

## Re-export surface
`myqbz/MyQbzMixModal.slint` remains the single import path
(`import { MyQbzMixModal } from "myqbz/MyQbzMixModal.slint";` or similar) that
the rest of the app uses to mount this modal — extracting the size-picker into
its own file and importing it back does not change `MyQbzMixModal`'s export
name/location.

## Coupling / watch out
- `root.can-shuffle` and `root.size-label` (computed properties on the modal
  root) are used by the footer's primary button (`enabled: root.can-shuffle`)
  AND by the extracted size-picker (`size-label` display, `can-shuffle`
  indirectly via `size-options.length`/`selected-size` > 0 checks) — decide
  whether `size-label` moves into the new sub-component (computed from
  in-properties there) or stays on the parent and gets passed in; moving it
  into `MyQbzMixSizePicker` (computed locally from its own in-properties) is
  cleaner and avoids a redundant property pass-through.
- The size-picker's slider directly calls `MyQbzMixActions.set-index(v)` today
  — the extracted component can either import `MyQbzMixState`/`MyQbzMixActions`
  itself (simplest, matches how `SlimCarousel`'s children already reach global
  singletons directly) or expose a `changed(int)` callback the parent wires to
  `MyQbzMixActions.set-index`. Prefer the direct-global-access approach for
  consistency with the rest of this codebase's Slint conventions (globals are
  used freely across components here, per `ShellState`/`AppearanceState` usage
  elsewhere).
- Keep the "Per spec §23 the button labels are HARDCODED English..." comment
  and the overall modal-behavior doc comment on the main `MyQbzMixModal.slint`
  file — it documents the whole modal's contract, not just the size field.

## Verify after split
- Run the Slint build/viewer check for `qbz-ui`.
- Smoke-test: open the "Random queue" modal from the My QBZ hero CTA, confirm
  the loading state shows then resolves to the slider, dragging the slider
  updates the size label and "songs available" text, Cancel and backdrop-click
  close without committing, and "Add to queue" is disabled/enabled correctly
  and replaces the queue on confirm.
