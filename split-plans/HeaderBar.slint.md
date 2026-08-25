# crates/qbz-ui/ui/shell/HeaderBar.slint — split plan

**Current: 147 lines. Budget: 130. Over by 17.**

## 1. Why is this file long?

It is not one irreducible declaration, but it is already close to minimal:
the search field, the left cluster, the right cluster and the app menu were
all extracted in earlier passes (see the `Split out of HeaderBar.slint`
header comments in `HeaderSearch.slint`, `HeaderLeftControls.slint`,
`HeaderRightControls.slint`, `HeaderMenu.slint`, `HeaderNavTabs.slint`,
`HeaderCompactNav*.slint`, `HeaderFullNavRow.slint`,
`OfflineStatusBadge.slint`). What is left is a composer plus exactly one
block that is still real markup rather than composition: the custom window
chrome (the header IS the title bar). That block is 37 lines — the drag
`TouchArea` with its Wayland latch, and the two conditional `WindowControls`
placements. Everything else in the file is either a property definition, a
one-line child instantiation, or the comment explaining it.

So the file is long because one responsibility (window-chrome *rendering*)
never got its own file, even though the window-chrome *math* is deliberately
kept on the root because two different children consume it.

## 2. The seam

Extract lines **82–118** into a new sibling file.

| Lines | Content | Moves? |
|---|---|---|
| 1–22 | header comment + imports | stays (edited) |
| 23–45 | callbacks, `cache-rendering-hint`, height/background | stays |
| 47–52 | responsive breakpoints (`show-tab-icons`, `search-width`) | stays |
| 54–62 | `nav-view-probe` + `changed` handler | stays |
| 64–80 | chrome math (`chrome-drag-enabled`, `chrome-controls`, `wc-on-left`, `wc-cluster-width`, `chrome-left-inset`) | **stays** |
| 82–107 | drag-surface comment + `if root.chrome-drag-enabled: TouchArea { … }` | **moves** |
| 109–118 | drawn `WindowControls` comment + the two `if` placements | **moves** |
| 120–125 | absolutely centred `header-search := HeaderSearch` | stays |
| 127–146 | left/right control clusters | stays |

### New file

**`crates/qbz-ui/ui/shell/HeaderWindowChrome.slint` — est. 62 lines**

```
// header comment (~10 L)
import { WindowControlActions } from "../state.slint";
import { WindowControls } from "WindowControls.slint";

export component HeaderWindowChrome inherits Rectangle {
    in property <bool> drag-enabled: false;
    in property <bool> controls: false;
    in property <bool> wc-on-left: false;
    in property <length> wc-cluster-width;
    // …lines 82–118 verbatim, with root.chrome-drag-enabled →
    // root.drag-enabled and root.chrome-controls → root.controls…
}
```

The `if` guards move *with* the block rather than being lifted to the call
site, so the `TouchArea` and the `WindowControls` are still conditionally
instantiated exactly as today; the outer instance itself is unconditional
and costs one empty `Rectangle`.

### HeaderBar after the split

Lines 82–118 are replaced by:

```
    // Custom window chrome (the header IS the title bar): drag surface +
    // drawn min/max/close cluster. Declared FIRST so every interactive
    // element below wins hit-testing. The chrome math stays on the root —
    // HeaderRightControls reads it too (see the header comment).
    HeaderWindowChrome {
        width: root.width;
        height: root.height;
        drag-enabled: root.chrome-drag-enabled;
        controls: root.chrome-controls;
        wc-on-left: root.wc-on-left;
        wc-cluster-width: root.wc-cluster-width;
    }
```

Import churn: `+ import { HeaderWindowChrome } from "HeaderWindowChrome.slint";`,
`- import { WindowControls } from "WindowControls.slint";`, and
`WindowControlActions` drops out of the `../state.slint` import list (line 17)
because its only two uses (lines 93, 104) move. While editing that line,
`OfflineState` can also go — it is already unused in the body today.

**Estimated result: 147 − 37 + 12 = 122 lines.** Eight lines of headroom.

## 3. Public surface

Unchanged. `HeaderBar` keeps its path, its name, and its four callbacks
(`logout`, `close-app`, `recovery-login`, `navigate`). The only importer is
`crates/qbz-ui/ui/shell/AppShell.slint:15` (instantiated at line 85, binding
`logout` / `close-app` / `recovery-login` / `navigate`); it does not change.
Everything else that mentions `HeaderBar` in the tree is a comment. No Rust
touches it — `qbz-ui` exports only the window type; the grep hits in
`crates/qbz/src/**` are comments about header menu routes.

`HeaderWindowChrome` is a new export used by one caller. No re-export shim is
needed, because nothing outside HeaderBar ever referenced the moved markup.

## 4. Alternatives rejected

- **Move the centred search.** Nothing to move: `HeaderSearch` is already a
  separate component, and lines 121–125 are just its absolute-centering
  bindings, which must live at the call site (they resolve `root.width` /
  `root.height` of the header). Saves 5 lines at best, not 17.
- **Move the responsive breakpoints (47–52) to a helper or a global.**
  `show-tab-icons` and `search-width` are derived from `root.width`, which is
  only known at the header root, and they are consumed by two *different*
  children. Turning them into a global would make them window-wide mutable
  state for a 6-line saving.
- **Move the chrome math (64–80) along with the markup.** `chrome-controls`,
  `wc-on-left` and `wc-cluster-width` are read by the `HeaderRightControls`
  x-binding (lines 136–141) and by its four in-property bindings, and
  `chrome-left-inset` is read by `HeaderLeftControls` (line 129). Moving them
  means either duplicating the derivations or adding `out` properties and
  reading them back through the child instance — more coupling, not less.
  This is what the existing header comment means by "must stay together".

