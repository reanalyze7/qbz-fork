# split-plan: crates/qbz-ui/ui/app.slint

**Current: 237 lines. Budget: 130. Target after split: 118.**

`ui/app.slint` is the single entry point handed to `slint_build::compile_with_config`
(crates/qbz-ui/build.rs:13). Everything the Rust side sees — `AppWindow`, `AppScreen`,
every `*State` / `*Actions` global, every struct — is whatever this document *exports*.
`crates/qbz-ui/src/lib.rs` is just `slint::include_modules!()`, so the generated names
land in `qbz_slint_ui::*` and are re-imported all over `crates/qbz/src`
(`use crate::{AppWindow, MusicianState}`, ~940 `.global::<…>()` call sites).
The compile cannot be verified locally (documented 20–30 GB memory wall on a full
`cargo check` of the Slint tree), so the plan is built on the 1.16.1 compiler source
under `~/.cargo/registry/…/i-slint-compiler-1.16.1`, cited inline.

---

## 1. Why is this file long?

Line-by-line census of the 237 lines:

| Lines | Bytes of concern | Category | Movable? |
|---|---|---|---|
| 1–5 | file header comment | prose | stays (5 L) |
| 7–12 | 4 × `import "assets/fonts/Inter_18pt-*.ttf"` + comment | font registration | **movable** (see §2.1) — recommended to stay |
| 14–21 | 8 named imports used by the window body | imports | shrinks with the body |
| 23–29 | `export { Theme, ThemeColors, ThemeState, Typography }` | re-export | **movable** (§2.2) |
| 31–49 | the old "EXEMPTION (130-line rule)" comment | prose | **deleted** — its premise is wrong (§2.2) |
| 50 | `export { …170 names… } from "state.slint"` | re-export | **movable** (§2.2) |
| 52–54 | `export enum AppScreen` + comment | type decl | **movable** (§2.3) |
| 56–95 | `AppWindow` window-level properties (title, min/preferred size, `no-frame`, `resize-border-width`, `default-font-family`) | window contract | **irreducible** (§3) |
| 97–105 | `init` / `changed width` publishing `ShellState.window-width` | window contract | irreducible (needs `self.width` of the Window) |
| 107–121 | std-widgets `Palette.color-scheme` sync | side effect | **movable** (§2.4) |
| 123–144 | `screen` property + 18 callback declarations | Rust-facing API | must stay *declared* here, but collapses from 20 L to 18 L (§2.5) |
| 146–214 | the three `if screen == …:` screen mounts + 18 callback forwards | routing | **movable** (§2.5) — 69 L, the single largest block |
| 216–222 | `Toast` overlay | 1 element | stays (7 L) |
| 224–236 | frameless 1px hairline | overlay element | **movable** (§2.6) |

So: the file is long for one boring reason — **the screen-routing tree and its
callback plumbing (lines 146–214, 69 lines) live in the same file as the window
contract**. The 170-name re-export that the old exemption comment blamed is *one
physical line*; it was never the problem.

---

## 2. The seams

### 2.1 Font imports — they CAN move, and are staying anyway

Claim in the old comment (implicitly) and in Slint folklore: `import "x.ttf"` only
registers a font if it sits in the entry document. **This is false in 1.16.1.**

`passes.rs:306` calls
`collect_custom_fonts::collect_custom_fonts(doc, std::iter::once(&*doc).chain(type_loader.all_documents()), …)`,
and the pass itself (`passes/collect_custom_fonts.rs:19-23`) does:

```rust
let mut all_fonts = BTreeSet::new();
for doc in all_docs { all_fonts.extend(doc.custom_fonts.iter().map(|(path, _)| path)) }
```

then attaches the registration calls to **the root document's** `exported_roots()`.
`embed_glyphs.rs:98` walks the same all-documents iterator. So a `.ttf` imported by
*any* document in the compilation unit is registered in `AppWindow`'s init code.
Two constraints if they ever move:

