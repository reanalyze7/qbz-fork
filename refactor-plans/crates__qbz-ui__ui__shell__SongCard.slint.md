# crates/qbz-ui/ui/shell/SongCard.slint (666 lines)

## Summary
The now-playing song card: cover art (+ idle placeholder, fetch-overlay
spinner, hover art-preview trigger), title + artist·album meta line (two
layout variants: default vs `text-center`), and an absolutely-pinned "audio
stamp" (quality badge + backend/mode LEDs). Used by New/Classic now-playing
modes and (via `text-center`/`narrow` params) the compact player bar.

## Proposed split
By responsibility (cover / text-block / audio-stamp are the natural seams —
this is presentation, so split by visual region rather than pure/IO):

- `shell/SongCard.slint` (~150 lines) — stays the re-export/orchestrator:
  module doc, imports, the two small helper components `MetaLink` and
  `DotLed` MOVE OUT (see below) but `SongCard` itself keeps its `in property`
  surface (all ~15 properties: `art-size`, `show-art`, `show-badges`,
  `art-border-width/color`, `title-font-size`, `meta-font-size`,
  `title-meta-gap`, `art-text-gap`, `text-center`, `narrow`, `glass`,
  `compact-stamp`, `dot-leds`) and callbacks, computing the shared derived
  properties (`ctx-kind`, `ctx-id`, `stamp-w`, `has-stamp`, `title-avail`,
  `quality-*`) since these are read by both the cover and text sub-parts, then
  composes the 3 extracted regions.
- `shell/SongCardArt.slint` (~90 lines) — the cover/idle-cat/fetch-overlay/
  hover-preview block (lines 246–315: the `if root.show-art: VerticalLayout`).
  Takes `art-size`, `art-border-width/color` as `in property` and reads
  `NowPlayingState`/`ArtPreviewState` directly (same as today).
- `shell/SongCardInfo.slint` (~230 lines) — the title+meta text column (lines
  320–575), covering BOTH the `text-center` and default layouts, since they
  share the `MetaLink`-based artist/album row logic. Still likely over 130 —
  split further into `SongCardInfoCentered.slint` (~100, the `text-center`
  branch) and `SongCardInfoDefault.slint` (~110, the default branch), both
  importing the shared `MetaLink` component.
- `shell/SongCardStamp.slint` (~90 lines) — the audio-stamp block (lines
  584–665: `QualityBadgeFull` + downgrade arrow + `DotLed` row), taking
  `stamp-w`, `compact-stamp`, `narrow`, `dot-leds`, and the pre-computed
  `show-delivered`/`quality-stamp-tooltip` as `in property` params.
- `shell/SongCardHelpers.slint` (~100 lines) — the two small standalone
  components `MetaLink` (lines 20–51) and `DotLed` (lines 58–101), shared by
  `SongCardInfo*` and `SongCardStamp` respectively.

## Re-export surface
`shell/SongCard.slint` remains the only file other `.slint` files import
(`export component SongCard`) — its public property/callback surface is
unchanged, so every existing call site (now-playing New/Classic, PlayerBar,
the "Small" compact bar) needs zero edits.

## Coupling / watch out
- `title-avail` is computed in the parent from `root.width`, `has-stamp`,
  `stamp-w`, `show-art`, `art-size`, `art-text-gap` — it must be passed DOWN
  into `SongCardInfo*` as an `in property <length>` rather than recomputed,
  since it depends on the parent's absolute width.
- The absolutely-positioned stamp (`x: root.width - root.pad - ...`) depends
  on `root.width`/`root.height` of the OUTER card, not the stamp's own
  container — when extracted, `SongCardStamp` needs `card-width`/`card-height`
  passed in, or stay positioned by the parent (i.e. parent does
  `SongCardStamp { x: ...; y: ...; }` rather than the child self-positioning).
- `MetaLink`'s `stretch` property (artist=2.0, album=1.0) encodes an elision
  priority contract described in a comment — preserve exactly when moving to
  `SongCardHelpers.slint`.
- `ttta`/`tc-ta` TouchArea hover logic pushes `TooltipState.text` etc. — these
  reference `NowPlayingState.title`/`title-txt.preferred-width` local to
  whichever info-variant file owns them; no cross-file id references needed
  here (unlike FavoritesView's Flickable/AlphaStrip pairing) since each
  TouchArea's Text sibling stays in the same file.
- `ctx-kind`/`ctx-id` (computed from `NowPlayingState.context-id` etc.) are
  used by the context/layers button in BOTH the centered and default info
  layouts — keep these as parent-computed `in property` passed to both.

## Verify after split
- `cargo build -p qbz-ui` (Slint compile-time check across all `.slint` files).
- Manual smoke test: play a track, check New mode, Classic mode, and the
  compact/Small player bar all render cover + title/artist/album + audio stamp
  identically to before the split (hover tooltips, art-preview overlay,
  quality-downgrade arrow, dot-LEDs all still fire).
