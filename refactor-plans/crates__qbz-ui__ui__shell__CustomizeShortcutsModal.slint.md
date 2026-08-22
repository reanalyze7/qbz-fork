# crates/qbz-ui/ui/shell/CustomizeShortcutsModal.slint (366 lines)

## Summary
The editable "Customize Shortcuts" modal (1:1 port of the Tauri
`KeybindingsSettings.svelte` + `ShortcutInput.svelte`): a self-gated overlay
with a scrim, an Esc-aware `FocusScope`, a header (title + modified-count
badge + close-x), a three-column scrollable body of category groups with
per-row keycap capture buttons and conflict messages, and a footer
"Reset All to Defaults" button.

## Proposed split
Split into three Slint files by component boundary (this file already has
two natural components: the reusable `KbEditGroup` row-group, and the modal
shell):

- `CustomizeShortcutsModal.slint` (~230 lines) — becomes the re-export/entry
  surface: the `export component CustomizeShortcutsModal`, its scrim +
  `FocusScope` + `card` shell, the header (title/badge/close-x), the
  three-column `Flickable` body (importing `KbEditGroup` from the new
  file), and the footer reset-all button. Still slightly over 130 on its
  own — extract the header and footer too if a hard 130 cap is enforced:
  - `shell/shortcuts/ShortcutsHeader.slint` (~60 lines) — title + "N
    modified" badge + close-x, taking `modified-count` in and emitting a
    `close()` callback.
  - `shell/shortcuts/ResetAllButton.slint` (~45 lines) — the footer
    reset-all button, taking `active: bool` in and emitting a `clicked()`
    callback.
  - That leaves `CustomizeShortcutsModal.slint` at roughly 130-150 lines:
    scrim, FocusScope/key-handling, card shell, the Flickable's three
    `VerticalLayout` columns (each just a `for group in
    KeybindingsState.groups-colN: KbEditGroup { group: group; }`), and
    wiring the header/footer callbacks back to `KeybindingsState`/
    `KeybindingsActions`.
- `shell/shortcuts/KbEditGroup.slint` (~115 lines) — the `KbEditGroup`
  component verbatim (category header + the `for entry in group.rows`
  capture-row block with its keycap button, per-row reset, and conflict
  message).

## Re-export surface
`CustomizeShortcutsModal.slint`'s `export component
CustomizeShortcutsModal` stays the only import path anything outside
`shell/` uses (wherever `AppShell.slint` or the settings host does
`import { CustomizeShortcutsModal } from "shell/CustomizeShortcutsModal.slint";`)
— unchanged. `KbEditGroup`, `ShortcutsHeader`, `ResetAllButton` become
internal-only imports used solely from within `CustomizeShortcutsModal.slint`.

## Coupling / watch out
- `KbEditGroup` currently reads `KeybindingsState.recording-id` /
  `KeybindingsState.pending-display` / `KeybindingsState.conflict-label`
  and calls `KeybindingsActions.start-record`/`reset-one` directly (it does
  NOT take these as properties/callbacks from its parent) — when extracted
  to its own file it needs its own `import { KeybindingsState,
  KeybindingsActions, KeybindingCategoryGroup } from "../state.slint";`
  (or `"../../state.slint"` depending on the new directory depth) rather
  than relying on the parent's import.
  order the row buttons over the modified/recording border logic; the
  `entry: modified` styling used both by the border-width and the reset-row
  gate must stay exactly as written.
- The `focus-timer` deferred-mount-focus idiom (30ms Timer that calls
  `keys.focus()` once) is noted as "same idiom as the PlaylistPickerModal"
  — do not simplify or remove it during the split; it is a documented
  workaround for a real focus race.
- The `key-pressed(event)` handler on the `FocusScope` (Escape cancels
  recording else closes the modal) must remain on the same node as
  `focus-timer`/`keys.focus()` — keep both in the surface file, not moved
  into a sub-component.

## Verify after split
- Build the Slint UI (`cargo build -p qbz-ui`) to confirm every extracted
  component compiles and every callback/global import resolves.
- Visual smoke-test: open Settings → Customize Shortcuts, click a keycap to
  enter recording mode, verify the conflict message appears/disappears
  correctly, use per-row reset and "Reset All to Defaults", and confirm
  Escape cancels an in-progress recording before closing the modal on a
  second Escape.