- the path is resolved against the **importing** file
  (`object_tree.rs:205-220`, `pathutils::join(token_path, import_file_path)`), so a
  file under `ui/foundation/` must write `"../assets/fonts/…"`. The tree already
  relies on this for `@image-url("../assets/icons/…")`.
- the host file must be reachable from `app.slint`, and it must be reachable by a
  **named** import or an `export … from`. A bare `import "foundation/fonts.slint";`
  is a hard error: `object_tree.rs:232` — *"Import names are missing"* for any
  non-font file import. A font-only `.slint` file is therefore not importable.

**Decision: keep lines 7–12 in `app.slint` (7 lines).** The budget does not need
them, global font registration is exactly what a reader expects to find in the entry
file, and moving them buys a relative-path change plus a silent-failure mode (fonts
registered only as long as the host file stays in the graph) for nothing. If the
fallback in §5 is ever needed, `ui/foundation/typography.slint` (28 L) is the
coherent host — it already owns the type scale, and `app.slint` already keeps it in
the graph via the `Typography` re-export.

### 2.2 New file — `ui/app-exports.slint` (~40 lines)

**Moves: lines 23–29 and 50** (the 170-name `state.slint` re-export, the four
`foundation/*` re-exports), regrouped into one `export { … } from "…"` line per
domain, mirroring the grouping already used by `ui/state.slint`.

`app.slint` then carries exactly one line for the whole surface:

```slint
export * from "app-exports.slint";
```

Why this works, and why the old exemption comment was wrong:

- `export * from "file";` is valid grammar — `parser/document.rs:235` and the
  `SyntaxKind::Star` branch at `:280`.
- It is genuinely transitive: `typeloader.rs:1140-1158` handles
  `ImportKind::ModuleReexport` with a star by copying **`doc.exports`** of the target
  wholesale. `doc.exports` of `app-exports.slint` is itself built from its own
  `export … from` lines (`object_tree.rs:203`, `exports.add_reexports(reexports, …)`),
  which is precisely the chain `state.slint` already uses.
- The generated API is byte-identical **because the barrel re-exports the same 170
  names, not `export * from "state.slint"`**. `state.slint` exports ~250 symbols; a
  star straight off it would *add* generated globals/structs. Additive, but this plan
  is for parity, so the explicit list survives — it just lives one file over.
- Hard constraint: `typeloader.rs:1144-1148` — *"re-exporting modules is only allowed
  once per file"*. `app.slint` gets exactly one `export *`. Named `export { X } from`
  lines are unrestricted, so the barrel can use as many as it likes.
- Duplicates are a warning, not an error (`object_tree.rs:2936`), but the barrel must
  still not re-export `AppWindow` — that one is declared locally in `app.slint`.

The barrel lives at `ui/` root, **not** in a subdirectory: it re-exports from
`"state.slint"` and `"foundation/*.slint"` with the same relative paths the current
lines use, so the move is a pure copy with no path edits.

### 2.3 New file — `ui/state/app_screen.slint` (~7 lines)

**Moves: lines 52–54.**

```slint
export enum AppScreen { splash, login, shell }
```

This is forced, not cosmetic: the router in §2.5 needs `AppScreen` for its `screen`
property, and if the enum stayed in `app.slint` the router would have to import from
`app.slint`, which imports the router — a cycle. `ui/state.slint:20-24` documents the
same trick for `UiFocusState`.

`app.slint` imports it directly (`import { AppScreen } from "state/app_screen.slint";`)
and `app-exports.slint` re-exports it, so `r#AppScreen` keeps being generated under the
same name. Precedent: `ContentView` and `ToastKind` are declared in
`state/drag_local_structs.slint`, re-exported through `state.slint`, and appear in the
generated export list today.

### 2.4 New file — `ui/foundation/NativePaletteSync.slint` (~28 lines)

