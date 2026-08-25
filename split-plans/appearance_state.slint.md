# crates/qbz-ui/ui/state/appearance_state.slint — 197 lines

Declares one global: `AppearanceState`. Exported from the barrel
`crates/qbz-ui/ui/state.slint` (one line, `export { AppearanceState } from
"state/appearance_state.slint";`).

**Verdict: split into three globals** (`AppearanceState`,
`AppBackgroundState`, `CustomThemeState`, `WindowChromeState`), landing the
original file at ~118 lines. Two of the three extractions are justified
independently of the line budget; one is budget-driven. If the reviewer
rejects the third, the fallback is a documented exception at ~150 lines,
which is still strictly better than today's 197. Detail below.

---

## 1. Why is this file long?

Not because of one irreducible declaration, and not because of filler.

- 197 lines total: **81 declaration lines**, **97 comment lines**, 15 blank.
  Nearly half the file is prose. The comments are load-bearing — they record
  which properties are actually wired to Rust and which are inert replicas of
  the old Tauri appearance panel, per-OS default rationale for the title-bar
  block, and the env-var tuning contract for the dynamic background. Deleting
  them to fit the budget would destroy the only record of that.
- The remaining 81 declarations are **not one concern**. The file is a
  concatenation of nine commented sections (`--- THEME ---`,
  `--- TITLE BAR ---`, `--- SYSTEM TRAY ---`, `--- RENDERER ---`, …) that were
  merged because they all happen to be reachable from the *Settings >
  Appearance* panel. "Appears in the same settings panel" is a UI-layout fact,
  not a data-cohesion fact.

The Slint constraint is real and settles the shape of the answer: a `global`
is a single atomic declaration and cannot be spread over two files. So the
only lever is **how many globals there are**, and the question is whether the
current single global is one thing or several. It is several — but the seams
are not where the section comments are, because the section comments track the
settings panel's visual grouping, not who reads the data.

The useful seam is **who consumes the property**:

| consumer shape | properties | verdict |
| --- | --- | --- |
| read by leaf render components all over the app | `app-background-*` | genuinely misplaced |
| read by exactly one settings sub-panel + its own Rust module | `custom-*` | genuinely separable |
| read by the window shell (`app.slint`, `HeaderBar`) as window policy | title-bar / `wc-*` | separable, weaker case |
| read only by the Appearance panel rows themselves | theme, language, tray, renderer, notifications, startup, ui-scale, visual extras | one cohesive settings surface — must stay whole |

---

## 2. What are the seams?

Measured from the current file. Line ranges are inclusive.

### Seam A — `AppBackgroundState` (lines 168–189, 22 lines)

Properties that move, verbatim:

```
app-background-modes        app-background-mode-index
app-background-available    app-background-dim
app-background-surface-alpha app-background-bar-alpha
```

**Why this is the strongest case, budget aside.** 34 of the 39 `.slint` files
that mention `AppearanceState` reference *nothing but* these six properties:

```
primitives/ToggleButton, QbzSelect, CircleAction, SegmentedTabBar,
ExpandableSearchOverlay
shell/AppShell, AppShellContentFrame, AppShellDynamicBackground, HeaderSearch,
PlayerBar, QueueSidebar, Sidebar, SidebarNowPlayingDock, sidebar/SidebarSortMenu
discover/ (13 files: carousels, nav buttons, toolbars, tag bars, search field)
artist/ArtistPageChrome, label/LabelPageOverlays, playlist/PlaylistSearchSort,
search/SearchToolbar
state/immersive_window_control, state/shell_state_main
```

`app-background-surface-alpha` alone has 27 reference sites. These are leaf
primitives reading a translucency float to decide how solid to paint a chrome
strip. Today a button primitive has to import the whole appearance settings
surface — tray flags, renderer options, custom-theme swatches — to do that.
That is a real coupling problem, and it is what makes `AppearanceState` show up
in an import list where it makes no sense.

After the move, those 34 files stop depending on the settings surface entirely.
Only two files are mixed and would import both globals:
`shell/HeaderBar.slint` and `settings/appearance/ThemeSection.slint` (the
picker row itself).

