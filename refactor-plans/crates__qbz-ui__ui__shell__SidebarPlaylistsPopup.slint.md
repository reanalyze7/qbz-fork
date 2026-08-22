# crates/qbz-ui/ui/shell/SidebarPlaylistsPopup.slint (275 lines)

## Summary
The closed-sidebar "playlists flyout" popup: a search header, a scrollable
6-row-visible list of folder/playlist entries (`PpRow`), and a footer with
Import + "Manage playlists" links — rendered at the AppShell level so it can
overflow the (clipping) header.

## Proposed split
Just barely over budget (275 vs 130 — needs a real split, not a trim). Split
along the same component-boundary lines already implicit in the file: the row
component, the scrim+panel shell, and the footer actions.

- `shell/SidebarPlaylistsRow.slint` (~60 lines) — the `PpRow` component
  (18-75): one folder/playlist row, fully self-contained (`in property
  <SidebarEntry> entry`, its own `TouchArea`). Extract verbatim.
- `shell/SidebarPlaylistsPopupFooter.slint` (~90 lines) — the Import row
  (194-234) and "Manage playlists" row (236-271) as one small composed
  component, e.g. `component SidebarPlaylistsFooter inherits Rectangle`
  exposing two callbacks (`import-clicked`, `manage-clicked`) so the parent
  still owns the `SidebarPlaylistsPopupState.open = false` +
  `SidebarState.search-query = ""` + `SidebarActions.search-changed("")`
  close-and-clear sequence (that 3-line sequence repeats 3x in the original —
  a good place to also add a shared `close-popup-and-clear-filter()` callback
  on the footer component instead of duplicating it three times, but that's a
  behavior-preserving simplification to apply carefully during the real split,
  not required just to hit the line budget).
- `shell/SidebarPlaylistsPopup.slint` (~130 lines) — becomes the composing
  shell: the scrim `TouchArea` (85-91), the anchored panel `Rectangle` sizing
  math (93-99), the search header with its focus-timer workaround (125-151),
  the entries `Flickable` + `ListScrollbar` + empty-state text (157-187), and
  instantiates `SidebarPlaylistsRow { entry: entry; }` in the `for` loop plus
  `SidebarPlaylistsFooter { import-clicked => {...} manage-clicked => {...} }`
  at the bottom. This remains the file other `.slint` files import.

## Re-export surface
`SidebarPlaylistsPopup.slint` stays the single import surface — `export
component SidebarPlaylistsPopup` keeps its name and zero external
callbacks/properties (it reads global state directly:
`SidebarPlaylistsPopupState`, `SidebarState`, `SidebarActions`, `OfflineState`,
`UiFocusState`), so the AppShell-level instantiation needs no changes.
`SidebarPlaylistsRow` and `SidebarPlaylistsFooter` are new internal
components — only `SidebarPlaylistsPopup.slint` needs to `import` them.

## Coupling / watch out
- The height calculation at the top (`list-h`, and the panel's `height: 34px +
  root.list-h + 1px + 40px + 40px + 8px` comment "search(34) + list + divider(1)
  + import(40) + manage(40) + paddings(8)") hard-codes the Import/Manage row
  heights (40px each) inline in the parent's size math — if `40px` ever changes
  inside the new `SidebarPlaylistsFooter` component, this comment/formula in
  the parent goes stale silently. Consider exposing the footer's total height
  as a computed property on `SidebarPlaylistsFooter` (e.g. `out property
  <length> total-height: 81px;`) that the parent references instead of a
  hard-coded literal, to keep the two in sync — flag this even if not fixed in
  this pass.
- `pp-search`'s focus workaround (a 1ms `Timer` calling `.focus()`, because
  focusing during mount/layout panics with "Recursion detected" per the
  code comment referencing i-slint-core) must stay colocated with the
  `LineEdit` itself in the parent shell — do not extract the search header into
  a separate component without carrying this workaround and its comment.
- `pp-flick` (the entries Flickable) is referenced by ID from the sibling `if
  SidebarState.entries.length > 6: ListScrollbar` block for `viewport-height`/
  `visible-height`/`viewport-y` — both must stay in the SAME file (the parent
  shell), since Slint ids aren't visible across component/file boundaries
  the way they are across sibling `if` blocks in one file.
- `SidebarState.search-query` and `SidebarActions.search-changed(...)` are the
  SAME filter mechanism the full (open) sidebar uses (per the file's own
  header doc comment) — clearing it on popup-close ("so it doesn't silently
  scope the expanded sidebar's list later") is a cross-component UX contract;
  preserve the clear-on-close call in all three close paths (row click, scrim
  click, footer Import/Manage click) even after moving code around.

## Verify after split
- Confirm the Slint build step compiles (however this workspace invokes
  `slint-build` / `slint-compiler` — check `qbz-ui`'s build.rs or the
  consuming crate's build script).
- Visual/smoke check: open the closed-sidebar flyout, type in the search box
  (confirm live filtering + it clears on close), click a folder (expand/
  collapse in place), click a playlist (opens it + popup closes + filter
  clears), click Import and Manage playlists (both close the popup + clear the
  filter), and confirm the list scrollbar appears only when >6 entries.
- `grep -rn "SidebarPlaylistsPopup {" crates/qbz-ui` to confirm the single
  instantiation site (AppShell-level) needs no changes.
