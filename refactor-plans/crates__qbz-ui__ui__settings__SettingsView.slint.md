# crates/qbz-ui/ui/settings/SettingsView.slint (317 lines)

## Summary
The Settings page shell: header (back button + title), a left sub-navigation
column (9 sections: Audio/Playback/Appearance/Offline/Local Library/Blacklist/
Integrations/Developer/optional Flatpak-or-Snap), and a scrollable right panel
that conditionally instantiates the active section's settings component,
forwarding its callbacks up to Rust.

## Proposed split
Slint components split cleanly along "component boundary" lines — extract the
two visually/behaviorally distinct pieces (`SubNavItem` and the sub-nav column
itself) into their own files, leaving `SettingsView` as the thin composing
shell:

- `settings/SettingsSubNavItem.slint` (~45 lines) — the `SubNavItem` component
  (26-66): one clickable nav-row (icon + label + active/hover styling). Fully
  self-contained already (in/callback properties only), trivial to extract
  verbatim.
- `settings/SettingsSubNav.slint` (~110 lines) — the entire left sub-nav column
  (109-217): the `Rectangle { width: 232px; VerticalLayout { ... } }` block
  containing all 9 `SubNavItem` instances bound to `SettingsState.section`,
  plus the "Share logs" always-visible entry. Exposed as its own component
  (e.g. `component SettingsSubNav inherits Rectangle { ... }`) with no
  additional properties needed — it reads `SettingsState`/`SandboxState`/
  `LogViewerState` directly (globals), so it drops into `SettingsView` as a
  single `SettingsSubNav { }` instantiation.
- `settings/SettingsView.slint` (~165 lines) — becomes the re-export/composition
  root: imports `SettingsSubNavItem` (only if still referenced directly — after
  extraction it won't be, since `SettingsSubNav` owns it) and `SettingsSubNav`,
  keeps the header block (81-103), the `HorizontalLayout` wrapping
  `SettingsSubNav { }` + the active-panel `Flickable`/`ScrollView`-replacement
  logic (225-314) with its `sr-restore()` scroll-position-restore function and
  the big `if SettingsState.section == N: XSettings { ... }` chain
  (254-302) forwarding `settings-bool`/`settings-select`/`settings-slider`/
  `settings-string`/`settings-reset`/`settings-release-device` callbacks.
  This remains the file other code imports (`SettingsView` name unchanged).

## Re-export surface
`SettingsView.slint` stays the single import surface — `export component
SettingsView` keeps its name, its 6 callbacks (`settings-bool`,
`settings-select`, `settings-slider`, `settings-string`, `settings-reset`,
`settings-release-device`) are unchanged, so whatever `.slint` file instantiates
`SettingsView { settings-bool(k,v) => {...} ... }` today needs no edits.
`SettingsSubNavItem` and `SettingsSubNav` are new internal components — only
`SettingsView.slint` needs to `import` them; no other file should need to.

## Coupling / watch out
- The active-panel `Flickable`'s scroll-restore logic (`sr-armed`,
  `sr-restore()`, the `changed viewport-height =>` / `changed viewport-y =>`
  handlers at 230-240) is wired to `NavState.restore-scope == "settings"` — this
  is page-identity-sensitive code; keep it inside `SettingsView.slint` itself
  (not the extracted sub-nav) since it concerns the RIGHT panel, not navigation.
- `SandboxState.install-method` conditionally renders EITHER the Flatpak OR
  Snap sub-nav item (never both) inside what becomes `SettingsSubNav` — verify
  after extraction that both `if` branches still reference `SettingsState.section
  == 8` (both write to the same section index; a copy-paste split error could
  easily point one branch elsewhere).
- Section index literals (0 through 8) are the ONLY thing linking a
  `SubNavItem`'s `active`/`clicked` binding in `SettingsSubNav` to the
  corresponding `if SettingsState.section == N: XSettings {}` conditional in
  `SettingsView`'s right panel — these two lists must be kept in sync manually
  across the two files; consider a code comment in both listing the full
  section-index table as a cross-reference.
- The `ListScrollbar` overlay (305-313) binds directly to the `settings-flick`
  id declared inside the panel `Rectangle` — this coupling is unaffected by the
  sub-nav extraction (both stay in `SettingsView.slint`) but confirm the `id`
  isn't accidentally renamed.

## Verify after split
- `cargo build -p qbz-ui` (or however the Slint build step is invoked —
  check for a `build.rs` slint-build step) to confirm the `.slint` files still
  compile with the new imports.
- Visual/smoke check: open Settings, click through all 9 (or 10, with
  Flatpak/Snap) sub-nav entries, confirm the correct panel renders and the
  scroll-position restore still works when navigating away and back.
- `grep -rn "SettingsView {" crates/` to confirm no other `.slint` file
  constructs `SettingsSubNavItem` or `SettingsSubNav` directly (they should be
  implementation details of `SettingsView` only).
