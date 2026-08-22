# crates/qbz-ui/ui/shell/SidebarFolderPopup.slint (146 lines)

## Summary
Collapsed-sidebar folder flyout popup: shows a folder's playlists (max 4 rows
visible, scrollbar beyond that) anchored to the right of the mini sidebar,
rendered at the AppShell level since the sidebar clips its own overflow.
Contains one small sub-component (`FpRow`, a playlist row) plus the outer
`SidebarFolderPopup` scrim/panel/header/list.

## Proposed split
Only 16 lines over budget — this is a borderline case. Extract the already
self-contained `FpRow` sub-component into its own file (the natural, minimal
split) rather than restructuring the outer component's layout.

- `shell/SidebarFolderPopup.slint` (~110 lines) — the outer
  `SidebarFolderPopup` component: the scrim `TouchArea`, the anchored panel
  `Rectangle`, header (folder name + icon), the `Flickable`/`for entry in
  ...: FpRow` list, and the `ListScrollbar`. Imports `FpRow` from the new file
  in place of the current inline definition.
- `shell/sidebar-folder-popup/FpRow.slint` (~40 lines) — the `FpRow` component
  (one playlist row: icon + elided name, hover background, click dispatch to
  `SidebarActions.open-playlist`), exported so the parent can import it.

If this feels like overkill for a 16-line overage, an equally valid call is to
leave this file as-is and prioritize the larger files in this batch — flag
that judgment call to whoever does the real split pass (the 130-line rule is
a target, not a hard gate that must fire on every single-digit overage; note
this explicitly so it isn't silently "fixed" in a way that fragments a
genuinely small, cohesive file for no real readability gain).

## Re-export surface
`shell/SidebarFolderPopup.slint` stays the single import surface —
`shell/AppShell.slint`'s `import { SidebarFolderPopup } from
"SidebarFolderPopup.slint";` is unaffected. `FpRow` is imported and used only
inside `SidebarFolderPopup.slint` itself, not re-exported.

## Coupling / watch out
- `FpRow`'s `clicked` callback currently inlines
  `SidebarActions.open-playlist(entry.id); SidebarFolderPopupState.open =
  false;` in the PARENT's `for entry in ...: FpRow { clicked => { ... } }`
  block, not inside `FpRow` itself — this stays exactly the same after
  extraction (FpRow only emits `clicked()`, the parent's `for` loop still
  wires the action), so no logic moves, just the component definition.
- `root.row-h`/`root.visible-rows`/`root.list-h` properties on the outer
  component drive both the panel's `height` calculation AND the Flickable's
  `viewport-height` — these stay on the outer component, not something `FpRow`
  needs.

## Verify after split
- Slint compile check on `SidebarFolderPopup.slint` and its importer
  `AppShell.slint`.
- Manual smoke-test: collapse the sidebar, click a folder, verify the popup
  opens at the right anchor point, shows up to 4 rows with a scrollbar beyond
  that, clicking a playlist row navigates and closes the popup, clicking
  outside the panel (the scrim) also closes it.
