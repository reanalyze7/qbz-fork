# `crates/qbz-ui/ui/shell/HeaderBar.slint` (1192 lines)

The largest file in this batch. Top header bar: custom window-chrome drag surface,
centered search field with cortinilla keyboard handling, left nav-tab cluster (full +
compact variants, each duplicated for a recommendations-on/off toggle), right controls
(offline status badge + hamburger menu), and the hamburger's dropdown menu. Eight
internal helper components already exist (`NavTab`, `MenuItem`, `NavTabWithMenu`,
`CompactNavBtn`, `CompactPlaylistsBtn`, `HeaderNavBtn`, `HeaderIconButton`,
`OfflineStatusBadge`) plus the `export component HeaderBar` itself.

## Proposed split

- `HeaderBar.slint` (~120 lines) — stays the public surface: `export component
  HeaderBar`, the chrome-drag/window-controls block, top-level responsive properties
  (`show-tab-icons`, `search-width`, `chrome-*`), and composes the extracted blocks below.
- `shell/HeaderSearch.slint` (~180 lines) — the entire `search-scope` `FocusScope` block
  (lines ~631-792): cortinilla keyboard handling (`on-enter`, arrow/Enter/Escape
  interception), the search `TextInput`, placeholder, and Enter-hint. This is the single
  most complex, self-contained chunk — extract as one component taking `search-width` as
  a property (or reading it back from a shared global if simpler) and forwarding
  `SearchActions`/`SearchState` (already globals, no threading needed beyond width).
- `shell/HeaderNavButtons.slint` (~40 lines) — small helper components `HeaderNavBtn` +
  `HeaderIconButton` (lines 285-349), used only by the left/right control clusters.
- `shell/HeaderNavTabs.slint` (~90 lines) — `NavTab` + `NavTabWithMenu` (lines 23-196).
- `shell/HeaderCompactNav.slint` (~80 lines) — `CompactNavBtn` + `CompactPlaylistsBtn`
  (lines 201-281).
- `shell/HeaderLeftControls.slint` (~230 lines) — the entire `left-controls`
  `HorizontalLayout` (lines ~797-1023): history-nav buttons + the full section-nav tab
  row + the compact section-nav row. This is the largest remaining chunk and itself
  contains a LOT of near-duplicated `if`-gated blocks (recommendations on/off, full vs
  compact) — flag as a simplification opportunity (see below), not just a line-count fix.
- `shell/OfflineStatusBadge.slint` (~155 lines) — extract the existing `OfflineStatusBadge`
  component (lines 359-512) verbatim into its own file.
- `shell/HeaderMenu.slint` (~140 lines) — extract `MenuItem` (lines 86-122) plus the
  `menu := PopupWindow { ... }` hamburger dropdown content (lines 1056-1191) into one file,
  taking `logout`/`close-app` callbacks forwarded from `HeaderBar`.
- `right-controls` (lines 1026-1054, ~30 lines) can stay inline in `HeaderBar.slint` or
  move into `HeaderMenu.slint` alongside the button that opens it.

## Coupling to flag

- **Duplication smell**: the full nav-tab row and the compact nav-tab row (lines
  826-1022) each have TWO near-identical `if` branches for Discover
  (`show-recommendations` on/off) that differ only by one extra `NavMenuEntry` in the
  `items` array — this pattern repeats 4 times total (full+compact × on+off). The file's
  own comment explains why ("Slint can't infer `[NavMenuEntry]` from a ternary of array
  literals"), so this may be a genuine Slint limitation rather than sloppiness — but it's
  worth flagging for whoever does the real split: a small helper function/computed
  property building the items array in Rust (or a `for`-based conditional append) could
  collapse 4 near-duplicate blocks into 2 (or 1).
- `HeaderMenuState`/`SidebarPlaylistsPopupState` globals are written directly by
  `NavTabWithMenu`/`CompactNavBtn`/`CompactPlaylistsBtn` to publish menu identity — this
  is the same mechanism `HeaderMenuOverlay.slint` (also in this gap-fill batch) reads from;
  the two files are tightly coupled through that global and must stay in sync if either
  is refactored.
- `cache-rendering-hint: true` on the root `HeaderBar` (perf note: femtovg repaint cost,
  see spec 2026-07-19-cpu-idle-repaint-617 §9.2) — keep this on the actual root component
  after the split, not accidentally dropped or moved to a sub-component where it wouldn't
  have the same effect.
- The custom window-chrome drag surface, traffic-light inset math
  (`chrome-left-inset`/`wc-cluster-width`), and `right-controls`'s x-position all
  cross-reference `root.chrome-controls`/`root.wc-on-left` — keep these together in the
  parent `HeaderBar.slint`, not split across files.
- `search-scope.clear-focus()` is called from `HeaderBar`'s `changed nav-view-probe`
  handler — if `search-scope` moves into `HeaderSearch.slint`, this focus-release-on-
  navigate hygiene logic needs a forwarded function/callback so `HeaderBar` can still
  clear it on every page change.

## Verify after split

- Slint compile check.
- Visual + interaction smoke test: window drag/double-click-maximize, search field
  (cortinilla open/close, arrow-key navigation, Enter to submit/activate, focus guard for
  hotkeys), full and compact nav tabs (Discover with/without Recommendations, Library,
  Local Library, My QBZ dropdowns), offline status badge (all 3 states + flyout actions),
  hamburger menu (all items + logout/close), responsive breakpoints (tab icons hide,
  search narrows).
