# crates/qbz-ui/ui/primitives/EditPlaylistModal.slint (193 lines)

One-line summary: the "Edit playlist" rename/delete/offline-toggle modal — small overage, straightforward split.

## Proposed split
- `EditPlaylistModal.slint` (~130 lines) — **stays the public re-export** (`export component EditPlaylistModal`). Keeps the scrim, panel chrome, header, name/description inputs.
- `EditPlaylistFooter.slint` (~65 lines, new) — the offline-only toggle row (lines 97-132) + the Delete/Save footer buttons (lines 134-188). Takes `EditPlaylistState`/`EditPlaylistActions` (globals, no props needed) — can just be a plain extracted component with no properties at all since it reads globals directly.

## Tricky coupling to flag
- The hand-rolled checkbox (18px box, check icon, border toggling on `EditPlaylistState.offline-only`) is nearly identical to the one in `SettingsExportModal.slint`'s include-auth checkbox and `LibFolderEditModal`'s toggles — a shared `QbzCheckboxRow` primitive would remove this duplication project-wide, but is a separate follow-up, not required here.
- Save button's enabled/opacity logic (`EditPlaylistState.name != "" && !EditPlaylistState.busy`) is duplicated between the `opacity` binding and the `TouchArea`'s guard condition and `mouse-cursor` — keep both copies in sync if extracted.

## Verify after split
- Compiles; rename, delete, and offline-toggle (local playlists only) all still work; Enter-to-save from the name LineEdit (`accepted => { EditPlaylistActions.save(); }`) still fires.