## 5. Bridging properties (exact list)

All one-way `in` properties on `HeaderWindowChrome`, all fed from `root.` in
HeaderBar. None are two-way; nothing writes back.

| New property | Type | Fed from | Read at (old line) |
|---|---|---|---|
| `drag-enabled` | `bool` | `root.chrome-drag-enabled` (69–70) | 87 |
| `controls` | `bool` | `root.chrome-controls` (71–73) | 111, 115 |
| `wc-on-left` | `bool` | `root.wc-on-left` (74) | 111, 115 |
| `wc-cluster-width` | `length` | `root.wc-cluster-width` (76) | not read by the moved block — see Risks |
| `width` / `height` | `length` | `root.width` / `root.height` | 112, 113, 117 |

`wc-cluster-width` is listed because the drawn-controls placement *looks* like
it should use it, but line 112 sizes off `self.preferred-width` instead. Do
**not** add the property speculatively: if the moved block does not reference
it, leave it out and keep it on the HeaderBar root only (where
`HeaderRightControls` genuinely consumes it). Verify at edit time and drop the
row if unused — the table above is the audit, not a requirement.

## 6. Cross-references checked

- **Element ids.** The file declares exactly one id, `header-search`
  (line 121), referenced from the `changed nav-view-probe` handler (line 61).
  Both stay in HeaderBar. The moved block declares **no** ids: the drag
  `TouchArea` refers to itself only via `self` (lines 92, 95–104) and the two
  `WindowControls` instances via `self.preferred-width` /
  `self.preferred-height`. So the move breaks no id reference in either
  direction.
- **`parent.`** No `parent.` appears anywhere in the file.
- **`root.`** in the moved block: `root.chrome-drag-enabled`,
  `root.chrome-controls`, `root.wc-on-left`, `root.width`, `root.height`.
  After the move, `root` means the *new* component — see Risks.
- **Two-way (`<=>`) bindings.** None in the file.
- **Globals.** `WindowControlActions` is used only inside the moved block
  (93, 104) and must be imported in the new file; `WindowControls.slint`
  imports it separately already.
- **Absolutely centred search.** Lines 121–125 are untouched by this split.
  `header-search.x = (root.width - self.width) / 2` still resolves against the
  HeaderBar root, so the field stays centred on the window regardless of the
  left/right cluster widths. The new chrome component is a sibling of the
  search, not an ancestor, so it cannot introduce a layout parent.
- **Breakpoints.** `show-tab-icons` (48) and `search-width` (51–52) are read
  at lines 131 and 124 respectively — both remain in HeaderBar. No breakpoint
  value crosses the new boundary, so this split adds **zero** breakpoint
  bridging properties.

## 7. Risks

1. **A user-defined component instance does not fill its parent.** Inside a
   plain `Rectangle`, a built-in `TouchArea` defaults to the parent's size,
   which is why line 87 works today with no `width`/`height`. A
   `HeaderWindowChrome { }` instance defaults to its *preferred* size instead.
   If `width: root.width; height: root.height;` is omitted at the call site,
   the drag surface silently collapses (window drag and double-click-maximize
   stop working) and the right-anchored `WindowControls` land at
   `x = 0 - preferred-width - 8px`, i.e. off-screen. This is the single most
   likely way to get a compiling but broken header. Smoke-test: drag an empty
   part of the header, double-click it, and check the min/max/close cluster on
   both `wc-position-index` settings.
2. **`root.width` / `root.height` rebind.** Lines 112–117 use them for the
   cluster placement. After the move they refer to the new component, which is
   only correct because of risk 1's bindings. The two are coupled: fixing one
   without the other gives a wrong-but-plausible position.
3. **Declaration order is load-bearing.** The comment at 82–86 states the drag
   surface must be declared first so later interactive elements win
   hit-testing. The new instance must therefore stay in the same position in
   HeaderBar's child list — first, before `header-search`. Moving it below the
   clusters would make the whole header un-clickable.
4. **Z-order inside the new component.** The drawn `WindowControls` are
   declared after the drag `TouchArea` within the moved block, so they keep
   winning hit-testing over it. Preserve that order verbatim; do not reorder
   the two `if` blocks while renaming the properties.
5. **Property rename typos.** `chrome-drag-enabled` → `drag-enabled` and
   `chrome-controls` → `controls` happen at four use sites (87, 111 ×1, 115
   ×1). A missed rename does not fail to compile if the old name accidentally
   matches nothing — it fails with an unknown-property error, which is the
   good case; the bad case is renaming only the guard and leaving the call
   site bound to a stale name. Alternatively keep the original names on the
   new component and accept the slightly redundant `chrome-` prefix — that is
   the lower-risk option and matches `HeaderRightControls`, which kept
   `chrome-controls` / `wc-on-left` verbatim. **Prefer keeping the names.**
6. **`cache-rendering-hint`.** Stays on the HeaderBar root (line 40, spec
   2026-07-19-cpu-idle-repaint-617 §9.2). Do not move it to, or duplicate it
   on, the new component: a nested cached subtree inside a cached subtree is
   wasted texture memory, and the hint on the root already covers the chrome.
7. **Headroom.** 122 lines leaves 8. HeaderBar is a composer, so the next
   feature that adds a child cluster will re-breach the budget. The next seam
   after this one is the chrome math block (64–80, 17 lines), which can only
   be moved by also moving the left/right cluster placements that read it —
   i.e. a much larger redesign, not a follow-up trim.
