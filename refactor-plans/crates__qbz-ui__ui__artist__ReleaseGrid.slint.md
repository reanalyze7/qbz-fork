# crates/qbz-ui/ui/artist/ReleaseGrid.slint (142 lines)

## Summary
Wrapping grid of `AlbumCard`s for an artist page's release sections (Albums,
EPs & Singles, Live, Compilations, …): a header (title + per-section sort +
"See discography" link) over an absolutely-positioned card grid, plus a
"Load more" footer link — a partial preview (no item count shown, since
`has-more` means the count would mislead).

## Proposed split
Only 12 lines over budget, so a minimal split: extract the header row into
its own component, leaving the grid + load-more in the main file.

- `artist/release_grid_header.slint` (~60 lines) — new component
  `RgHeader`, extracted from the `if !root.hide-header: HorizontalLayout { ... }`
  block (lines 44-91): title text, "See discography" link (`callback see-all()`),
  and the sort `QbzSelect` (`in property <int> sort-index`,
  `callback sort-changed(string)`).
- `artist/ReleaseGrid.slint` (~95 lines) — the slimmed main export: keeps
  `in property <ArtistReleaseSection> section`, `in property <bool> hide-header`,
  the four callbacks (`album-clicked`, `media-action`, `set-section-sort`,
  `load-more-section`, `open-releases`), mounts
  `if !root.hide-header: RgHeader { ...; see-all => { root.open-releases(root.section.release-type); } sort-changed(s) => { root.set-section-sort(root.section.release-type, s); } }`,
  then the existing absolutely-positioned `grid` `Rectangle` and the
  "Load more" footer.

## Re-export surface
`artist/ReleaseGrid.slint` stays the file other `.slint` imports
(`import { ReleaseGrid } from "../artist/ReleaseGrid.slint"`) — unchanged
export name, now a thin composition including the new header component.

## Coupling / watch out
- `root.sort-index` (derived from `root.section.sort-by` via the 5-way
  ternary chain) currently feeds the inline `QbzSelect.current-index` — move
  that derivation into `RgHeader` as its own `property <int> sort-index:` computed
  from an `in property <string> sort-by` passed down from the main
  component's `root.section.sort-by`, rather than re-deriving it from a
  full `ArtistReleaseSection` prop (keeps `RgHeader` decoupled from the
  section struct's other fields).
- `RgHeader`'s sort selection currently calls
  `root.set-section-sort(root.section.release-type, <sort-string>)` — since
  `release-type` lives on the main component's `section` prop, either pass
  `release-type` into `RgHeader` as an `in property <string>` too, or (simpler)
  have `RgHeader` emit a plain `callback sort-changed(string /* sort key */)`
  and let the main `ReleaseGrid.slint` supply the `release-type` argument
  when forwarding to `set-section-sort` — the plan above uses this simpler
  approach.
- The grid's absolute-positioning math (`columns`/`rows`/height calc based
  on `card-width`/`card-height`/`gap`) and the `for album[i] in root.section.albums`
  loop stay in the main file untouched — this part isn't over budget on its
  own and doesn't need touching.

## Verify after split
- Slint compile check for the crate.
- Manual smoke test on an artist page: each release section's header (title,
  "See discography" link navigates correctly, sort dropdown changes the
  section's order and persists the selected index across re-renders), the
  card grid reflows correctly at different window widths (columns
  recompute), and "Load more" appends further items when `has-more` is true.
- Grep for `ReleaseGrid` usage (including the collapsible "Other" section
  that passes `hide-header: true`) to confirm both call sites still work
  identically.
