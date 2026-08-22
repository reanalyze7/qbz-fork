# crates/qbz-ui/ui/location/ArtistsByLocationView.slint (173 lines)

## Summary
"Artists from the same place" scene-discovery view: a scene header
(location icon + name + genre summary) over a wrapping `ArtistGridCard` grid
with manual "Load more" pagination, opened from an Origin section's location
link.

## Proposed split
Only modestly over budget (173 lines). The cleanest cut is extracting the
scene header block, since it's the one visually/logically distinct region
that doesn't touch the `Flickable`/grid-index-math the rest of the file is
built around.

- `LocationSceneHeader.slint` (~45 lines) — the "Scene header" VerticalLayout
  block (65-95): the map-pin icon + scene-label text row, plus the optional
  genre-summary text. Takes `scene-label: string, genre-summary: string` as
  inputs; no callbacks needed (purely presentational).
- `ArtistsByLocationView.slint` (~130 lines after extraction) — keeps the
  `Flickable` + scroll-restore idiom (32-46), the loading/empty states
  (99-109), the `grid := Rectangle { for item[i] in ... }` manual-grid-math
  block (112-133, deliberately NOT extracted — the `columns`/`rows`
  computation and the `for` loop's `x`/`y` positioning are tightly coupled
  and splitting them would obscure rather than clarify), the "Load more"
  affordance (135-160), and the `ListScrollbar` (165-173).
- If still a few lines over 130 after the header extraction, additionally
  pull the "Load more" button block (136-160, ~25 lines) into a tiny
  `LoadMoreButton.slint` (`in property <bool> loading; callback clicked();`)
  — this pattern (a text button that swaps its label to "Loading…" while a
  fetch is in flight) likely repeats across other paginated views in this
  codebase (worth a note for whichever agent covers those), but do the
  extraction here regardless since it directly gets this file under budget.

## Re-export surface
No Rust-style re-export mechanics — the one import surface other files keep
using is `import { ArtistsByLocationView } from
"../location/ArtistsByLocationView.slint";` (wherever the Origin-section
location link routes to it), unchanged. `LocationSceneHeader` and (if
extracted) `LoadMoreButton` are internal-only additions; no other `.slint`
file needs to import them for this view to keep working.

## Coupling / watch out
- Same scroll-restore-idiom duplication note as `ArtistReleasesView.slint`
  (keyed here by the `"location"` restore-scope string) — do not deduplicate
  across files as part of this split; just flag it, since another agent may
  be covering `ArtistReleasesView.slint` in the same wave and could
  independently notice the same pattern.
- The grid's `columns`/`rows` computation (`Math.max(1, Math.floor((self.width
  + gap) / (card-width + gap)))`) is read by the `for item[i] in
  LocationViewState.artists: ArtistGridCard { x: ...; y: ...; }` loop
  immediately below it in the SAME Rectangle (`grid`) — these must stay in
  the same component; extracting the grid body without its own `columns`/
  `rows` properties would break the manual x/y positioning math.
- `LocationViewState.scene-label`/`genre-summary` are read directly by the
  extracted header — pass them as explicit `in property` bindings from the
  parent rather than having the child import `LocationViewState` itself,
  to keep the header a "dumb" presentational component (matches how
  `ArtistReleasesHeader` was planned for the sibling file).

## Verify after split
- Slint compile check (project's build-time compilation / `slint-viewer`
  load) with no unresolved-import warnings.
- Visual smoke test: open an artist's Origin section, click a location link,
  confirm the scene header (icon + label + genre summary) still renders,
  the artist grid still wraps/reflows on resize, "Load more" still paginates,
  and the empty/loading states still show correctly for a scene with zero
  artists.