Five files import `AppearanceState` and reference no property at all
(`album/AlbumHeader`, `artist/ArtistHeaderBio`, `discover/PlaylistCarousel`,
`settings/view/SettingsActivePanel`) — stale imports, drop them while here.

### Seam B — `CustomThemeState` (lines 40–44, 46–68, 28 lines)

Properties and callbacks that move:

```
custom-surface-main   custom-surface-card   custom-surface-elevated
custom-text-primary   custom-text-secondary custom-accent
custom-danger         custom-warning        custom-success
custom-border         custom-favorite       custom-is-dark
custom-open-token
callback custom-set-token(string, color)
callback custom-set-token-hex(string, string)
callback custom-seed-from-current()
callback custom-toggle-dark(bool)
```

`theme-is-custom` (line 45) **stays** in `AppearanceState`. It is one leg of the
tri-state `theme-is-system` / `theme-is-auto` / `theme-is-custom` that the theme
dropdown owns; separating one leg from the other two would be worse than the
problem being solved.

Why it separates cleanly: 17 members with exactly one UI consumer
(`settings/appearance/CustomThemeEditor.slint`, plus `custom-open-token` in
`settings/appearance/shared.slint`), and a Rust owner that is *already* its own
module (`crates/qbz/src/custom_theme/`). The global is the only place the
boundary is not drawn.

### Seam C — `WindowChromeState` (lines 94–130, 37 lines)

Properties that move:

```
window-title-show      window-title-template
use-system-title-bar   system-title-bar-active
hide-title-bar         match-system-chrome    hardware-accel-enabled
wc-positions           wc-position-index
wc-styles              wc-style-index
wc-sizes               wc-size-index
wc-color-presets       wc-color-preset-index
show-window-controls
```

`is-macos` (line 139) **stays** in `AppearanceState`: it is a platform flag read
by the tray section and `AppearanceSettings.slint` as well as by the chrome
rows, and duplicating it into two globals would give Rust two things to keep in
sync for no gain.

This is the weaker of the three. The argument for it: window chrome is OS
window policy, consumed at render time by `app.slint` (which sets `no-frame`)
and `HeaderBar.slint` (drag region, inset, controls) — not by the settings
panel alone. The argument against it: the six consumer files are all *mixed*
(they also read `is-macos` and fire `appearance-bool`), so every one of them
ends up importing two globals instead of one. The honest framing is that Seam C
is what gets the file under budget; A and B would stand on their own merits at
any budget.

### Line arithmetic

| file | lines |
| --- | --- |
| `state/appearance_state.slint` (after) | ~118 |
| `state/app_background_state.slint` (new) | ~32 |
| `state/custom_theme_state.slint` (new) | ~38 |
| `state/window_chrome_state.slint` (new) | ~47 |

Original 197, minus 22 (A) minus 28 (B) minus 37 (C) = 110, plus ~8 lines of
replacement pointer comments at each removal site and in the file header = ~118.
Margin under budget is ~12 lines, which is thin — see Risks.

### Optional, independent: dead properties

Found while mapping consumers. These are declared, some are seeded from Rust,
and **no `.slint` file reads any of them**:

- `fonts`, `font-index` — the typography row was never built.
- `match-system-chrome`, `hardware-accel-enabled` — "Phase 2 (deferred)" per the
  inline comment; nothing reads either.
- `wc-styles`, `wc-style-index`, `wc-sizes`, `wc-size-index`,
  `wc-color-presets`, `wc-color-preset-index` — all six are seeded from
  `drag_sidebar/part3.rs:46-63`, none is rendered. Only `wc-positions` /
  `wc-position-index` are live.
- `theme-is-system` — written from Rust in three places, read by no `.slint`.
- `tray-minimize-to-tray` — set in `shell_bootstrap/part2.rs`, no reader.

Removing these (and their Rust seed lines) is worth ~13 declaration lines plus
comments. Keep it as a **separate commit** from the split: it is a behavior
decision (are these deferred features or abandoned ones?), not a mechanical
move, and mixing the two would make the split diff unreviewable.

---

## 3. What does the public surface become?

### Slint

`state.slint` gains three export lines next to the existing one:

