# crates/qbz-ui/ui/settings/AppearanceSettings.slint (845 lines)

## Summary
The full Appearance settings panel: theme picker + auto-theme + custom-theme
editor, typography/language, library & visuals toggles, notifications, window
title, title bar, window controls, player views, system tray/menu bar, and
renderer — a 1:1 structural replica of Tauri's Settings > Appearance, mostly
unwired UI rows bound to `AppearanceState`.

## Proposed split
By domain section, matching the file's own `// ====` banner comments
(THEME / TYPOGRAPHY & LANGUAGE / LIBRARY & VISUALS / NOTIFICATIONS / WINDOW
TITLE / TITLE BAR / WINDOW CONTROLS / PLAYER VIEWS / SYSTEM TRAY / RENDERER),
plus hoisting the small shared leaf components into their own file:

- `settings/appearance/shared.slint` (~100 lines) — lines 25-114: the small
  helper components used across sections: `GroupHeader`, `TokenGroupLabel`,
  `CustomTokenCell`, `Divider`, `SecondaryButton`. Exported so every section
  file below can import them.
- `settings/appearance/ThemeSection.slint` (~260 lines) — lines 127-376 (THEME
  group): theme select + filter cycle, dynamic-background toggle, intelligent
  search toggle, auto-theme (source/regenerate/detected-DE) rows, and the
  custom-theme editor grid + shared `ColorPicker`. This is the single biggest
  chunk and still over 130 on its own — split further:
  - `ThemeSection.slint` (~130 lines) — theme select/filter, dynamic bg,
    intelligent search, auto-theme rows (lines 127-258).
  - `CustomThemeEditor.slint` (~140 lines) — the "Start from current theme" /
    "Dark theme" rows + the full swatch grid + shared `ColorPicker` (lines
    260-376), instantiated as one child component from `AppearanceSettings`.
- `settings/appearance/TypographySection.slint` (~40 lines) — lines 387-414
  (TYPOGRAPHY & LANGUAGE): language select + interface-size select.
- `settings/appearance/LibraryVisualsSection.slint` (~130 lines) — lines
  425-541 (LIBRARY & VISUALS): nav-in-sidebar, compact-header-nav, My QBZ
  rename row (with its LineEdit + reset button), sidebar-collage toggle,
  local-library-track-artwork toggle.
- `settings/appearance/NotificationsSection.slint` (~25 lines) — lines
  552-572: in-app toasts + system notifications toggles.
- `settings/appearance/WindowTitleSection.slint` (~40 lines) — lines 581-622
  (including the large commented-out "Title template" block — keep the
  comment verbatim, it documents an intentionally-hidden feature per owner
  request).
- `settings/appearance/TitleBarSection.slint` (~35 lines) — lines 635-664:
  use-system-title-bar + hide-title-bar rows (both `!is-macos`-gated).
- `settings/appearance/WindowControlsSection.slint` (~35 lines) — lines
  673-702: wc-position select + show-window-controls toggle.
- `settings/appearance/PlayerViewsSection.slint` (~30 lines) — lines 711-736:
  volume-steppers toggle + startup-page select.
- `settings/appearance/TraySection.slint` (~100 lines) — lines 744-807:
  tray-enable, close-to-tray, macOS hide-dock-icon, tray-icon-variant select.
- `settings/appearance/RendererSection.slint` (~35 lines) — lines 809-842:
  renderer-backend + preferred-GPU selects (both `renderer-setting-visible`
  gated).
- `settings/AppearanceSettings.slint` (~60 lines) — becomes the thin
  orchestrator: imports every section above plus `shared.slint`, keeps the
  root `VerticalLayout`, the shared `tb-locked` property (used by both
  TitleBarSection-adjacent rows and WindowControlsSection — needs passing in
  as an `in property` on those two sections since it's derived from
  `AppearanceState` fields but computed once here), and the `Rectangle {
  height: 12px } Divider {} Rectangle { height: 12px }` separators between
  sections.

## Re-export surface
`settings/AppearanceSettings.slint`'s exported `AppearanceSettings` component
stays the only import surface — it's what the Settings shell/router already
imports (`import { AppearanceSettings } from "./settings/AppearanceSettings.slint";`
or similar). The new section files are internal implementation details, never
imported directly by anything outside this directory.

## Coupling / watch out
- `root.tb-locked` (line 122, `hide-title-bar || use-system-title-bar`) is
  read by rows in BOTH the future `WindowControlsSection` (wc-position,
  show-window-controls `enabled:` bindings) and conceptually related to
  `TitleBarSection`. Since it's a derived property, either recompute it
  independently inside `WindowControlsSection` from `AppearanceState` directly
  (simplest — avoids threading an `in property` across files) or pass it in
  explicitly; recomputing locally is less coupling and is the recommended
  approach.
- The custom-theme editor's `ColorPicker` binding (lines 357-375) is a long
  chained ternary keyed on `AppearanceState.custom-open-token` — if
  `CustomThemeEditor.slint` is split out, this ternary and the token-key
  strings used in every `CustomTokenCell` must stay byte-identical (they are
  the wire contract with the Rust-side `custom-set-token`/`custom-set-token-hex`
  handlers).
- Several sections are conditionally hidden entirely via `if
  AppearanceState.is-macos` / `if !AppearanceState.is-macos` /
  `if AppearanceState.renderer-setting-visible` at the GROUP HEADER level —
  when these become separate files, the `if` gating must move to how the
  orchestrator INSTANTIATES the section component (e.g.
  `if !AppearanceState.is-macos : TitleBarSection {}`), not disappear.
- `MyQbzBrandingState`, `SidebarState`, `ShellState`, `ToastState`,
  `UiFocusState` are all imported from `../state.slint` alongside
  `AppearanceState` — each section file that uses one of these needs its own
  matching import line; don't assume a blanket re-export.

## Verify after split
- `slint-viewer` (or the project's usual slint compile check) on
  `AppearanceSettings.slint` and each new section file.
- Full app build (`cargo build`) since Slint files are compiled into the
  binary — a broken import surfaces as a build error, not just a lint.
- Manually open Settings > Appearance and click through every section
  (theme picker, custom theme editor open/close, tray toggles, renderer
  selects) to confirm no row silently vanished or lost its callback wiring.
