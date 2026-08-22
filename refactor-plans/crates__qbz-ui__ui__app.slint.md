# crates/qbz-ui/ui/app.slint (219 lines)

## Summary
The Slint UI entry point: bundles fonts, imports/re-exports every state
global + the theme globals (one enormous re-export statement of ~140 state
symbols from `state.slint`, currently written as a single physical line), and
defines the top-level `AppWindow` (screen switch between Splash/Login/
AppShell, window chrome/title/palette-sync logic, the global Toast overlay,
and the frameless-mode hairline border).

## Proposed split
This file's line count is dominated by one thing: `AppWindow`'s body (screen
routing + window-chrome logic, ~185 of the 219 lines) plus the giant
re-export line (line 32, one physical line listing ~140 symbols — already
"1 line" by a naive line-counter, but violates the SPIRIT of the rule since
it's an unmaintainable wall of text). Split by concern:

- `app.slint` (~70 lines) — stays the re-export/entry surface: font imports,
  `Palette`/`Theme`/`ThemeState`/`Typography` re-exports, the `AppScreen` enum,
  and `export component AppWindow inherits Window { ... }` header
  (title/size/background/no-frame/resize-border, `system-font`, `screen`
  property, forwarded callbacks) — but delegates the BODY (screen-switch tree,
  palette-sync, window-width publish) to a helper, OR keeps the property/
  callback declarations here and moves only the re-export line + the
  screen-tree out. Recommended concrete split below.
- `foundation/state-reexports.slint` (~10 lines, but see note) — the giant
  `export { ... } from "state.slint";` line (line 32), reformatted as ONE
  symbol per line (Slint's `export {A, B, C} from "x"` syntax allows
  multi-line formatting) — this alone will be ~140 lines once reformatted, so
  it should be split further by domain into 3-4 files: e.g.
  `state-reexports-library.slint` (Favorites/Library/Playlist/Sidebar/Folder-
  related symbols), `state-reexports-discover.slint` (Home/Discover/ForYou/
  Pinned/ExternalReco/Mix/Genre), `state-reexports-player.slint` (NowPlaying/
  Queue/TrackInfo/AlbumInfo/Booklet/Visualizer/Immersive/SleepTimer),
  `state-reexports-settings.slint` (Settings/Keybindings/Diagnostics/About/
  WhatsNew/Report-issue/Dac-wizard), each doing
  `export { X, Y, Z } from "state.slint";` for its slice, then `app.slint`
  does `export { ... } from "foundation/state-reexports-library.slint";`
  chained re-exports (Slint supports re-exporting an already-exported symbol
  transitively) — OR, simpler and less risky: just leave the re-export line as
  literal pass-through and reformat to multi-line in `app.slint` itself,
  accepting that this one export statement (even multi-line) is arguably "one
  unit" the 130-line rule's spirit tolerates as a deliberate exception (flag
  this for the owner to decide — see below).
- `shell/AppWindowChrome.slint` (~130 lines) — extract `AppWindow`'s screen-
  routing tree (`if root.screen == AppScreen.splash/login/shell: ...`, lines
  128–196) into a child component `AppWindowScreens` that takes `screen:
  AppScreen` plus all the forwarded callback bindings, OR keep this inline —
  given `AppWindow` IS the Window root and Slint callbacks/`in-out property`
  screen must live on the actual `Window`-inherited component for the winit
  layer to see them (title/no-frame/etc. are Window-level properties), this
  extraction is LOWER-VALUE than the re-export split; prioritize the
  re-export split first.

## Re-export surface
`app.slint` remains the file `main.rs` imports the generated `AppWindow` from
(`slint::include_modules!()` or explicit `slint_build` config points at this
file) — this is the root of the whole UI, so nothing else re-exports it; it IS
the top of the tree. The state re-exports keep flowing through `app.slint`
(directly or transitively via the new `state-reexports-*.slint` files) so
Rust's `slint::ComponentHandle` global setters (`app.global::<FavoritesState>()`
etc.) keep resolving via the same generated module.

## Coupling / watch out
- **This file is a genuine edge case for the 130-line rule.** The re-export
  line is functionally one statement (forwarding globals so Rust can set
  them) — mechanically splitting it into multiple files adds indirection
  without adding cohesion (unlike the FavoritesView-style UI splits). Flag
  this to the owner: recommend either (a) accept this file as a rule
  exception with a comment explaining why, or (b) do the domain-sliced
  `state-reexports-*.slint` split above purely to satisfy line-count, with
  the understanding it's bookkeeping, not a readability win.
- `AppWindow.no-frame` computation (`!AppearanceState.is-macos &&
  !AppearanceState.system-title-bar-active`) and the frameless hairline
  Rectangle at the bottom (lines 211–218) both depend on this same
  `no-frame`/`WindowControlActions` state — keep together if extracting
  window-chrome.
- `sync-native-palette()` is called both from `init =>` and from two
  `changed` handlers (`pal-is-dark`, `pal-is-system`) — these three call
  sites must stay attached to the SAME `AppWindow` instance; don't split this
  function away from the property declarations it reads.
- The generated Rust bindings (`slint::include_modules!()` or codegen output)
  reference `AppWindow` and every re-exported global by name — a botched
  re-export split (missing a symbol, wrong transitive chain) will show up as
  a Rust compile error in every crate that does `app.global::<...State>()`,
  which is a LOT of call sites given ~140 exported symbols — this is the
  highest-blast-radius file in this whole batch; do the split incrementally
  (one domain slice at a time) and `cargo check -p qbz-ui` (or the crate that
  owns `main.rs`) after each slice.

## Verify after split
- `cargo build -p qbz-ui` (or whichever crate contains `main.rs` /
  `build.rs` running `slint_build::compile`) after EVERY incremental
  re-export slice — this is the file most likely to break Rust callers
  silently if a symbol is dropped from the export chain.
- Grep the Rust side for every `.global::<XxxState>()` / `.global::<XxxActions>()`
  call and confirm each symbol used there is still exported from `app.slint`
  (directly or transitively).
- Manual smoke test: launch the app, confirm Splash → Login/Shell routing
  still works, theme switching (light/dark/system) still syncs the native
  palette, window resize/frameless hairline still renders.
