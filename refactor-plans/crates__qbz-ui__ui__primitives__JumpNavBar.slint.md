# crates/qbz-ui/ui/primitives/JumpNavBar.slint (286 lines)

## Summary
Reusable sticky "JUMP TO" tab bar with an inline expanding search input, shown above
section-rich pages (artist/library); presentation-only component driven by callbacks
(`tab-clicked`, `search-clicked`, `search-text-edited`).

## Proposed split
Slint doesn't support splitting one component's body across files (a component must
be defined whole in one place), but this file already cleanly separates into two
visually/logically distinct sub-widgets that CAN be extracted as their own
components and composed back together — this is the idiomatic Slint way to shrink
an oversized component.

- `primitives/JumpNavBar.slint` (~110 lines) — top-level `JumpNavBar` component:
  the `JumpNavTab` struct, the outer `Rectangle`/`HorizontalLayout` scaffold
  (JUMP TO label, bottom border, outer search toggle button), instantiating the two
  extracted sub-components below in place of their current inline blocks.
- `primitives/jump-nav-bar/TabRow.slint` (~60 lines) — the tab-rendering
  `HorizontalLayout`/`for tab in root.tabs` block (currently lines 101-145):
  a new small component `JumpNavTabRow` taking `tabs`, `active-tab-id`,
  `search-open` (to disable clicks while searching) and re-emitting `tab-clicked`.
- `primitives/jump-nav-bar/SearchOverlay.slint` (~110 lines) — the animated
  `search-overlay` Rectangle (currently lines 151-248): a new component
  `JumpNavSearchOverlay` taking `search-open`, `search-text` (in-out), emitting
  `search-clicked`/`search-text-edited`, owning the `UiFocusState` wiring.

## Re-export surface
`primitives/JumpNavBar.slint` stays the single import surface — all existing
`import { JumpNavBar, JumpNavTab } from "primitives/JumpNavBar.slint";` call sites
are unaffected since the two new sub-components are only imported and composed
internally by `JumpNavBar.slint` itself, not exported separately (unless another
page later wants to reuse just the search overlay, which is easy to promote later).

## Coupling / watch out
- The `bar-bg` doc comment explicitly explains WHY `ShellState`/`state.slint` is not
  imported here (would create an import cycle since `state.slint` imports
  `JumpNavTab` from this file) — preserve `JumpNavTab` staying defined in the
  top-level `JumpNavBar.slint` file (not moved into a sub-file) so that cycle-avoidance
  note stays accurate.
- `search-input.focus()` on `changed search-open` must remain wired to the actual
  `TextInput` inside the extracted `SearchOverlay` component — when split, expose a
  public `focus-search()` callback/function on `JumpNavSearchOverlay` that
  `JumpNavBar` calls from its `changed search-open` handler, since Slint can't reach
  into a child component's private elements directly.
- `UiFocusState.text-input-focused` global-state write on `changed has-focus` must
  move with the `TextInput` into `SearchOverlay.slint`.
- Icon paths (`../assets/icons/search.svg`, `../assets/icons/x.svg`) are relative to
  `primitives/` — adjust to `../../assets/icons/...` in the new
  `primitives/jump-nav-bar/` subdirectory files.

## Verify after split
- `slint-viewer` (or the project's Slint compile check) on `JumpNavBar.slint` and any
  page importing it (artist page, library page).
- Visual smoke-test: tab click still scrolls, search icon still expands/collapses
  with the 200ms width animation, typing still fires `search-text-edited`, X button
  still clears/closes.
- Confirm no other `.slint` file imports `JumpNavTabRow` or `JumpNavSearchOverlay`
  directly (they should stay internal to JumpNavBar's composition).
