# crates/qbz-ui/ui/shell/HeaderSearch.slint — 164 lines

Target: 130. Overshoot: 34 lines.

## 1. Why is this file long?

Three separable responsibilities sit in one file:

1. **Focus plumbing** — the `FocusScope` root, `forward-focus: search-input`,
   the `release-focus()` public function, and the `changed has-focus` guard on
   the TextInput. This is the part that is genuinely irreducible: it must stay
   in one file (see §4).
2. **Key routing** (lines 35–82, 48 lines) — `on-enter()` plus the
   `key-pressed` decision table (Down/Up/Return/Escape vs. reject). It is
   stateless dispatch onto the `SearchState` / `SearchActions` globals; the
   only thing it needs from the file is `search-input.text`.
3. **Non-interactive text hints** (lines 140–162, 23 lines) — the centred
   "Search" placeholder and the right-edge "↵ Enter" hint. Pure painting; no
   focus, no input, no callbacks.

Only (1) is irreducible. (2) and (3) move out, which is enough: 164 − 40 − 17
+ 2 imports ≈ **109 lines** left in `HeaderSearch.slint`.

The existing header comment claims the file is "DELIBERATELY NOT split
further". That claim is correct *about the FocusScope and its TextInput* and
this plan keeps them together; it does not cover the key table or the hints.
The comment must be rewritten, not deleted, so the `forward-focus` constraint
stays recorded.

## 2. The seams

### New file A — `crates/qbz-ui/ui/shell/HeaderSearchKeys.slint` (~60 lines)

A **global**, not a component: the extracted code is stateless dispatch and has
no geometry, so it needs no owning element. Precedent in this tree:
`ui/primitives/ColorPickerMath.slint` (global holding functions extracted from
`ColorPicker.slint`), and `ShellState.cycle-sidebar()` for a global function
that fires a callback on its own global.

Moves: **lines 35–82** (the `on-enter()` function and the whole `key-pressed`
body, with their comments).

Shape:

```
export global HeaderSearchKeys {
    // (comment block from lines 35-38)
    public function on-enter(input-text: string) { ... }
    // (comment block from lines 51-53)
    // Returns true when the key was consumed (caller returns `accept`).
    public pure function is-handled(key: string) -> bool { ... }   // optional
    public function handle-key(key: string, input-text: string) -> bool { ... }
}
```

Call site in `HeaderSearch.slint` becomes:

```
key-pressed(event) => {
    if (HeaderSearchKeys.handle-key(event.text, search-input.text)) {
        return accept;
    }
    return reject;
}
```

Body transformations required:

- `search-input.text` (line 47) → the `input-text` parameter. This is the one
  element-id reference that crosses the file boundary; passing it as an
  argument at the call site is the bridge.
- `root.on-enter()` (line 70) → `HeaderSearchKeys.on-enter(input-text)`.
  Inside a global, `root` refers to the global itself, so no `root.` may
  survive the move except as an explicit `HeaderSearchKeys.` qualifier.
- `return accept` / `return reject` are `KeyEventResult` values and are **not**
  valid outside a key handler. They become `return true` / `return false`, and
  the caller maps them back. This is the only semantic rewrite in the move.

Verified: a global function may call a callback on a *different* imported
global (`SearchActions.cortinilla-move-selection(1)` from inside
`HeaderSearchKeys`), and a `key-pressed` handler may call it. Compiled a
minimal reproduction of exactly this shape with `slint-viewer` — no diagnostics.

### New file B — `crates/qbz-ui/ui/shell/HeaderSearchHints.slint` (~42 lines)

A component holding the two text overlays painted on top of the input.

Moves: **lines 140–162** — the placeholder `Text` (140–149), the comment block
(151–153) and the "↵ Enter" `Text` (154–162).

```
export component HeaderSearchHints inherits Rectangle {
    // True while the field is empty. Bridged from the caller because
    // `search-input` is an id in HeaderSearch.slint and ids are file-scoped.
    in property <bool> show-placeholder;
    background: transparent;
    Text { visible: root.show-placeholder; ... width: root.width; height: root.height; }
    Text { visible: SearchState.cortinilla-open; x: root.width - self.width - 10px; ... }
}
```

Call site, declared **after** `search-input` inside the box Rectangle so the
paint order (hints over the input) is unchanged:

```
HeaderSearchHints {
    width: 100%;
    height: 100%;
    show-placeholder: search-input.text == "";
}
```

Imports needed by the new file: `Theme`, `Typography`, `SearchState`.

### What deliberately does NOT move

- **The magnifier `QbzIcon` (lines 94–103).** It is 8 lines, and it is declared
  *before* the TextInput on purpose: centred text paints over the glyph. Folding
  it into `HeaderSearchHints` (declared after the input) would flip that paint
  order and put the glyph over a long query string. Not worth 8 lines.
- **The `TextInput` and its `changed has-focus` handler (106–139).** See §4.
- **`animate width` on the root (line 26).** The caller recentres via
  `x: (root.width - self.width) / 2`; the animation must stay on the element
  whose `width` the caller reads.

## 3. Public surface

`shell/HeaderSearch.slint` keeps its path and exports `HeaderSearch` with the
identical surface:

- `in property <length> search-width` — unchanged.
- `public function release-focus()` — unchanged, still calls
  `search-input.clear-focus()` (the id stays in this file) and
  `root.clear-focus()`.
- No callbacks are declared today and none are added.

The single importer is `crates/qbz-ui/ui/shell/HeaderBar.slint`
(import line 19, instantiation `header-search := HeaderSearch { … }` at line
121, `header-search.release-focus()` in the `changed nav-view-probe` handler at
line 61). It needs **no edit**. A repo-wide grep for `HeaderSearch` over
`crates/qbz-ui/ui/` returns only that file plus the declaration itself.

The two new files are internal to the split and are imported only by
`HeaderSearch.slint`; neither is re-exported from `state.slint` (they are not
app state) nor from `app.slint`.

## 4. Slint hazards

**FocusScope placement.** The `FocusScope` stays as the root of
`HeaderSearch.slint`, and `search-input` stays in the same file. Two reasons,
both hard constraints:

- `forward-focus:` only resolves an id declared in the *same* file. This is
  documented in-tree by `shell/LinkResolverInputRow.slint`, whose header records
  that the parent's `forward-focus: url-input` stopped resolving once the input
  moved into a child file, and that the workaround was a `public function
  focus-input()`. Applying that workaround here would be a behaviour change,
  not a refactor: `forward-focus` routes *every* keystroke, not a one-shot focus
  call, so the "scope owns focus, typing lands in the input" arrangement would
  have to be rebuilt.
- Key events are delivered to the focus owner. `key-pressed` must remain
  declared on the `FocusScope` element for the scope's handler to run *before*
  the input consumes the event — that ordering is what makes Up/Down/Enter/
  Escape cortinilla-navigable while the input keeps the cursor. Only the
  *body* of the handler moves; the declaration site does not.

**Element-id references crossing a new boundary.** Exactly two, both bridged:

| Reference | Old form | Bridge |
| --- | --- | --- |
| `search-input.text` in `on-enter` (line 47) | direct id read | `input-text: string` parameter of `HeaderSearchKeys.on-enter` / `handle-key`, passed at the call site |
| `search-input.text == ""` in the placeholder (line 141) | direct id read | `in property <bool> show-placeholder` on `HeaderSearchHints`, bound at the call site |

The placeholder could instead read `SearchState.header-search-text == ""`
directly, since the input is two-way bound to it. Rejected: it makes the
placeholder depend on binding propagation order rather than on the widget it
describes, for no line saving.