**Moves: lines 107–121** — the `pal-is-dark` / `pal-is-system` mirror properties, the
`sync-native-palette()` function, its two `changed` handlers, and the whole 6-line
comment explaining the invisible-text bug it fixes. Also takes the
`import { Palette } from "std-widgets.slint";` half of line 14, and the
`root.sync-native-palette()` call out of `init` (line 101) — the component seeds
itself from its own `init`.

Why it can move: nothing in the block touches `root`. It reads two `Theme` globals and
writes one `Palette` global. It is a pure side-effect node, so it becomes a
zero-visual component instantiated once as `NativePaletteSync { }` (1 line in
`app.slint`, +2 comment). Note this is behaviour with no pixels — the natural home is
`foundation/`, next to the theme globals it bridges.

### 2.5 New file — `ui/shell/AppScreens.slint` (69 lines, drafted and counted)

**Moves: lines 146–214** — the three conditional screen mounts and all 18 callback
forwards — plus a copy of the 18 callback *signatures* from lines 126–144.

```slint
export component AppScreens {
    in property <AppScreen> screen: AppScreen.splash;
    callback open-album(string);
    …
    if root.screen == AppScreen.shell: AppShell {
        width: root.width; height: root.height;
        open-album(id) => { root.open-album(id); }
        …
    }
}
```

Imports become `../splash/SplashScreen.slint`, `../login/LoginScreen.slint`,
`AppShell.slint` (same directory), `../state/app_screen.slint`. Forward bodies collapse
to one line each — 69 lines forwarded over 3 lines each today is padding, not structure.

In `app.slint` the 18 declarations and the 18 forwards fuse into 18 alias lines:

```slint
callback open-album <=> screens.open-album;
```

`parser/element.rs:332` documents `callback foobar <=> elem.foobar;` in the
`CallbackDeclaration` test corpus, and `:370-372` confirms the alias form must omit the
parameter list (the signature is taken from the target). The callback is still declared
on `AppWindow`, so `AppWindow::on_open_album` is still generated with the same
signature, and `crates/qbz/src/main.rs` needs no edit. This is the only technique in the
plan without precedent in this repo — §5 covers what to do if it misbehaves.

### 2.6 New file — `ui/primitives/FramelessHairline.slint` (~22 lines)

**Moves: lines 224–236** with their comment. The condition reads `root.no-frame`, which
a child cannot see, so the component takes `in property <bool> frameless;` and
`app.slint` passes `frameless: root.no-frame;`. `WindowControlActions.is-maximized` /
`.is-fullscreen` are globals and move with the code — which also removes
`WindowControlActions` from `app.slint`'s import line.

It stays the **last** child of `AppWindow` so it keeps painting over everything.

---

## 3. What cannot move, and why

`AppWindow` must remain a component that literally `inherits Window`, declared in
`app.slint`, holding lines 56–105:

- `title`, `no-frame`, `resize-border-width`, `default-font-family`, `min-width` /
  `min-height` / `preferred-*` and `background` are **Window-level** properties. The
  winit adapter reads `no-frame` directly and re-applies it at every realization; the
  edge-resize border only engages on an undecorated window. Hoisting the body into a
  child component would leave these behind or silently drop them.
- `init` / `changed width` publish `self.width` **of the window** into
  `ShellState.window-width`. A child's `width` is only equal by construction, and the
  binding would break the moment a child's sizing changed.
- `in property <string> system-font` is set from Rust (`crates/qbz/src/main.rs:498-501`,
  `window.set_system_font(...)`), so it must be a property of the generated `AppWindow`.
- The 18 callback names must be declared on `AppWindow` (aliased or not) because Rust
  connects them as `window.on_<name>(…)`. Moving them onto a global would change ~18
  Rust call sites and is out of scope for a parity split.

That is 40 lines of window contract plus ~10 lines of comment, and it is the honest
floor of this file. Everything above the floor is what the plan removes.

---

## 4. Result