```slint
export { AppearanceState } from "state/appearance_state.slint";
export { AppBackgroundState } from "state/app_background_state.slint";
export { CustomThemeState } from "state/custom_theme_state.slint";
export { WindowChromeState } from "state/window_chrome_state.slint";
```

Unlike a struct split, this is **not** import-transparent: a new global is a new
name. Every consumer that reads a moved property must change both its import
list and its qualifier (`AppearanceState.app-background-dim` →
`AppBackgroundState.app-background-dim`). That is the price of the split and
the reason to weigh it. For Seam A the churn is mechanical and touches files
that reference nothing else (34 of 36); for B and C the files import both.

The barrel's header comment currently documents `appearance_state.slint` as one
of two deliberate over-budget exceptions. Update it: after this split only
`state/local_library_state.slint` (~152 lines) remains an exception.

### Rust

`crates/qbz-ui/src/lib.rs` is `slint::include_modules!()`, so the three new
globals appear in the generated bindings automatically — no manual export list
to extend. Call sites change from `window.global::<AppearanceState>()` to the
new type, and any function that touches two groups needs two handles.

---

## 4. Every Rust call site to update

Full list. Nothing outside these files touches a moved property.

### Seam A — `AppBackgroundState`

- `crates/qbz/src/main.rs:423` — `let ap = window.global::<AppearanceState>();`
  becomes `AppBackgroundState`. The whole block (lines 423–446) touches only
  moved properties: `set_app_background_available` (424),
  `set_app_background_dim` (432), `set_app_background_surface_alpha` (438),
  `set_app_background_bar_alpha` (444). One-line change.
- `crates/qbz/src/main.rs:581` — `window.global::<AppearanceState>()
  .set_app_background_mode_index(...)`. Retarget the global.
- `crates/qbz/src/drag_sidebar/part3.rs:37` — `set_app_background_modes`. This
  function reseeds ten option arrays from one `state` handle acquired at line
  19; it needs a second handle for this call.
- `crates/qbz/src/wire_link_and_import/appearance_select_a.rs:29` — the
  `"app-background"` branch persists to `ui_prefs` only and never touches the
  global. **No change.**

### Seam B — `CustomThemeState`

- `crates/qbz/src/custom_theme/state.rs:16, 56, 77` — three
  `window.global::<AppearanceState>()` bindings. Retarget all three; the ~35
  `get_custom_*` / `set_custom_*` accessor lines that follow are unchanged.
  Update the module doc comments at lines 1 and 13 that name `AppearanceState`.
- `crates/qbz/src/custom_theme/actions.rs:43` —
  `window.global::<AppearanceState>().set_custom_is_dark(is_dark)`. Retarget.
- `crates/qbz/src/wire_link_and_import/wire_appearance_action_custom.rs:8` —
  needs **two** handles. `on_appearance_action` (line 13) stays on
  `AppearanceState`; `on_custom_set_token` (27), `on_custom_set_token_hex` (33),
  `on_custom_toggle_dark` (39), `on_custom_seed_from_current` (45) move to
  `CustomThemeState`.
- `crates/qbz/src/wire_link_and_import/appearance_select_b.rs:58, 71, 91` —
  `set_theme_is_custom`. Stays in `AppearanceState`. **No change.**

### Seam C — `WindowChromeState`

- `crates/qbz/src/main.rs:548` — block acquires `appearance`; of its calls only
  `set_window_title_show` (549) moves. The rest of the block
  (`set_show_volume_steppers`, `set_sidebar_playlist_collage`,
  `set_local_library_track_artwork`, `set_in_app_toasts`, `set_theme_filter`)
  stays. Needs a second handle.
- `crates/qbz/src/main.rs:568` — the custom-chrome seed block. All five calls
  move: `set_use_system_title_bar`, `set_system_title_bar_active`,
  `set_hide_title_bar`, `set_show_window_controls`, `set_wc_position_index`.
  One-line change to the handle. Note the ordering constraint in its comment:
  this must still run before the first `show()`.
- `crates/qbz/src/wire_link_and_import/wire_appearance_bool.rs:25` —
  `w.global::<AppearanceState>().set_system_title_bar_active(value)` inside the
  `on_appearance_bool` handler. The callback itself stays on `AppearanceState`
  (line 9/11); only this inner write retargets.
