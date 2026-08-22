# crates/qbz-ui/ui/discover/Spotlight.slint (180 lines)

## Summary
Discover > For You "Spotlight" widget (port of Tauri's SpotlightLite): a header, a
hero block (circular portrait + artist name + play-top-tracks/open-artist circle
buttons), and a draggable Flickable row mixing a Top-Tracks `RadioCard` with the
artist's `AlbumCard`s. Defines one local component (`CircleButton`).

## Proposed split
Only ~50 lines over budget — extract the two visually distinct blocks (hero, content
row) into small sub-components; the local `CircleButton` helper is small enough to
keep local to whichever file uses it, or promote alongside the hero.

- `discover/Spotlight.slint` (~55 lines) — top-level `Spotlight` component: the four
  re-emitted callbacks, the `content-width`/`top-tracks-width`/`album-count`
  properties (needed by the content row, kept at the top level since they derive
  from `ForYouState` directly and are simple), the header block, and instantiation
  of `SpotlightHero` and `SpotlightContentRow` below.
- `discover/spotlight/CircleButton.slint` (~25 lines) — the `CircleButton` local
  component, promoted so both the hero (play/open-artist buttons) can use it.
- `discover/spotlight/SpotlightHero.slint` (~65 lines) — the hero
  `HorizontalLayout`: circular portrait Image/placeholder + touch-area, and the
  name/ARTIST-label/circle-buttons `VerticalLayout`, emitting `open-artist` and
  `play-top-tracks` up to the parent.
- `discover/spotlight/SpotlightContentRow.slint` (~55 lines) — the draggable
  Flickable row (Top Tracks `RadioCard` + `for album in ... AlbumCard`), taking
  `content-width` as an `in property` from the parent and re-emitting
  `open-album`/`play-top-tracks`/`media-action`.

## Re-export surface
`discover/Spotlight.slint` stays the single import surface — the
`export component Spotlight` signature (4 callbacks) is unchanged; none of the three
extracted sub-components are exported/imported elsewhere (unless `CircleButton` is
later found reusable by another Discover card, in which case it can be promoted
further without breaking this file).

## Coupling / watch out
- `content-width`/`top-tracks-width`/`album-count` properties are read by the
  Flickable's `viewport-width` inside the content row — when extracted into
  `SpotlightContentRow.slint`, these must become `in property`s passed down from the
  top-level `Spotlight` component rather than re-declared (they derive from
  `ForYouState.spotlight-has-top-tracks`/`.spotlight-albums.length`, which should
  stay read once at the top level to avoid duplicating the derivation logic).
- `ForYouState.spotlight-artist-id` is read by BOTH the hero (portrait click,
  circle-button clicks) and the content row (RadioCard clicks) — no shared local
  state, just two independent reads of the same global, so no extra plumbing needed
  beyond each sub-component importing `ForYouState` itself.
- `CircleButton`'s `primary` variant styling (accent background vs elevated surface)
  is only used by the hero's play-button — keep `CircleButton` importable by
  `SpotlightHero.slint` only; no other file in this batch needs it.

## Verify after split
- Slint compile check on `Spotlight.slint` and its importer (the For You / Discover
  page).
- Visual smoke-test: hero portrait click and circle buttons still call
  `open-artist`/`play-top-tracks`; the content row still drags/scrolls
  horizontally, the Top Tracks card only appears when
  `spotlight-has-top-tracks` is true, and album cards still fire `open-album` /
  `media-action`.
