# crates/qbz-ui/ui/shell/sidebar/SidebarRow.slint

242 lines / budget 130. Analysis pass only — no code moved yet.

## 1. Why is this file long?

It is genuinely multi-responsibility, not irreducible. Four unrelated
concerns share one component today:

1. **Row visuals** — background/border, leading icon slot, name text,
   folder count, chevron, active accent bar (lines 99-174). Pure
   rendering; reads `entry`, `active`, `ta.has-hover`, `drop-hot`.
2. **Position-derived geometry** — the drag drop-target hit test
   (47-75) and the two mini-sidebar anchors, tooltip (77-97) and folder
   flyout (192-207). All three depend on `root.absolute-position -
   root.{x,y}`, the Flickable double-count correction documented in the
   file header. This is the part that is *actually* pinned to this
   element.
3. **Global side effects** — writing `DragState.over-playlist-id`,
   `SidebarTooltipState.*`, `SidebarFolderPopupState.*` and calling
   `SidebarActions.load-folder-popup`. Stateless once the corrected
   coordinates are handed in.
4. **Input + menu plumbing** — the `ta` TouchArea and the `menu`
   PopupWindow, plus 12 callback declarations that relay upward.

Concerns 1 and 3 are freely movable. Concern 2 is not, and the split
must keep it in this file. Two siblings already carved out this way
(`SidebarRowIcon`, `SidebarRowChevron`, `SidebarRowContextMenu`) — the
directory's house style is component-per-file with direct
`import { X } from "Y.slint"`, no barrel; `ColorPickerMath.slint` and
`state/discover_reco_genre.slint:TextUtil` set the precedent for a
stateless helper `global` living next to its only consumer.

## 2. Seams

### A. `SidebarRowContent.slint` — new, ~92 lines

Everything that paints the row. Exists so the row's *appearance* can be
read and changed without stepping through the absolute-position logic,
and so the icon/text/chevron/count/active-bar group has one owner.

Moves from the original:

| original lines | what |
| --- | --- |
| 99-103 | `use-collage` derived property (read only by the icon slot) |
| 110-116 | `border-radius`, `background`, `border-width`, `border-color` |
| 118-163 | the whole `HorizontalLayout` (icon, name, count, chevron) |
| 165-174 | the active-row 3px accent bar and its comment |

Public surface of the new component:

```
in property <SidebarEntry> entry;
in property <bool> is-folder;
in property <bool> active;
in property <bool> show-collage;
in property <bool> hovered;    // was `ta.has-hover`
in property <bool> drop-hot;   // was root's private `drop-hot`
callback toggle-folder();      // no id arg; the row re-adds entry.id
```

`use-collage` stays private inside the new file, recomputed from `entry`
+ `show-collage` — the row no longer needs it at all.

`hovered` is deliberately `in`, not `in-out`: content must never write
hover state back. `drop-hot` likewise stays owned by the row.

### B. `SidebarRowFx.slint` — new, ~66 lines

`export global SidebarRowFx`. Stateless functions, each taking
*already-corrected* window coordinates. Exists so the state-mutation
bodies (which are long and comment-heavy) leave the row while the
correction that produces the coordinates stays in the row.

| original lines | becomes |
| --- | --- |
| 63-66 | `pure public function point-in(px, py, x0, y0, w, h) -> bool` |
| 67-71 | `public function claim-drop(id: string, hot: bool)` |
| 79-97 | `public function tooltip(hovered: bool, entry: SidebarEntry, is-folder: bool, ax: length, ay: length)` |
| 187-188 | `public function drop-tooltip()` |
| 192-207 | `public function open-folder-flyout(entry: SidebarEntry, ax: length, ay: length)` |

Behaviour contracts that must be preserved verbatim, not paraphrased:

- `claim-drop`: set `DragState.over-playlist-id = id` when hot;
  clear it **only if it currently equals this id**. A naive
  `else { … = "" }` would let a leaving row erase the entering row's claim.