**Two-way bindings.** The only `<=>` in the file is
`text <=> SearchState.header-search-text` (line 117). It stays with the
TextInput, so no two-way binding crosses a file boundary. Nothing in this plan
introduces a new one — `show-placeholder` is `in`/one-way on purpose (a `<=>`
would need a writable property on the child and give the child authority over
the input's text).

**`parent.` / `root.` meaning changes.**

- Lines 143–144 (`width: parent.width; height: parent.height` on the
  placeholder) and lines 160–161 (`x: parent.width - self.width - 10px`,
  `y: … parent.height …` on the Enter hint) resolve to the **box Rectangle**
  today. Inside `HeaderSearchHints` the nearest `parent` is the component's own
  root, so all four must be rewritten to `root.width` / `root.height` and the
  call site must pin the component to `width: 100%; height: 100%` of the box.
  With that pin the values are numerically identical.
- `root.on-enter()` inside the key table (line 70) becomes a
  `HeaderSearchKeys.`-qualified call; `root` inside a global is the global.
- `root.focus()` in `changed has-focus` (line 133) does **not** move and still
  means the FocusScope. Same for `root.search-width` (line 23).

**Absolute centring / geometry.** The centring lives entirely in the caller
(`HeaderBar.slint` lines 122–123, computed from `self.width`), and the animated
width lives on the `HeaderSearch` root. Neither moves. The hints are sized in
percentages of the box, so they follow the animated width for free — the same
as today. The magnifier's rounding expression
(`Math.round((parent.height - self.height) / 2 / 1px) * 1px`) stays in place
untouched; the identical expression on the Enter hint is carried into
`HeaderSearchHints` with `parent` → `root`.

**Hit testing.** `HeaderSearchHints` inherits `Rectangle` and contains only
`Text` elements — no `TouchArea` — so it consumes no pointer events and a click
anywhere in the box still lands on the TextInput, which is what drives the
`changed has-focus` → `root.focus()` routing. Do not give the new component a
`TouchArea` or a non-transparent background.

**Global singleton semantics.** `HeaderSearchKeys` is stateless (no
properties), so the fact that a Slint global is a process-wide singleton is
harmless even if a second search field is ever added.

**`cache-rendering-hint`.** Lives on `HeaderBar`'s root (line 40) and caches the
whole header subtree. Splitting a child into more components does not change
the subtree contents or its invalidation, so the #617 idle-repaint work is
unaffected.

## 5. Result

| File | Lines (est.) |
| --- | --- |
| `shell/HeaderSearch.slint` | ~109 |
| `shell/HeaderSearchKeys.slint` (new) | ~60 |
| `shell/HeaderSearchHints.slint` (new) | ~42 |

Verification: `cargo build -p qbz-ui` (`build.rs` compiles `ui/app.slint`, so
any unresolved id, bad `parent.` or invalid `return accept` is a build error),
then a manual pass on the live behaviours the file exists for — type to open
the cortinilla, Up/Down move the highlight without the caret moving, Enter on a
highlighted row vs. Enter with no highlight, Escape closes without unfocusing,
placeholder disappears on the first character, "↵ Enter" appears only while the
cortinilla is open, click the field then press a single-key hotkey (guard),
navigate to another page mid-search and confirm hotkeys revive
(`release-focus()`), and resize across the 960px / nav-in-sidebar breakpoints to
confirm the field stays centred while the width animates.

## Risks

- **Key-handler rewrite is not a pure move.** `accept`/`reject` become
  `true`/`false` and the mapping moves to the call site. A mistake inverts a
  branch and silently changes which keys reach the TextInput — e.g. returning
  `accept` for Up/Down while the cortinilla is closed would freeze the caret.
  Each of the five branches must be checked against the original individually.
- **`Key.Return` fallback path.** The TextInput's `accepted =>
  { root.on-enter(); }` (line 121) is a second entry point into the same logic
  and must be repointed to `HeaderSearchKeys.on-enter(self.text)`. Forgetting
  it leaves a stale `root.on-enter()` — a build error, so it fails loudly, but
  repointing it to the wrong text source (e.g. `search-input.text` vs
  `self.text`) would not.
- **Focus regressions are not caught by the compiler.** Anything touching the
  FocusScope/TextInput pair — including an accidental reorder of the children
  inside the box — can only be caught by the manual pass above.
- **Paint order.** `HeaderSearchHints` must be declared after `search-input`.
  Declaring it before hides the placeholder and the Enter hint behind the
  input's (transparent) background in the general case and is easy to miss in
  review.
- **Percentage pin.** If the call site omits `width: 100%; height: 100%`, the
  new Rectangle collapses to zero and the right-anchored Enter hint lands at a
  negative `x`. Silent visual bug, no diagnostic.
- **Line budget is met with ~20 lines of margin** in `HeaderSearch.slint`. The
  remaining file is the focus/geometry core; the next feature that touches
  cortinilla focus will eat into that margin, and there is no further seam left
  that does not break `forward-focus`. If it grows again, the correct move is a
  documented exception, not another split.