- `crates/qbz/src/drag_sidebar/part3.rs:45, 46, 52, 57` — `set_wc_positions`,
  `set_wc_styles`, `set_wc_sizes`, `set_wc_color_presets`. Same second-handle
  situation as Seam A; the same function now needs three handles.

### Untouched by all three seams (confirmed)

`auto_theme/mod.rs`, `auto_theme/interactive.rs`, `custom_theme/actions.rs:53`
(reads `SlintTheme`, not this global), `shell_bootstrap/part2.rs` (tray /
renderer / ui-scale — all stay), `wire_appearance_select.rs`,
`appearance_select_a.rs`, `ui_prefs/index_maps.rs`, `theme/dropdown.rs`,
`tray_settings.rs`, `playback/meta/statics.rs`, `main.rs:538/541/593/599/608/659`
(gradient, intelligent-search, startup page, language, theme block, is-macos).
The last four of those are doc comments only.

---

## The case for *not* splitting, and why it loses

It deserves stating, because the barrel currently records the opposite decision.

The argument for an exception: `AppearanceState` is the data backing one screen.
A user who opens Settings > Appearance sees these rows together; a developer
changing that screen wants them in one file. Splitting scatters that surface
across four files, and the generic dispatch callbacks (`appearance-bool`,
`appearance-select`, `appearance-text`, `appearance-action`) stay behind in
`AppearanceState`, so every extracted global's *edit* path still routes back
through the parent — the split separates storage without separating control
flow. On top of that, the split is not import-transparent: it is ~45 `.slint`
files and ~12 Rust sites of churn.

Why it loses anyway: the 34 leaf components reading `app-background-*` are not
part of that screen at all, and they are the majority of this global's
consumers. The "one cohesive screen" argument is true of the theme / language /
tray / renderer / notifications / startup remainder — and that remainder is
exactly what stays in `AppearanceState`, at ~118 lines, under budget, intact.
The split does not scatter the settings surface; it evicts from it two things
that were never part of it.

The dispatch-callback objection is real but is an argument about a *later*
refactor (giving each extracted global its own typed callbacks instead of the
string-keyed generic ones), not an argument for leaving 197 lines in one file.

---

## Risks

- **Not import-transparent.** Unlike every struct split in this repo, renaming
  a global breaks every reference by construction. ~45 `.slint` files change.
  Slint will error on an unknown global, so nothing fails silently — but a
  half-applied rename will not compile, and the diff is large enough that a
  reviewer cannot eyeball it. Do the three seams as three commits, each
  compiling.
- **Thin margin.** `appearance_state.slint` lands at ~118 lines, 12 under
  budget. The next two settings rows added put it back over. If that margin
  matters, land the dead-property cleanup (~13 lines) as the follow-up commit
  and the file sits near 105.
- **Startup ordering.** `main.rs:568` seeds chrome properties *before* the first
  `window.show()` because `AppWindow.no-frame` reads `use-system-title-bar` at
  surface creation (decorations negotiate then on Wayland). Retargeting the
  handle must not move the block. Regression symptom is subtle: a frameless
  window on Linux that comes up with system decorations, or the reverse, on
  first launch only.
- **Two-way binding across globals.** Several settings rows write their property
  locally (`AppearanceState.x = value`) and then fire `appearance-bool`. Where
  the property moves but the callback does not, that row now writes to global 1
  and calls global 2. Functionally fine, visually odd in the source; the rows in
  `TitleBarSection`, `WindowControlsSection` and `WindowTitleSection` should get
  a one-line comment saying why.
- **`is-macos` and the tri-state theme flags stay behind deliberately.** Anyone
  executing this plan will be tempted to move `is-macos` into
  `WindowChromeState` (three of its six readers are chrome files) and
  `theme-is-custom` into `CustomThemeState`. Both would create a
  split-source-of-truth or split-tri-state problem. They are flagged in the
  seam sections; keep them.
- **Dead-code discovery is out of scope but load-bearing.** Six of the sixteen
  properties moving in Seam C are unread. If the reviewer decides they should be
  deleted rather than moved, Seam C shrinks to ~25 lines and no longer gets the
  file under budget on its own. Settle the dead-property question *before*
  executing Seam C, not after.