- `tooltip`: the `ShellState.sidebar-mini` guard applies to the *show*
  path only. On `hovered == false` it must always run the id-guarded hide
  (`if SidebarTooltipState.id == entry.id`), otherwise a mini→open
  transition can strand an open bubble.
- `open-folder-flyout`: keeps the `SidebarActions.load-folder-popup(id)`
  call before setting `open = true`, in that order.

Imports needed: `SidebarEntry`, `ShellState`, `SidebarActions`,
`DragState`, `SidebarTooltipState`, `SidebarFolderPopupState` from
`../../state.slint`.

### C. `SidebarRow.slint` — stays, ~127 lines

Verified by drafting the post-split file, not estimated: 127 lines with
the header comment cut from 10 to 6 and the surviving comments kept.

What remains, and why each item cannot move:

- The 3 `in` properties and 12 callbacks — the public surface.
- `is-folder`, `is-local-pl`, `folder-query` — read by `menu`, which
  cannot move (see hazards).
- Two new one-line helpers `pure function ox() -> length` /
  `oy() -> length` returning `root.absolute-position.{x,y} - root.{x,y}`.
  They exist so the correction is written once instead of four times;
  they are only ever *called from event handlers*, never bound.
- `drag-px/py/on` mirrors, `drop-hot`, `recompute-drop()` and the three
  `changed` handlers — the mirrors are what turn a declarative
  `DragState` read into an event-time recompute.
- `hidden-in-mini`, `height`, `visible` — layout, must stay on the
  element the parent `VerticalLayout` sees.
- `body := SidebarRowContent { width: 100%; height: 100%; … }`.
- `ta := TouchArea { … }` and `menu := SidebarRowContextMenu { … }`,
  in that declaration order, after `body`.

Draft of the trimmed file is at
`scratchpad/row.draft2` for the executing pass to start from.

## 3. Public surface

Unchanged. `SidebarRow.slint` keeps
`export component SidebarRow inherits Rectangle` at the same path with
the same properties and callbacks, so its only importer —
`crates/qbz-ui/ui/shell/sidebar/SidebarPlaylistList.slint:13` and its
instantiation at line 32 — needs no edit. (Grep over `ui/` finds no
other importer; the remaining `SidebarRow` hits are comments in
`SidebarRowIcon/Chevron/ContextMenu.slint` and
`state/sidebar_structs_popups.slint:59`.)

No barrel file is introduced: this directory does not use one, and
`SidebarRow` is still a real component here rather than a re-export.

Note for the executing pass: `callback delete-folder(string)` (line 28)
is declared and forwarded by `SidebarPlaylistList` but **never fired**
anywhere in `SidebarRow`. It is dead, and removing it would free a line —
but it is part of the public surface, so it stays unless removed as a
separate, deliberate change.

## 4. Slint hazards, and how the split handles each

**Element ids are file-scoped.** Two cross-references break if handled
naively:

- `ta.has-hover` is read at line 113 (background) and line 150 (folder
  count opacity). Both readers move into `SidebarRowContent`, where `ta`
  is not visible. Bridge: `in property <bool> hovered`, bound from the
  row as `hovered: ta.has-hover;`. This is legal because `body` and `ta`
  are siblings *in the row's own file*.
- `menu.show()` is called from `ta.pointer-event`. Both `ta` and `menu`
  therefore stay in the row file. `menu` additionally must remain a
  direct child of the row's root Rectangle: `SidebarRowContextMenu`
  positions itself as `x: anchor-width - 210px; y: 28px`, relative to its
  parent — its own header comment records this constraint.

**Two-way bindings.** `folder-query <=> root.folder-query` (line 234) is
the only `<=>` in the file and both ends stay put. The split introduces
no new one; `hovered` and `drop-hot` are one-way `in` on purpose.

