# crates/qbz-ui/ui/myqbz/MixtapeDetailView.slint (1260 lines)

## Summary
The My QBZ Mixtape/Collection detail read-only view (spec 12): 5 private
helper components (`MenuOption` filter checkbox row, `ToolBtn` toolbar
trigger, `RowMenuItem` context-menu entry, `DetailRow` 8-col item row,
`DetailCard` grid card) plus the large `MixtapeDetailView` export (hero
header, empty-state, sticky toolbar, list/grid body). Most hero CTAs and
per-row context actions are logging stubs — a read-only boundary, per the
file's own header comment; several features (rename/delete/cover modals,
inline track expansion, BulkActionBar, live resolveItems) are explicitly
DEFERRED.

## Proposed split
- `mixtape_detail/mod.slint` (or keep `MixtapeDetailView.slint` itself,
  ~10 lines) — re-export shim so
  `import { MixtapeDetailView } from ".../MixtapeDetailView.slint"` keeps
  working unchanged.
- `mixtape_detail/menu_parts.slint` (~110 lines, new) — `MenuOption`
  (lines 47-78) + `ToolBtn` (79-152): the small toolbar/filter-menu
  building blocks.
- `mixtape_detail/row_menu.slint` (~35 lines, new) — `RowMenuItem`
  (154-188).
- `mixtape_detail/detail_row.slint` (~300 lines, new) — `DetailRow`
  (190-488), the 8-col item row — this alone may still need a second
  split (e.g. separating the leading checkbox/artwork cell from the
  trailing quality/menu cells) once the implementer sees the actual body;
  flag as "verify under 130 after extraction, split further if not."
- `mixtape_detail/detail_card.slint` (~100 lines, new) — `DetailCard`
  (489-590), the grid-mode card.
- `mixtape_detail/hero_header.slint` (~200 lines, new) — the `.header-
  content` block (lines 657-859): cover/mosaic, eyebrow/title/description/
  meta, and the play/shuffle/dj-mix/edit/delete/sync action row.
- `mixtape_detail/toolbar.slint` (~size TBD, new) — the sticky
  `.list-controls` block (from line 879): search + sort + type-filter +
  source-filter + reset + select + view-mode segmented group.
- `mixtape_detail/body.slint` (~size TBD, new) — the empty-state (860-
  873) + the list/grid body that instantiates `DetailRow`/`DetailCard`.
- `MixtapeDetailView.slint` itself (~100-150 lines) — the root component:
  composes hero header + toolbar + body, holds root-level state (search/
  sort/view-mode selection if any lives here rather than in
  `MyQbzDetailState`).

Given the file's size (1260 lines) and density, this is necessarily a
multi-pass split — the implementer should re-verify exact line boundaries
for the toolbar/body sections (only located via header comments in this
pass) and expect `DetailRow`/`hero_header`/`toolbar` may each need one more
level of splitting to land under 130 lines.

## Re-export surface
`MixtapeDetailView.slint` must remain the file every caller imports
(`import { MixtapeDetailView } from ".../MixtapeDetailView.slint"`) — keep
it exporting the same component with no prop/callback changes; it is
purely driven by `MyQbzDetailState`/`MyQbzDetailActions` globals.

## Coupling / watch out
- The file explicitly documents itself as read-only-boundary: most hero
  CTAs and per-row context menu actions are **logging stubs**, not real
  actions — do not "fix" or wire these up as part of a line-count split;
  preserve the stub behavior and the DEFERRED-feature comments verbatim.
- Exact geometry constants are called out as spec-verbatim in the header
  comment (row height 56px; 8-col grid column widths 40/1fr/140/80/160/
  72/60/40, gap 12; grid auto-fill minmax(150,1fr) gap 20; root padding
  8 8 100 18) — these must not drift when code moves between files.
- `DetailRow` and `DetailCard` both render the same underlying item data
  in two different layouts (list vs grid view-mode) — keep both reading
  from the same `MixtapeDetailItem`/`TrackItem` shape so a future
  resolveItems wiring only needs to touch one data path.
- Many primitive imports (`ExpandableSearch`, `QbzSelect`, `MultiSelectBar`,
  `ListScrollbar`, `SelectionCheckbox`, `SourceGlyph`, ...) are used across
  different sections — each new file needs only the imports it actually
  uses, but don't drop imports still needed by code that stays behind.

## Verify after split
- `cargo build -p qbz-ui`.
- Manually open a Mixtape/Collection detail view: hero header renders
  (cover/mosaic, actions), toolbar search/sort/filter/view-mode toggle all
  still update the list, list/grid modes both render correctly, and the
  empty-collection state still shows.
