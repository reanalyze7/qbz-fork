# crates/qbz-ui/ui/primitives/AlbumListRow.slint (438 lines)

## Summary
Two components in one file: `AlbumListCols` (a shared global defining fixed
column widths) plus `AlbumListHeader` (the column-header row) plus
`AlbumListRow` (the actual list row — thumbnail/title/artist/type/source/
quality/tracks/year/overflow-menu with a context menu), reused across
Discover "View all", Favorites, Local Library, and My QBZ.

## Proposed split
Slint files split by component/concern, same convention already used
elsewhere in this crate's directory-based structure (e.g. separate `*Row.slint`
vs `*Actions`/state files). Each new file keeps the `.slint` extension and is
imported via Slint's `import { X } from "./Y.slint";`.

- `AlbumListCols.slint` (~15 lines) — the `export global AlbumListCols { ... }`
  block (29-39) moved to its own file. Tiny but it is the shared contract both
  header and row (and every importer) depend on — isolating it means a future
  column-width tweak touches one small file instead of the 438-line one.
- `AlbumListHeader.slint` (~95 lines) — the `AlbumListHeader` component
  (43-132) plus its own `import { AlbumListCols } from "./AlbumListCols.slint";`.
- `AlbumListRow.slint` (~330 lines, still over budget) — the `AlbumListRow`
  component itself (134-438). This is the real offender and needs a second
  cut:
  - `AlbumListRowMenu.slint` (~90 lines) — extract the `row-menu :=
    PopupWindow { ContextMenu { ... } }` block (379-434, the 5
    ContextMenuItems: Open/Play/Play next/Add to queue/Favorite/Block) into
    its own component `AlbumListRowMenu` taking `album: AlbumCardItem` and
    exposing the same `clicked`/`media-action` callbacks it currently emits
    inline, plus a `show()` function forwarded from the parent's `more-ta`
    click. This is the single largest extractable chunk (~55 lines) and the
    most self-contained (it only reads `root.album` and re-emits the same
    two callback shapes already on `AlbumListRow`).
  - `AlbumListRow.slint` then shrinks to ~275 lines (still a bit over 130)
    — split further if needed by extracting the "artwork thumb with hover
    play glyph" block (216-258, ~40 lines) into a small `AlbumListArtCell.slint`
    (parameterized on `artwork: image, artwork-url: string, hovered: bool`,
    emitting a `play-clicked` callback), and/or the multi-select checkbox
    block (184-205) into `AlbumListSelectCheckbox.slint`. After both
    extractions `AlbumListRow.slint` lands around 190-210 lines of layout
    glue (TouchArea + HorizontalLayout wiring columns together) — if still
    over 130, the TYPE/SOURCE/QUALITY/TRACKS/YEAR column cells (which are
    each ~10-15 lines of near-identical `Text { width: AlbumListCols.X; ...
    }`) could be further factored into a tiny `AlbumListTextCell.slint`
    helper component parameterized on `width`/`text`/`align`, cutting ~40
    more lines of duplication.

## Re-export surface
No Rust re-export mechanics apply to Slint — the single import surface
other views keep using is `import { AlbumListRow, AlbumListHeader } from
"../primitives/AlbumListRow.slint";` (the existing import path, unchanged in
name/location). `AlbumListCols` becomes a NEW import some call sites will
need to add explicitly if they reference `AlbumListCols.xxx` directly
(check call sites — likely only `AlbumListHeader.slint` and
`AlbumListRow.slint` itself reference it, per the current file's own usage of
the global, so external callers are probably unaffected). Grep every `.slint`
file for `AlbumListCols` before assuming no external caller needs the new
import.

## Coupling / watch out
- `AlbumListHeader` and `AlbumListRow` MUST agree on `AlbumListCols` column
  widths pixel-for-pixel (that's the whole point of the shared global) —
  splitting is safe since Slint globals are singleton-imported, not
  duplicated, but do not accidentally inline a second copy of the widths
  into either extracted file.
- `AlbumListRowMenu`'s "Block this album" item is conditioned on
  `root.album.source != "local"` and calls the free function
  `BlacklistActions.block-album(...)` (a global, imported from `state.slint`)
  — the extracted menu component needs its own `import { BlacklistActions }
  from "../state.slint";`.
- The row's `row-hovered` property is computed from FOUR TouchAreas
  (`ta`/`art-ta`/`artist-ta`/`more-ta`) that live in different visual
  sub-blocks — if the art-cell or checkbox is extracted into a child
  component, `has-hover` on an inner TouchArea does not automatically
  propagate to the parent's `row-hovered` unless the child component exposes
  its own `out property <bool> hovered` that the parent binds into its
  `row-hovered` expression. This is the trickiest wiring in the split —
  verify hover-driven visuals (play glyph, background tint) still activate
  correctly after extraction.
- `art-ta`'s click handler branches on `root.select-mode` (toggle-select vs
  play) — this branching logic must move WITH the art cell if extracted, not
  stay behind in the parent (the child needs `select-mode: bool` as an input
  property).

## Verify after split
- The project's Slint compile check (however CI invokes it — `slint-viewer`
  smoke load or `cargo build` if UI is compiled via `slint-build`/build.rs)
  must succeed with zero warnings about unresolved imports.
- Visually smoke-test every importer context: Discover "View all" list mode,
  Favorites albums list, Local Library albums list (multi-select mode
  specifically, since `select-mode` is a real behavioral branch), My QBZ
  detail rows if applicable.
- Confirm the overflow "…" context menu still opens/positions correctly
  (the `PopupWindow` offset `x: -170px; y: 34px;` is relative to its parent
  Rectangle — verify this offset still reads correctly if the menu moved to
  a new component boundary).