**`root.` / `parent.` re-anchoring.** The active accent bar uses
`root.height` (lines 171) and centres on `root.height`. After the move,
`root` means `SidebarRowContent`. This is only equivalent because the
instantiation pins `width: 100%; height: 100%`. If that is omitted a
Rectangle child does **not** auto-fill its Rectangle parent in Slint, and
the bar silently collapses. The `parent.` uses at 119-124 are inside the
`HorizontalLayout` and move as a unit, so they are unaffected;
`SidebarRowIcon`'s internal `parent.` uses were already scoped to that
component.

**Private properties read by a moved block.** `use-collage` (private,
99-103) is read only at line 132 — it moves wholly into the content file
and is recomputed there. `is-folder` and `drop-hot` are private and are
read by moved code, so they become `in` properties on the content
component. `entry`, `active`, `show-collage` are already `in` on the row
and are passed through.

**PopupWindow / TouchArea / FocusScope placement.** The row currently
declares, in order: `HorizontalLayout` → active-bar `if` → `ta` → `menu`.
After the split it declares `body` → `ta` → `menu`. Same relative order,
so `ta` still sits above all painted content and hit-testing is
unchanged. The only other TouchArea in the subtree is
`SidebarRowChevron`'s `chevron-ta`, which is already one level deeper
than `ta` today and stays that way. There is no FocusScope in this file.
Moving `background` / `border-*` from the row's Rectangle to the content
Rectangle leaves the row transparent; identical output only while the
content fills the row exactly.

**A new exported global.** `SidebarRowFx` is reachable from the root
component through the import chain, so `slint-build` will generate a Rust
global for it. It is additive and holds no properties, but the name must
not collide with an existing global (checked: it does not).

## Risks

Things that compile cleanly and fail silently:

1. **The absolute-position correction losing its meaning.** If a later
   pass "simplifies" by moving `recompute-drop`, `ox()`/`oy()` or the
   flyout anchor into `SidebarRowContent`, the child's
   `absolute-position - x` is *not* the row's, because the child sits at
   `x: 0` inside the row while the row itself carries the Flickable
   offset. Result: drop targets and flyouts land one row-pitch off,
   drifting with scroll position — the exact bug the current header
   comment describes. The header comment must survive the split saying
   this.
2. **`pure` on `ox()`/`oy()`.** Reading `absolute-position` inside a
   function called from an event handler is fine, but the compiler may
   object to marking such a function `pure`, or (worse) a future
   declarative call site would turn it into a layout recursion. Fallback:
   drop `pure`; if the functions have to be abandoned entirely, the four
   inlined expressions cost ~4 lines and push the file to 131 — the
   escape hatch is trimming the surviving comments, not deleting them
   wholesale.
3. **Missing `width: 100%; height: 100%` on the content instance.** The
   row keeps its `height: 32px` / `0px` binding, so nothing errors; the
   content simply renders at its implicit size and the background, border
   and accent bar quietly stop covering the row.
4. **Hover no longer reaching the background.** If `hovered:
   ta.has-hover` is forgotten, the row keeps painting `transparent` on
   hover and only the `active` case still lights up — easy to miss
   because `active` rows look correct.
5. **`claim-drop` losing the id guard.** As noted above, converting the
   `else if (DragState.over-playlist-id == entry.id)` into a plain `else`
   makes one row's leave event cancel the next row's claim, so a drag
   sliding across rows intermittently drops onto nothing.
6. **Tooltip mini-guard placed on the wrong branch.** Folding show and
   hide into one `tooltip()` function makes it easy to put the
   `sidebar-mini` check around the whole body; the hide path must run
   unconditionally or a bubble can outlive a mini→open transition.
7. **A 3-line margin.** The row lands at 127/130. Any comment added later
   pushes it over. The next seam, if one is needed, is flattening the
   context-menu relay: `SidebarRowContextMenu` calls `SidebarActions.*`
   directly instead of bubbling through `SidebarRow` and
   `SidebarPlaylistList`, both of which are pure pass-throughs today
   (`SidebarPlaylistList.slint:36-44`). That frees ~12 lines here and ~6
   there — but it *does* change `SidebarRow`'s public callback surface
   and touches its importer, so it is a separate decision, not part of
   this split.
