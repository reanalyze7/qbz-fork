# crates/qbz-ui/ui/settings/BlacklistManagerView.slint (1114 lines)

## Summary
The standalone Artist/Album/Recommendations blacklist manager screen: 3
private near-identical row components (`ArtistRow`, `AlbumRow`,
`DismissedRow`) plus the large `BlacklistManagerView` export (header, tab
bar, description, controls row with search + Clear-All + count badge,
disabled-warning banner, a 4-branch body per tab × loading/empty/no-
results/list, and a Clear-All confirm modal).

## Proposed split
- `BlacklistManagerView.slint` (~120 lines) — **stays the public
  re-export/root**: top bar (Back button), header, tab bar, description,
  and composes the sections below; imports the row components and the
  new sub-views.
- `BlacklistRows.slint` (~310 lines, new) — `ArtistRow` (31-135),
  `AlbumRow` (138-246), `DismissedRow` (251-336). These three are
  structurally parallel (avatar/cover + name/meta + remove-X) and belong
  together as the "list row" module.
- `BlacklistControlsRow.slint` (~170 lines, new) — enable/disable toggle,
  search box + clear ×, Clear-All button, count badge (lines 479-647).
- `BlacklistDisabledBanner.slint` (~35 lines, new) — the amber warning
  banner (lines 648-679).
- `BlacklistArtistsBody.slint` (~110 lines, new) — the Artists-tab
  loading/empty/no-results/list branches (roughly 680-798), instantiating
  `ArtistRow` from `BlacklistRows.slint`.
- `BlacklistAlbumsBody.slint` (~size TBD, new) — the parallel Albums-tab
  branches (from ~799 onward), instantiating `AlbumRow`.
- `BlacklistRecoBody.slint` (~size TBD, new) — the Recommendations-tab
  branches, instantiating `DismissedRow`.
- `BlacklistClearAllModal.slint` (~size TBD, new) — the Clear-All confirm
  overlay near the end of the file (declared last per ADR-009 so it
  z-stacks above everything — this ordering constraint must be preserved
  wherever it's instantiated from the root).

The implementer should re-check exact line ranges for the three per-tab
body blocks (this pass only located the Artists-tab body precisely; the
Albums/Recommendations bodies mirror it further down — grep for
`active-tab == 1` / `active-tab == 2` to find their boundaries) and adjust
file boundaries so each lands under 130 lines, further splitting a body
file by loading/empty/list branch if needed.

## Re-export surface
`BlacklistManagerView.slint` keeps exporting `BlacklistManagerView` with
no props/callbacks (it's fully state/action-global driven via
`BlacklistState`/`BlacklistActions`) — importers only need the component
name unchanged.

## Coupling / watch out
- All three row components and all body branches read `BlacklistState`
  and call `BlacklistActions.*` directly (ambient globals) — splitting
  is low-risk for state threading, but every new file needs its own
  `import { BlacklistState, BlacklistActions, ... } from "../state.slint";`.
- Body branch order is explicitly called out as **load-bearing** in the
  file's own header comment (mutually-exclusive `if` conditions must
  evaluate in the documented precedence) — when moving branches into
  separate per-tab files, keep each file's internal branch order intact.
- The Clear-All modal's z-order (declared last, ADR-009/010) must survive
  the split — instantiate it last in `BlacklistManagerView.slint`'s tree
  regardless of which file defines it.
- `count` (full list count) vs `items` (search-filtered) semantics differ
  per the header comment — any body-branch file must keep using the
  correct one for its empty/no-results distinction.

## Verify after split
- `cargo build -p qbz-ui`.
- Manually exercise all three tabs (Artists/Albums/Recommendations):
  empty state, populated list, search-as-you-type + clear, Clear-All
  confirm flow, and the disabled-warning banner toggle.
