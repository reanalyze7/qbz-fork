# crates/qbz-ui/ui/shell/Sidebar.slint (1,082 lines)

## 1. Summary

The left navigation sidebar: playlist/folder list with drag-and-drop,
context menus, the section-nav rows (Discover/Library/Local
Library/MyQBZ), the playlists header toolbar (search/new/sort/collapse),
and a custom scrollbar — all in one file with four components
(`MenuItem`, `HeaderIconButton`, `SidebarRow`, `SidebarNavRow`) plus the
exported `Sidebar` root.

## 2. Proposed module split

By component/responsibility (each already a distinct, minimally-coupled
Slint `component`):

| New file | Owns | ~lines |
|---|---|---|
| `shell/sidebar/SidebarMenuItem.slint` | `MenuItem` component (generic context-menu row used by the sort/actions popup) | 60 |
| `shell/sidebar/SidebarHeaderIconButton.slint` | `HeaderIconButton` component (square toolbar icon button) | 30 |
| `shell/sidebar/SidebarRow.slint` | `SidebarRow` component — the single largest piece: folder/playlist row rendering, drag-drop target detection, tooltip, and its right-click context menu (folder/playlist actions + move-to-folder) | ~430 |
| `shell/sidebar/SidebarNavRow.slint` | `SidebarNavRow` component — section-nav row + its flyout-menu trigger logic | ~90 |
| `shell/sidebar/SidebarHeader.slint` | The "PLAYLISTS" header toolbar block (title/search input/search-toggle/new/sort button) plus the sort/actions `PopupWindow` — currently inlined in the `Sidebar` root's `VerticalLayout`; extract into its own component taking callbacks/bindings to `SidebarActions`/`SidebarState` (already globals, no new plumbing needed) | ~180 |
| `shell/sidebar/SidebarScrollbar.slint` | The custom thin auto-hide scrollbar (`pl-sb` + `thumb` + drag logic) as a reusable component parameterized over the `Flickable` it tracks | ~70 |
| `shell/Sidebar.slint` | Thin root: imports the above, lays out the section-nav block, `SidebarHeader`, the `Flickable`+`for entry in SidebarState.entries: SidebarRow` list wired to `SidebarScrollbar`, and the large-NPB dock space reservation | ~130 |

Total component count grows from 4 to 7, but every one of the 4 originals
maps 1:1 onto a file above (`SidebarRow` is the only one still borderline
large at ~430 lines because its right-click context menu, drop-target
math, and tooltip logic are all tightly bound to the same `root.entry`
state — see §4 for how to shrink it further).

### Further splitting `SidebarRow` (currently ~430 lines, still over budget)

`SidebarRow` itself should split into:
- `SidebarRow.slint` — the row shell: icon slot, label, chevron, active
  bar, `TouchArea` click/hover wiring (~180 lines).
- `SidebarRowContextMenu.slint` — the `menu := PopupWindow { ContextMenu
  { ... } }` block (folder menu / playlist menu / move-to-folder search
  + list), taking the row's `entry`, callbacks, and `folder-query` as
  in/out properties (~180 lines).
- `SidebarRowDrag.slint` — not a visual component but worth extracting as
  shared *functions* (`recompute-drop`, the `drag-px/py/on` mirroring) into
  a small mixin-like component wrapped around drop-target rows, since the
  exact same absolute-position-correction pattern is duplicated in
  `SidebarFolderPopupState` anchoring elsewhere in the codebase (grep for
  "absolute-position double-counts" — at least one other file has the
  identical comment/pattern). Alternatively keep as inline functions in
  `SidebarRow.slint` if extraction proves awkward in Slint (functions
  aren't first-class exportable the way components are) — call this out
  as a judgment call for whoever does the actual split.

## 3. Re-export / public API surface

`Sidebar.slint` keeps exporting `component Sidebar` — the only symbol any
other file imports (`import { Sidebar } from "shell/Sidebar.slint";` in
the app shell). None of the new sub-files need to be exported outside the
`shell/sidebar/` directory; `Sidebar.slint` imports them locally
(non-exported components are fine as long as they're `import`ed, not
`export`ed, from the sub-files — match the existing convention used
elsewhere in `ui/primitives/`).

## 4. Tricky coupling to watch out for

- `SidebarRow` reads/writes a large set of globals directly:
  `ShellState`, `DragState`, `SidebarTooltipState`, `SidebarFolderPopupState`,
  `SidebarActions`, `SidebarState`. Splitting the context menu out means
  passing `entry`, `folder-query` (in-out), and the six callbacks
  (`open-playlist`, `toggle-folder`, `move-playlist`, `delete-folder`,
  `edit-playlist`, `edit-folder`, `hide-playlist`, `hide-folder`,
  `add-to-mixtape`) through as component properties/callbacks rather than
  each sub-component re-touching the globals independently — keeps a
  single source of truth for what the row does on each action.
- The **absolute-position double-count correction** (`root.absolute-position.x
  - root.x`) appears three times in this file alone (tooltip anchor,
  folder-flyout anchor, drop-target math) with near-identical comments
  explaining the same Flickable quirk. When splitting, hoist this into one
  shared `function` (or note it can't be shared across components in Slint
  and must be copy-pasted with a comment pointing at the canonical
  explanation) rather than three near-duplicate blocks.
- `pl-sb`/`thumb`/`pl-flick` — the scrollbar directly reads `pl-flick`'s
  `viewport-height`/`viewport-y`/`height`. Extracting `SidebarScrollbar`
  requires either passing the `Flickable` by reference (not directly
  possible in Slint) or exposing the handful of properties it needs
  (`viewport-height`, `viewport-y`, `height`) as two-way bindings —
  design this interface carefully before moving code.
- `SidebarNavRow`'s `open-menu()` function publishes into the **global**
  `HeaderMenuState` (shared with the header's own nav-tab dropdowns) —
  this is intentional (same flyout mechanism, different anchor), not a
  bug to fix during the split.

## 5. What to verify after the real split

- `cargo build` / Slint compile check on `qbz-ui` succeeds with the new
  file layout.
- Visual smoke test: sidebar open/mini/closed states, playlist
  drag-to-folder, right-click context menus (folder and playlist), the
  sort/actions popup, and search — all still behave identically (this
  file has a lot of hand-tuned pixel-level comments; regressions would be
  visual, not compile errors).
- Confirm `import { Sidebar } from "shell/Sidebar.slint";` in the app
  shell file is unchanged and still resolves.
- Check no other `.slint` file imports the four inner components
  (`MenuItem`, `HeaderIconButton`, `SidebarRow`, `SidebarNavRow`) directly
  by name from `Sidebar.slint` — `grep -rn 'from "shell/Sidebar.slint"'`
  and `grep -rn 'from "../Sidebar.slint"'` across `ui/` to be sure only
  `Sidebar` itself is consumed externally.
