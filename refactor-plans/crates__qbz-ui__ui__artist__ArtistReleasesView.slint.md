# crates/qbz-ui/ui/artist/ArtistReleasesView.slint (257 lines)

## Summary
The dedicated, paginated artist-discography page (reached via "See
discography"): a fixed header (nav buttons + artist kicker/bucket title +
sort dropdown) over an infinite-scroll `Flickable` grid with loading/error/
empty states, driving `ArtistReleasesActions.load-more()`.

## Proposed split
Single component, but with clearly separable visual regions — split into a
few small child components imported back into the main view, keeping
`ArtistReleasesView` itself as thin layout glue.

- `ArtistReleasesHeader.slint` (~75 lines) — the "Fixed header" Rectangle
  block (36-103): nav buttons + kicker/title on the left, the `QbzSelect`
  sort dropdown on the right. Takes `name: string, title: string, sort-index:
  int` as inputs and emits a `sort-selected(int)` callback the parent maps to
  `ArtistReleasesActions.set-sort(...)`. This is the most self-contained
  extraction (pure header, no Flickable/state coupling beyond simple props).
- `ArtistReleasesGridStates.slint` (~90 lines) — the loading spinner block
  (140-156), the error+retry block (158-193), and the empty-state block
  (195-203) — these three mutually-exclusive states are currently inline
  siblings inside `page`; group them into one component
  `ArtistReleasesGridStates` with `in property <bool> loading, load-error;
  in property <bool> is-empty;` and a `retry()` callback, reducing the parent
  to one conditional include.
- `ArtistReleasesView.slint` (~110-120 lines after both extractions) — keeps
  the `Flickable`/scroll-restore logic (106-129, tightly coupled to
  `NavState.restore-scope`/`scroll-restore` and must NOT be extracted since
  splitting a `Flickable` from its `changed viewport-y` handlers is
  error-prone), the `AlbumCollectionView` grid itself (205-231, thin
  pass-through of state + callbacks), the `load-more-loading` footer spinner
  (233-237), and the `ListScrollbar` (242-256).

## Re-export surface
Slint has no explicit re-export step — the one import surface other files
keep using is unchanged: `import { ArtistReleasesView } from
"../artist/ArtistReleasesView.slint";` (used from wherever routing mounts
this view, likely `main.slint` or a shell router). The two new child
components (`ArtistReleasesHeader`, `ArtistReleasesGridStates`) are internal
to this view only — no other `.slint` file needs to import them, so no
external caller is affected either way.

## Coupling / watch out
- The scroll-restore idiom (`property <string> sr-armed: NavState.restore-
  scope; function sr-restore() { ... } init => {...} changed sr-armed =>
  {...} changed viewport-height => {...}`) is copy-pasted near-verbatim
  across several views in this codebase (this file, `ArtistsByLocationView`,
  likely others) keyed by a per-view string literal (`"artist-releases"` here
  vs `"location"` in the location view) — do NOT extract this into a shared
  component during this split; it's flagged as a cross-file duplication
  pattern other agents splitting sibling view files should be aware of, but
  actually de-duplicating it changes behavior-coupling to `NavState` and is
  out of scope for a pure file-size split.
- The infinite-scroll trigger (`changed viewport-y => { ... if (-self.viewport-y
  + self.height >= self.viewport-height - 600px && ArtistReleasesState.has-more
  && !ArtistReleasesState.load-more-loading && !ArtistReleasesState.loading)
  { ArtistReleasesActions.load-more(); } }`) reads `flick`'s own geometry —
  keep this handler physically inside the `Flickable` block, do not move it
  into an extracted child (Slint callbacks on `changed` need direct access to
  the property they watch).
- `page.preferred-height` is referenced by `flick`'s `viewport-height:
  page.preferred-height;` binding — if the loading/error/empty states are
  extracted into a component, confirm `page`'s `preferred-height` still
  aggregates their heights correctly (Slint layouts compute preferred size
  from children automatically, so this should be transparent, but verify
  visually since a wrongly-sized child component could break scroll extent
  calculation).

## Verify after split
- Slint compile check (`slint-viewer` load or the project's build-time Slint
  compilation) with no unresolved-import warnings.
- Visual smoke test: open an artist page, click "See discography" for a
  release bucket, confirm the header (kicker/title/sort) still renders,
  changing sort still re-fetches/re-sorts, scrolling near the bottom still
  triggers `load-more`, and back-navigation still restores scroll position.
- Confirm the loading/error/empty conditional states still render exactly as
  before (each is `if` -gated on `ArtistReleasesState.loading`/`load-error`/
  `albums.length == 0` — these three must remain mutually exclusive after
  extraction).
