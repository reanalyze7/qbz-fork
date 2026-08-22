# crates/qbz-ui/ui/album/AlbumCreditsModal.slint (476 lines)

## Summary
The Album Info (Credits/Review) modal: a scrim + centered card with a fixed
left column (artwork + label/date/meta/quality) and a right column that tabs
between an expandable per-track Credits list (`AlbumTrackRow`, with
per-performer credit rows) and a plain Review text block, plus a
loading/error overlay.

## Proposed split
By component boundary (the file already has one clean natural seam:
`AlbumTrackRow` vs the modal shell):

- `album/AlbumCreditsModal.slint` (~120 lines) — stays the re-export surface:
  module doc, imports, `export component AlbumCreditsModal` shell (scrim,
  outer sizing math, `FocusScope`/Escape handling, header with title+close),
  delegating the body to the two new components below.
- `album/AlbumTrackRow.slint` (~145 lines) — the `AlbumTrackRow` component
  (lines 19–162: header row with number/play-on-hover/expand chevron, expanded
  performer credits + copyright, divider). Exported so
  `AlbumCreditsModal.slint` can import it; still just over budget — if so,
  split the expanded-credits block (lines 111–155, ~45 lines) into
  `AlbumTrackCredits.slint` and have `AlbumTrackRow` compose it.
- `album/AlbumCreditsBody.slint` (~190 lines) — the two-column body (lines
  268–443: left `leftcol` artwork+meta, right tab-switcher + Flickable content
  with the Credits/Review `for` loop + `ListScrollbar`). Takes
  `AlbumInfoState`/`AlbumInfoActions` directly (same globals, no new plumbing
  needed) and imports `AlbumTrackRow`.
- `album/AlbumCreditsLoadingOverlay.slint` (~35 lines) — the
  loading/error overlay (lines 446–472), small enough it could also just stay
  inline in the main file if the main file has room; only pull out if needed
  to hit the 130-line target.

## Re-export surface
`album/AlbumCreditsModal.slint` remains the only file imported by callers
(`AlbumPageView` opens it via `media-action("album", id, "info")`) —
`export component AlbumCreditsModal` keeps the same (implicit, no
properties/callbacks) signature.

## Coupling / watch out
- The outer card's `height:` binding (lines 186–192) references
  `hdr.preferred-height` and `leftcol.preferred-height` / `content.preferred-
  height` by Slint element id — since `hdr` stays in the main file but
  `leftcol`/`content` would move into `AlbumCreditsBody.slint`, this height
  formula CANNOT reach across the file boundary by id. Fix: either (a) keep
  the whole body (not just track rows) in the main file and only extract
  `AlbumTrackRow`, or (b) have `AlbumCreditsBody` expose its own
  `preferred-height` and reference `body.preferred-height` from the parent
  instead of the nested `leftcol`/`content` ids directly. Plan (b) is cleaner:
  wrap the extracted body in a root `Rectangle`/`VerticalLayout` and bind the
  modal card's height off `body.preferred-height` plus the header, computed
  the same way internally inside `AlbumCreditsBody` if needed — recommend
  testing this specific height math carefully since it's the trickiest part
  of this file's split.
- `AlbumTrackRow`'s local `expanded` state must remain per-row (it already is,
  as a component-local property) — no cross-row shared state to watch for.
- The Credits/Review tab switcher's `active-tab` and the `has-review` gate
  both come from `AlbumInfoState` directly — no prop drilling needed if
  `AlbumCreditsBody` also imports `AlbumInfoState`/`AlbumInfoActions`.

## Verify after split
- `cargo build -p qbz-ui`.
- Manual smoke test: open an album, click the (i) info button, verify the
  modal's height still sizes correctly (this is the risky part per the height
  binding note above), switch Credits/Review tabs, expand a track's credits,
  scroll a long tracklist, verify Escape closes it.
