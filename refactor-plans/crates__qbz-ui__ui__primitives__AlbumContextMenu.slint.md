# crates/qbz-ui/ui/primitives/AlbumContextMenu.slint (164 lines)

One-line summary: the album header "⋯" context menu (ported 1:1 from Tauri's AlbumMenu) — a flat list of `ContextMenuItem`s inside a `PopupWindow`, small overage.

## Proposed split
- `AlbumContextMenu.slint` (~60 lines) — **stays the public re-export** (`export component AlbumContextMenu inherits PopupWindow`). Keeps the `in property`s, `media-action` callback, sizing (`width`/`height` — note height is a literal formula tied to row count, must stay in sync with however many `ContextMenuItem`s the split-out content renders), and `close-policy`.
- `AlbumContextMenuItems.slint` (~110 lines, new, not exported) — the `ContextMenu { ... }` body: all 9 `ContextMenuItem`s + 4 separators (lines 54-162), including the pin/unpin, offline/refresh, and block/unblock conditional pairs. Takes `album-id`, `fully-cached`, `is-blocked`, `is-pinned` as properties and a `media-action`/`close()` callback (or just calls `root.close()` if kept as a direct child so `root` still resolves to the PopupWindow — verify Slint's `root` scoping when nesting a plain component inside a PopupWindow; may need an explicit `closed()` callback instead of `root.close()`).

## Tricky coupling to flag
- `root.close()` calls inside item handlers currently resolve to the enclosing `PopupWindow`'s close(); if the items move into a separate non-PopupWindow component, `root` there refers to that new component, not the popup — every `root.close()` must become a callback bubbling up to the real `PopupWindow.close()`. This is the one thing to get right; consider keeping the items inline and only extracting the icon/label string tables instead if this proves awkward.
- The height formula comment (`9 * 33px + 4 * 1px + 10px`) documents that swap-pairs (offline/refresh, block/unblock, pin/unpin) keep the row count constant — any future added row must update this constant, regardless of the split.

## Verify after split
- Compiles; every menu action (play next, queue, pin/unpin, add to playlist/mixtape, share Qobuz/Album.link, cache/recache, block/unblock) still fires and closes the popup; popup still uses `close-on-click-outside` (not the default) so a single click on a row is not swallowed by an early dismiss.
