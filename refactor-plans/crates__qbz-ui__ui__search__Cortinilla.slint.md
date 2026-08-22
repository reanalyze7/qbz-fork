# crates/qbz-ui/ui/search/Cortinilla.slint (482 lines)

## Summary
`Cortinilla` — the live as-you-type search dropdown overlay under the header
search box (a plain non-popup `Rectangle` so it doesn't steal hover/focus):
idle-auto-close timer, click-outside-dismiss scrims, and the scrolling result
panel (skeleton loading rows, top-result, labelled sections with "View more").

## Proposed split
By component (Slint files can hold multiple `component`s; split into
sibling files under the same directory, one exported root + private helper
components moved to their own files and imported):

- `search/CortinillaResultRow.slint` (~80 lines) — the `CortinillaResultRow`
  component (currently lines 36-114), exported so `Cortinilla.slint` can
  `import { CortinillaResultRow } from "./CortinillaResultRow.slint";`.
- `search/CortinillaSkeletonRow.slint` (~60 lines) — the
  `CortinillaSkeletonRow` component (lines 120-175), same export/import
  treatment.
- `search/Cortinilla.slint` (~130 lines after extraction, still needs an
  internal split of the root `Cortinilla` component itself) — keep the
  top-of-file doc comment, imports, and the `export component Cortinilla`
  shell: callbacks, `panel-width`/`search-box-width` properties, the idle-
  close `Timer` + activity probes, and the three click-outside scrims
  (~230 lines total for just the shell before the panel body) — this alone
  is over budget, so the panel's inner body should move to a dedicated
  sub-component:
  - `search/CortinillaPanel.slint` (~180 lines) — a new `component
    CortinillaPanel` covering everything currently inside `if root.visible:
    panel := Rectangle { ... }` (lines 265-481): the scroll-cap property, the
    keyboard scroll-into-view logic, the hover surface, and the whole
    `VerticalLayout`/`Flickable`/skeleton/top-result/sections body. This
    component takes the same `SearchState`/`ShellState` globals directly
    (Slint globals are ambient, not passed as props) plus forwards
    `row-clicked`/`view-more` callbacks up to `Cortinilla`.
  - `Cortinilla.slint` then just instantiates `CortinillaPanel { }` inside
    its `if root.visible:` block, keeping the shell (timer, scrims,
    positioning math) at ~110 lines.

## Re-export surface
`search/Cortinilla.slint`'s `export component Cortinilla` stays the only
symbol other `.slint` files import (`import { Cortinilla } from
"./search/Cortinilla.slint";`, almost certainly from `AppShell.slint` or
similar per the file's own doc comment about being "mounted as a LAST child
of AppShell"). `CortinillaResultRow`, `CortinillaSkeletonRow`, and the new
`CortinillaPanel` are internal to the `search/` directory and do NOT need to
be exported from `Cortinilla.slint` itself unless something outside this
directory already references `CortinillaResultRow` (grep before finalizing —
unlikely, since it's a private helper in the same file today, but Slint
`component` without a leading `export` is directory-import-only, and if
another `.slint` file already did `import { CortinillaResultRow } from
"./Cortinilla.slint"` this split would need it re-exported or that import
path updated to the new file).

## Coupling / watch out
- `CortinillaResultRow` is used both standalone (top-result) and inside a
  `for` loop (section rows) inside the panel body — after the split it's
  imported by `CortinillaPanel.slint`, not by `Cortinilla.slint` directly,
  since the panel is the only place it's instantiated.
- The idle-close `Timer` MUST stay in the root `Cortinilla` component, NOT
  inside `CortinillaPanel` — the file's own comment explains why: "a Timer
  in there cannot be restart()ed — 1.16 compiler panic — so the timer lives
  here" (referring to the `if`-conditional panel subtree). Moving the panel
  body into its own component does not change this constraint: the panel
  (and hence `CortinillaPanel`) is STILL instantiated inside an `if
  root.visible:` conditional, so the timer must remain in `Cortinilla.slint`
  itself, driven by `root.panel-hovered` which the panel reports back via a
  callback or two-way property.
- `panel-hovered` is currently set by a `TouchArea` (`hover-ta`) INSIDE the
  panel subtree and read by the root-level `Timer`'s `running` binding —
  after the split this becomes a genuine cross-component data flow: either
  (a) `CortinillaPanel` exposes an `out property <bool> hovered` that
  `Cortinilla.slint` binds `root.panel-hovered: panel.hovered;` to (cleanest,
  keeps the two-way sync automatic), or (b) `CortinillaPanel` gets an `in-out
  property <bool> panel-hovered` passed down. Prefer (a) — avoids a two-way
  alias for what's really a one-directional "report hover" signal.
- `sel-scroll-y`/`body-flick` keyboard-scroll-into-view logic reads
  `SearchState.cortinilla-scroll-y` (a global, ambient — no prop threading
  needed) and mutates `body-flick.viewport-y` imperatively in a `changed`
  handler — this logic is self-contained to what becomes `CortinillaPanel`,
  no cross-component coupling to worry about there.
- The three click-outside scrims reference `root.search-box-width` (computed
  from `ShellState.nav-in-sidebar`) — stays in the shell (`Cortinilla.slint`)
  since scrims are siblings of the panel, not part of it.
- `panel-width` (320px) is read by the shell to position the panel AND
  implicitly bounds `CortinillaPanel`'s content width — if `CortinillaPanel`
  needs to know its own width for internal layout, pass it as an `in
  property <length> panel-width` rather than hardcoding 320px twice.

## Verify after split
- Build the Slint UI (however this repo compiles `.slint` — check for a
  `slint-viewer` invocation, `build.rs` codegen step, or `cargo build -p
  qbz-ui`) and confirm no import-path errors.
- Smoke-test in the running app: type a query (>=2 chars) in the header
  search box, confirm the cortinilla opens with skeleton rows, then real
  results (top-result + sections), hover highlighting, click-to-navigate,
  "View more", idle auto-close after ~4.5s of no activity, keyboard arrow
  selection + scroll-into-view, and click-outside dismiss (both left/right of
  the search box AND below the header).
