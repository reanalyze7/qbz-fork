# crates/qbz-ui/ui/primitives/SearchTrackHero.slint (250 lines)

## 1. Summary
The "most-popular" hero card for a TRACK top search result: a 160x220
vertical card with a 128px square artwork (hover scrim + favorite/play/more
overlay buttons wired to the shared `TrackMenuState`), plus a title/artist/
quality-detail text stack below.

## 2. Proposed module split
The private `OverlayButton` helper component (lines 24–56) is the clean seam
— it's a fully generic circular hover-action button also structurally
identical to `AlbumCard`'s own overlay button (per the file's own comment),
so extracting it is both a size-reduction AND a reuse win:

| New file | Owns | ~lines |
|---|---|---|
| `primitives/SearchTrackHero.slint` | Stays the re-export/orchestrator: module doc, imports, `export component SearchTrackHero` (card shell, artwork block including the hover scrim/TouchArea, the button row instantiating the imported `OverlayButton`, and the title/artist/quality text stack) | ~200 |
| `primitives/OverlayButton.slint` | The extracted `OverlayButton` component (circular hover-action button: icon, primary/secondary look, diameter/icon-size params, `clicked` callback) | ~50 |

Even after extracting `OverlayButton`, `SearchTrackHero.slint` itself is
still ~200 lines — a further split by visual region:

| New file | Owns | ~lines |
|---|---|---|
| `primitives/SearchTrackHero.slint` | Re-export/orchestrator: module doc, imports, `export component SearchTrackHero` shell + composes the two extracted blocks below | ~60 |
| `primitives/SearchTrackHeroArt.slint` | The artwork block (cover Image, placeholder glyph, hover scrim, `hover` TouchArea, the fav/play/more `OverlayButton` row + the `TrackMenuState` wiring for the "more" button) | ~120 |
| `primitives/SearchTrackHeroText.slint` | The title/artist/quality-label text stack | ~50 |

## 3. Re-export / public API surface
`primitives/SearchTrackHero.slint` remains the only file other `.slint`
files import (`export component SearchTrackHero`) — its `in property`
surface (`track`, `quality-label`) and `media-action` callback are
unchanged, so the SearchView "Most-popular" track-result slot keeps working
with zero edits. `OverlayButton` becomes a shared reusable import
(`import { OverlayButton } from "OverlayButton.slint";`) that
`SearchTrackHeroArt.slint` (or `SearchTrackHero.slint` directly, in the
2-file variant) uses — and note `AlbumCard.slint` has its own structurally
identical `OverlayButton`; a FUTURE follow-up (out of scope for this split,
which only touches this one file) could de-dupe those two, but that is a
behavior-preserving reuse improvement, not part of this line-count split.

## 4. Tricky coupling / shared-state to watch out for
- The "more" button's click handler writes SEVEN fields onto the global
  `TrackMenuState` singleton (`track-id`, `source`, `cache-status`,
  `row-inert`, `force-local-menu`, `remove-from-playlist-action`,
  `track-info-action`, `local-goto-actions`, `x`/`y` via
  `absolute-position`, `open-token`) before incrementing `open-token` — this
  is the exact same fragile "stamp a global singleton then bump a token"
  pattern used elsewhere for the unified shell-level context menu; keep ALL
  of these field-writes together in one block wherever the "more" button
  ends up, since a partial move risks dropping a field silently (Slint won't
  error on an incomplete `TrackMenuState` initialization — it'll just
  misbehave at menu-open time).
- `root.absolute-position.x - root.x + 20px` / `+ 48px` are anchor
  coordinates relative to the ORIGINAL `root` (the whole `SearchTrackHero`,
  160x220) — if the "more" button moves into a child component
  (`SearchTrackHeroArt`), `root` there refers to the child's own root, not
  the outer card, so these offsets must be recomputed relative to whichever
  component's `root` actually ends up owning the TouchArea (either pass the
  outer card's width/position in as an `in property`, or keep the "more"
  button's click handler in the outer `SearchTrackHero.slint` file with the
  button itself passed down as a plain visual + callback).
- `overlay-on` (drives both the scrim opacity and the button-row opacity) is
  computed from THREE different TouchAreas' `has-hover` (`hover`, `fav-btn`,
  `play-btn`, `more-btn`) — if the buttons move to a child component, this
  aggregate hover state needs those child components to expose their own
  `hovered` output property (which `OverlayButton` already does:
  `out property <bool> hovered`) so the parent can still OR them together.

## 5. What to verify after the real split
- `cargo build -p qbz-ui` (Slint compile-time check).
- Manual smoke test: run a search, look at the "Most-popular" track hero
  card — hover to confirm the scrim + favorite/play/more buttons fade in,
  click play (starts the track), click favorite (toggles), click "more" and
  confirm the unified context menu opens anchored correctly under the
  button (not offset incorrectly after the coordinate-math move).
