# crates/qbz-ui/ui/album/TrackInfoModal.slint (364 lines)

## Summary
The Track Info modal (1:1 port of Tauri's `TrackInfoModal.svelte`): a
scrim + centered card showing track metadata (duration/quality/ISRC/label),
a two-column credits grid, and a copyright line, driven entirely by
`TrackInfoState`/`TrackInfoActions` globals from `state.slint`.

## Proposed split
Slint files split by extracting reusable sub-components into their own
files and importing them back, same convention as other `.slint` plans in
this batch. This file already exports two small reusable pieces
(`MetaCell`, `CreditCell`) at its top, used by BOTH this modal and (per the
file's own comment) the "immersive split Track Info panel" elsewhere — that
existing reuse is the natural split seam:

- `album/track_info/meta_cell.slint` (~30 lines) — the `MetaCell`
  component, unchanged, with its own `import` of `Theme`/`Typography`.
- `album/track_info/credit_cell.slint` (~45 lines) — the `CreditCell`
  component, unchanged, with its own imports (`Theme`, `Typography`,
  `InfoCreditRow` from `state.slint`).
- `album/TrackInfoModal.slint` (~300 lines, still slightly over 130 —
  see further split below) — the `TrackInfoModal` component itself,
  importing `MetaCell`/`CreditCell` from the two new files instead of
  defining them inline.

Since `TrackInfoModal` itself is still ~300 lines after extracting the two
sub-components, split its body further along its own visual sections
(mirrors the Svelte original's sections: header / metadata / credits /
copyright):

- `album/track_info/TrackInfoHeader.slint` (~65 lines) — the title/album/
  artist-link/close-X header block (currently inline in the "Header"
  section, lines ~166-226).
- `album/track_info/TrackInfoMetaRows.slint` (~70 lines) — the Duration/
  Quality/ISRC/Label `MetaCell` rows block (lines ~236-294).
- `album/track_info/TrackInfoCredits.slint` (~45 lines) — the two-column
  credits grid block (lines ~296-333), taking `credits-left`/
  `credits-right` as `in` properties and re-emitting `name-clicked`.
- `album/TrackInfoModal.slint` (~140 lines) — the outer scrim + card +
  `FocusScope`/`Flickable`/`ListScrollbar` shell, composing
  `TrackInfoHeader` + `TrackInfoMetaRows` + `TrackInfoCredits` +
  (inline) the copyright block (small enough to leave inline).

## Re-export surface
`album/TrackInfoModal.slint` stays the single import surface — every
caller (`crate::info_modals` in Rust, and whatever top-level `.slint` file
mounts the modal in `AppShell`) keeps doing
`import { TrackInfoModal } from "album/TrackInfoModal.slint";` unchanged.
The new sub-component files are internal implementation details, imported
only by `TrackInfoModal.slint` itself (and, per the file's existing
comment, potentially by the separate "immersive split Track Info panel"
file, which should switch to importing `MetaCell`/`CreditCell` from their
new locations instead of duplicating them — check that file when doing the
actual split, since it's explicitly called out as reusing these exports
"1:1").

## Coupling / watch out
- `MetaCell`/`CreditCell` are `export component`s — grep the whole
  `qbz-ui/ui/` tree for other importers of `TrackInfoModal.slint` pulling
  just `MetaCell`/`CreditCell` (the doc comments claim an "immersive split
  Track Info panel" does exactly this) before moving them, so that other
  file's import path gets updated too.
- `card.credits-col-w` is a property computed on the outer `card :=
  Rectangle` and passed down into `CreditCell.col-w` on both left/right
  columns — if `TrackInfoCredits.slint` is extracted, this width must be
  passed in as an `in property <length>` from the parent shell rather than
  reached via `card.` (Slint has no cross-file implicit parent access).
- The `TrackInfoActions.close()` / `.open-artist()` / `.open-label()` /
  `.open-musician()` global callbacks are called from multiple extracted
  sections (header close-X + artist link, meta-rows label link, credits
  name-click) — each extracted sub-component needs its own `import {
  TrackInfoActions } from "../../state.slint";` (relative path adjusted per
  new file depth).
- The `fs := FocusScope` Escape-key handler and the `Timer`-based
  focus-on-mount trick must stay in the outer shell (`TrackInfoModal.slint`)
  since they wrap the whole scrollable body, not any one section.

## Verify after split
- Run the project's Slint compile check (`slint-viewer` smoke check or
  whatever `cargo build -p qbz-ui` / `cargo build -p qbz` triggers for
  `.slint` codegen) — a broken relative import path is the most likely
  failure mode here.
- Manually open the Track Info modal in the running app (via the NPB info
  button, a song-card title, or TrackRow context menu → "Track info") and
  confirm: header renders (title/album/artist link/close), metadata row
  renders (Duration/Quality/ISRC/Label), credits two-column grid renders
  and each name is still clickable to musician, copyright line still shows
  when present, Escape still closes the modal, and the scrollbar still
  tracks the Flickable viewport.
- If an "immersive split Track Info panel" file elsewhere imports
  `MetaCell`/`CreditCell` from this file today, update and smoke-test that
  file's rendering too.
