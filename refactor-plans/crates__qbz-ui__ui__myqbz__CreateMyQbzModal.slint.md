# crates/qbz-ui/ui/myqbz/CreateMyQbzModal.slint (199 lines)

## Summary
The "Create Mixtape / Collection" modal: a Name field, a Kind radio toggle
(Mixtape vs. Collection), and Cancel/Create buttons — opened from either
MyQBZ grid's "+ New" CTA.

## Proposed split
This file is only modestly over budget (199 lines); a single small
extraction (the private `KindRadio` component) plus trimming brings the
root under 130.

- `CreateMyQbzModal.slint` (~125 lines) — module doc, imports, `export
  component CreateMyQbzModal`: the scrim/backdrop, panel sizing, header
  (title + close X), the Name field, and the Cancel/Create button row.
  Imports `KindRadio` for the Kind toggle section.
- `myqbz/KindRadio.slint` (~35 lines) — the `KindRadio` component (lines
  22-55), exported.

## Re-export surface
`CreateMyQbzModal.slint` stays the single import surface (the MyQBZ page's
`import { CreateMyQbzModal } from "./myqbz/CreateMyQbzModal.slint";` is
unaffected); it gains one new sibling import
(`import { KindRadio } from "./KindRadio.slint";`).

## Coupling / watch out
- `KindRadio`'s `selected`/`label` are `in property`s and its `clicked()`
  is a plain `TouchArea` callback already — it has zero coupling to
  `CreateMyQbzModal`'s internals beyond being placed inside the Kind
  toggle's `HorizontalLayout`, so this extraction is low-risk.
- The Kind-toggle click handlers (lines 152-169) directly mutate
  `MyQbzCreateState.kind` (a Slint global) guarded by `!MyQbzCreateState.creating`
  — this logic stays in the root file's own `KindRadio { clicked => {...} }`
  instantiation, not inside `KindRadio.slint` itself (the component is a
  dumb radio-button visual; the state-mutation stays at the call site,
  matching the existing pattern of every other component in this repo).
- `root.is-mixtape` (computed from `MyQbzCreateState.kind`) and
  `root.can-create` are used both by the title/button-label text and by
  the Kind-toggle's `selected` bindings — no extraction risk since they
  stay on `root`.

## Verify after split
- `cargo build` through the Slint build step to confirm compilation.
- Smoke-test: open the modal from MyQBZ's "+ New" CTA on both the
  Mixtapes and Collections grids, confirm the title/button label switch
  correctly, the radio toggle switches `kind`, Create is disabled while
  the name is blank or a create is in flight, and Enter/Create both submit.