| File | Status | Lines (est.) | Content |
|---|---|---|---|
| `ui/app.slint` | rewritten | **118** (drafted and counted) | header 1–5, fonts 7–12, 8 imports, one `export *`, `AppWindow` window contract, 18 callback aliases, 4 child elements |
| `ui/app-exports.slint` | new | ~40 | old lines 23–29 + 50, regrouped one line per domain |
| `ui/state/app_screen.slint` | new | ~7 | old lines 52–54 |
| `ui/shell/AppScreens.slint` | new | **69** (drafted) | old lines 146–214 + 18 callback signatures |
| `ui/foundation/NativePaletteSync.slint` | new | ~28 | old lines 107–121 + the `Palette` import |
| `ui/primitives/FramelessHairline.slint` | new | ~22 | old lines 224–236 |
| `ui/foundation/typography.slint` | untouched | 28 | fallback host for the font imports only if §5 forces it |

Public surface: unchanged. The generated export list (checkable against
`crates/target/debug/build/qbz-ui-*/out/app.rs`, the `pub use` line at the end of the
generated module) must contain the same set before and after — `r#AppWindow`,
`r#AppScreen`, and the 170 re-exported names. **This diff is the verification
artefact**: regenerate, `tr ',' '\n'` both export lists, `sort`, `diff`. An empty diff
is the acceptance criterion, and it does not require the app to launch.

No file under `crates/qbz/src` changes.

---

## 5. Risks

1. **`callback x <=> child.x;` is unprecedented in this repo.** Grammar-confirmed
   (`parser/element.rs:332`) but never exercised here, and a mistake surfaces only at
   full compile. If the alias form fails, or if the generated `on_x` setter changes
   shape, fall back to 18 plain declarations plus 18 one-line forwards inside
   `screens := AppScreens { … }`. That is +18 lines → **136**, over budget; recover by
   moving the four font imports and their comment into
   `ui/foundation/typography.slint` (−7, see §2.1) and trimming two comment blocks
   (−3) → ~126. The fallback fits, but with no headroom.
2. **`export *` count limit.** `typeloader.rs:1144` errors on a second `export *` in
   one file. If someone later adds another star to `app.slint` the build breaks with a
   clear message — worth a comment on the line, which the draft carries.
3. **Export-set drift.** The 170-name list is retyped into domain-grouped lines. A
   dropped or misspelled name is a compile error in the barrel *only if* the symbol is
   missing from `state.slint`; a silently *omitted* name is not an error — it just
   deletes a generated Rust type and breaks `crates/qbz/src` at `cargo build`. Do the
   regrouping mechanically (`tr ',' '\n'` → group → rejoin) and diff the sorted name
   list against the original line 50 before committing.
4. **`NativePaletteSync` init ordering.** Today `Palette.color-scheme` is seeded from
   `AppWindow.init`. As a child component it is seeded from that child's `init`. Both
   run during window construction, before the first show, but the relative order of
   parent/child `init` is not something this plan verified. Symptom if wrong: native
   `LineEdit`/`TextEdit` text renders in the wrong scheme for one frame on a light
   theme over a dark OS palette. Cheap to check by eye on the Settings page.
5. **Implicit-root geometry.** `AppScreens` and `FramelessHairline` have implicit
   (`Empty`) root elements. The draft sets `width`/`height` explicitly on both
   instantiations; if either is left unset, the conditional `AppShell` collapses to its
   preferred size — the exact regression the comment at old line 164 documents.
6. **Z-order.** `NativePaletteSync` / `AppScreens` / `Toast` / `FramelessHairline` must
   stay in that declaration order. `Toast` and the hairline are painted-last overlays,
   and the split moves them past a new sibling.
7. **No local compile.** The Slint tree cannot be `cargo check`ed in this environment
   (20–30 GB memory wall). Every claim above is sourced from the 1.16.1 compiler source
   rather than from a build, and the export-list diff in §4 is the only cheap check
   available. Land this on its own branch and run the full build in CI before merging.
