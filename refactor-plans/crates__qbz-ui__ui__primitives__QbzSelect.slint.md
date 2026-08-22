# crates/qbz-ui/ui/primitives/QbzSelect.slint (327 lines)

One-line summary: the dropdown-select primitive (collapsed control + searchable/grouped popup list) — badges/headers, the control, and the open-list popup are all in one file.

## Proposed split
- `QbzSelect.slint` (~150 lines) — **stays the public re-export surface** (`export component QbzSelect`). Keeps the collapsed control (HorizontalLayout with current value + badge + chevron), the TouchArea/FocusScope activation, and the focus-ring overlay. Imports the two files below.
- `QbzSelectBadges.slint` (~35 lines, new, not exported) — `BpBadge`, `AudioGlyph`, `SectionHeader` private components (lines 26-55).
- `QbzSelectPopup.slint` (~140 lines, new, not exported or exported as internal helper) — the `PopupWindow` body: search box (lines 218-255) + the Flickable/rows-col option list (257-322). Takes `options`, `badges`, `groups`, `current-index`, `filter` and a `selected(int)` callback as properties so `QbzSelect` just instantiates it.

## Tricky coupling to flag
- `filter` is a private property on `QbzSelect` bound two-way into the popup's search `TextInput` — needs to become an in-out property passed into `QbzSelectPopup`.
- Row height math (`row-height`, `header-height`, `search-height`, `max-list-height`) is shared between the popup's height calc (on `QbzSelect`) and the internal Flickable — must stay consistent across the two files (pass as properties, don't duplicate constants).
- `TextUtil.contains-ci` filtering logic must move with the popup.

## Verify after split
- `slint-viewer`/build compiles (no missing imports).
- Every caller of `QbzSelect` (Output-device picker with `searchable`/`badges`, plain settings selects) still renders identically — no prop renamed.
- Manual check: opening a searchable list still filters by visibility only (index stability for `selected(i)`).
