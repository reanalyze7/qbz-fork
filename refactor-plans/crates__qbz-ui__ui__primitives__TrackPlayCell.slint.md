# `crates/qbz-ui/ui/primitives/TrackPlayCell.slint` (228 lines)

## 1. Summary
The shared track-row leading cell (play affordance): an artwork-thumbnail
variant with a dark hover/playing overlay, or a plain track-number variant
that swaps to a play/pause glyph — both driven by shared active/playing/
loading state derived from `NowPlayingState`, plus an animated 3-bar
equalizer shown while the track is playing.

## 2. Proposed module layout

Given this is a `primitives/` file already used widely (`grep -rl
TrackPlayCell` → `AlbumPageView.slint`, `state.slint`, `ArtistPageView.slint`,
`TrackRow.slint`), keep the split shallow and inside `primitives/`:

- `primitives/TrackPlayCell.slint` (~115) — stays the export; keeps the `in`
  properties, the derived `is-active`/`is-playing`/`is-loading`/
  `show-overlay`/`show-bars` properties (these are the shared "brain" both
  variants read), the outer sizing (`width`/`height` switching on
  `show-artwork`), the `if root.show-artwork: ... / if !root.show-artwork:
  ...` dispatch to the two variant components below, and the single
  `TouchArea` (click toggles play/pause or fires `play()`).
- `primitives/EqualizerBars.slint` (~70) — `EqBar` + `EqualizerBars`
  components (the animated 3-bar equalizer), used identically by both
  variants. Preserve the comment explaining why it's driven by
  `ShellState.coarse-tick-ms` rather than `animation-tick()` (a documented
  CPU-usage fix, not incidental).
- `primitives/TrackPlayCellArtwork.slint` (~75) — the artwork-thumbnail
  variant body (cover image, dark overlay, equalizer bars, fetch spinner,
  play/pause glyph). Takes `artwork`, `art-size`, and the derived
  `show-overlay`/`show-bars`/`is-loading`/`is-playing`/`row-hovered` as `in`
  properties from the parent.
- `primitives/TrackPlayCellNumber.slint` (~75) — the number variant body
  (track-number text, equalizer bars, fetch spinner, circular play/pause
  glyph backing). Same `in` properties as above plus `number`/
  `number-width`.

## 3. Re-export / public API surface
`crates/qbz-ui/ui/primitives/TrackPlayCell.slint` remains the sole import
path for all 4+ existing importers — its public `in`/`out`
properties/callbacks (`track-id`, `number`, `artwork`, `show-artwork`,
`row-hovered`, `art-size`, `number-width`, `out hovered`, `callback play()`)
are unchanged; only its internal body is decomposed.

## 4. Tricky coupling to watch
- `out property <bool> hovered: ta.has-hover;` — the row-level callers OR
  this with their own `TouchArea`s to compute `row-hovered` because Slint's
  `has-hover` doesn't propagate to ancestor `TouchArea`s (per the existing
  comment). This property must stay defined on the OUTER
  `TrackPlayCell.slint` root (bound to the `ta` TouchArea that also stays at
  the root, per item below), not accidentally duplicated inside a variant
  sub-component.
- The single `ta := TouchArea` currently wraps the WHOLE cell (both variant
  branches render as siblings inside it implicitly, since it's declared
  after them at the root level) — keep the `TouchArea` at the
  `TrackPlayCell.slint` root, not inside either variant file, so a click
  anywhere on either variant still hits it.
- `is-active` has a documented dual-match rule (`NowPlayingState.track-id ==
  root.track-id` OR `local-track-id` fallback for offline-cache tracks) —
  this is exactly the kind of subtle business rule that must not be
  recomputed differently in two places; it's already centralized as one
  property on the root, keep it there and pass the DERIVED
  booleans (`is-active`/`is-playing`/`is-loading`/`show-overlay`/
  `show-bars`) down to the two variants rather than having each variant
  re-derive from `NowPlayingState` independently.
- Both variants currently share near-identical fetch-spinner
  `transform-rotation` expressions
  (`(ShellState.reduce-motion ? ShellState.coarse-tick-ms * 1.0 :
  animation-tick() / 1ms) * 0.36) * 1deg`) — keep this expression IDENTICAL
  in both `TrackPlayCellArtwork.slint` and `TrackPlayCellNumber.slint`
  (don't extract into a shared property unless it's trivial to do so,
  since the two spinners differ in size/tint).

## 5. What to verify after the real split
- Slint compile check / `cargo build -p qbz-ui`.
- Manual smoke-test across all consumers (`AlbumPageView`, `ArtistPageView`,
  `TrackRow`): hover a row (play triangle appears), click to play
  (equalizer bars animate), click again to pause, hover the currently
  playing row (pause glyph appears), and check the loading spinner appears
  correctly while a track is resolving — both the artwork and plain-number
  row styles need this pass (some views use one, some the other).
- Confirm `state.slint`'s reference to `TrackPlayCell` (if it's a type
  reference for a property, not just an import) still typechecks.
