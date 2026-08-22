# crates/qbz-ui/ui/primitives/CircleAction.slint (133 lines)

One-line summary: the circular header/action button (play/secondary actions) — just 3 lines over the cap; a light split suffices.

## Proposed split
- `CircleAction.slint` (~95 lines) — **stays the public re-export** (`export component CircleAction`). Keeps the root Rectangle, background/border logic, the TouchArea + tooltip wiring, and the FocusScope key handling.
- `CircleActionGlyph.slint` (~45 lines, new, not exported) — the icon-vs-spinner branch (`if !root.loading: QbzIcon { ... }` / `if root.loading: LoadingSpinner { ... }`, lines 62-86) plus the focus-ring overlay Rectangle (lines 101-112), parameterized by `diameter`, `primary`, `on-surface`, `active`, `loading`, `icon`, and the outer `FocusScope`'s `has-focus`.

## Tricky coupling to flag
- The focus-ring Rectangle needs `fs.has-focus` from the FocusScope declared in the main file — pass it in as a `in property <bool> focused` rather than trying to move the FocusScope too (FocusScope must stay adjacent to the TouchArea it complements).
- This exact "non-layout focus ring" pattern is duplicated near-verbatim in `QbzSelect.slint` and `ExpandableSearch.slint` (border-color: `fs.has-focus ? Theme.focus-ring : transparent`) — worth a shared `FocusRingOverlay` component in a later cross-file cleanup pass, out of scope for this split.

## Verify after split
- Compiles; every CircleAction caller (album/artist/label headers, Popular Tracks clusters, ephemeral pane) renders unchanged — primary play button and secondary ghost buttons both checked in dark and on-surface variants.
