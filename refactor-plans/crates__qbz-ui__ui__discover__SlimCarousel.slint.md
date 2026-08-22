# crates/qbz-ui/ui/discover/SlimCarousel.slint (220 lines)

## Summary
A paginated "slim card" grid section (Recently Played Tracks, Popular albums):
4x3 grid capped at two pages, with prev/next nav buttons, an optional "View all"
link, optional title-adjacent list actions (queue/create-playlist), and an
optional refresh affordance — plus two small locally-defined helper components
(`NavButton`, `ViewAllLink`).

## Proposed split
Slint files split cleanly by "extract a sub-component to its own file, import
it back" — the two internal helper components are the obvious first cut, since
they are self-contained and reused conceptually across other carousels:

- `discover/SlimCarousel.slint` (~140 lines) — kept as the main file / import
  surface: the `export component SlimCarousel` definition, its properties,
  callbacks, and the header + Flickable/grid layout. Imports `NavButton` and
  `ViewAllLink` from their new files instead of defining them inline.
- `discover/carousel_nav_button.slint` (~45 lines) — the `NavButton` component
  (icon + enabled + hover background + click), exported so it can be reused by
  other carousel-style sections later (it is a generic small round icon
  button, not SlimCarousel-specific).
- `discover/carousel_view_all_link.slint` (~25 lines) — the `ViewAllLink`
  component (the "View all" header link), likewise generic enough to reuse.

If further trimming is wanted after that split (140 lines is already under
budget, so this alone suffices), the header row's list-actions/refresh/nav
blocks could additionally move into a `SlimCarouselHeader` sub-component, but
that is not required to clear 130 lines and would add an extra prop-plumbing
layer (callbacks for `view-all-clicked`, `list-action`, `refresh-clicked`, plus
the `flick` viewport reference for the chevrons) — only do this if the header
grows further.

## Re-export surface
`discover/SlimCarousel.slint` remains the single import path other `.slint`
files use today (`import { SlimCarousel } from "discover/SlimCarousel.slint";` or
similar relative path) — extracting `NavButton`/`ViewAllLink` to their own files
and `import`-ing them back does not change `SlimCarousel`'s own export name or
location.

## Coupling / watch out
- `NavButton` currently reads `Theme`, `ShellState`, `AppearanceState` for its
  background (the "app background active" alpha-blend logic) — these imports
  must move with it into `carousel_nav_button.slint`.
- `ViewAllLink` reads `Theme`, `Typography` — same, must move with it.
- The chevron `NavButton` instances inside `SlimCarousel` bind `enabled:` to
  `flick.viewport-x` (the Flickable's live scroll position) — that binding
  stays in `SlimCarousel.slint` (it's a property passed INTO `NavButton`, not
  something `NavButton` itself needs to know about), so no coupling issue.
- Both `NavButton` and `ViewAllLink` should be marked `export component` so
  Slint allows importing them from another file.

## Verify after split
- Run the Slint build/viewer check for `qbz-ui` (however this repo currently
  validates `.slint` files — e.g. `cargo build -p qbz-ui` if Slint files are
  compiled via `slint-build`, or `slint-viewer` on the file directly) to confirm
  no syntax/import errors.
- Smoke-test: open a Home view with "Recently Played Tracks" and "Popular
  albums" sections and confirm both still page correctly, the nav chevrons
  enable/disable at the right scroll boundaries, and "View all" still navigates.
