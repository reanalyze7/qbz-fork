# crates/qbz-ui/ui/discover/PlaylistTagFilter.slint (185 lines)

## Summary
A compact multi-select dropdown for filtering Qobuz Playlists by category tag
(replaces WebPlayer pills): a 30px collapsed button + a popup checkbox list
with a "Clear/All categories" footer.

## Proposed split
Just over budget. Split the reusable checkbox row out of the popup shell —
mirrors how other primitive rows (e.g. `FilterChip` in LocalLibraryView) live
standalone:

- `PlaylistTagFilter.slint` (~150 lines) — keeps the file's own imports, the
  `export component PlaylistTagFilter` (button + popup shell), importing
  `TagRow` from its new file.
- `primitives/TagRow.slint` (~50 lines) — the `component TagRow` (checkbox +
  localized name), a generic-enough shape it could be reused by other
  tag/category pickers later. Needs `Theme`, `Typography`, `PlaylistTagItem`,
  `QbzIcon` imports.

## Re-export surface
Slint has no re-export indirection — `PlaylistTagFilter.slint` stays the
single import surface every caller already uses (`import { PlaylistTagFilter }
from "../discover/PlaylistTagFilter.slint";`); it internally does
`import { TagRow } from "../primitives/TagRow.slint";`. No caller path changes.

## Coupling / watch out
- `TagRow` reads `root.tag.selected` / `root.tag.slug` / `root.tag.name` from
  the `PlaylistTagItem` state struct (`state.slint`) — must keep that import
  in the new file.
- The popup's height calculation (`Math.min(list-col.preferred-height,
  root.max-list-height) + 32px + 10px`) depends on `TagRow`'s fixed 32px
  row height — if `TagRow` height ever changes, this constant must be
  updated too; leave a comment noting the coupling when splitting.

## Verify after split
- `cargo build -p qbz-ui` (or the project's slint-viewer/compile check) to
  confirm both files compile and the import resolves.
- Visual smoke-test: open Discover > Playlists, click "Filter by category",
  confirm rows render/toggle and "All categories" clears correctly.
