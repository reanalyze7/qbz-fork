# crates/qbz-ui/ui/primitives/TagEditorModal.slint (540 lines)

One-line summary: the local-album metadata editor modal — header, album/year/genre fields, remote MusicBrainz/Discogs lookup panel, per-track table, and footer all in one file.

## Proposed split
- `TagEditorModal.slint` (~150 lines) — **stays the public re-export** (`export component TagEditorModal`). Keeps the scrim/backdrop, panel chrome, header (title + close-X), and footer (persistence QbzSelect + Cancel/Save), and composes the three pieces below. The private `Field` helper component (lines 19-29) stays here since the footer's QbzSelect doesn't need it but album fields do — or move `Field` into its own tiny file if both consumers need it (see below).
- `TagEditorFields.slint` (~90 lines, new) — the album title/artist/year/genre/catalog `Field` blocks (lines 91-171), including the shared `Field` component.
- `TagEditorRemoteLookup.slint` (~200 lines, new) — the whole remote-metadata block: provider QbzSelect + search button + no-results note + result cards + Apply/open-in-browser row (lines 173-363).
- `TagEditorTrackTable.slint` (~70 lines, new) — the per-track LineEdit table (lines 365-431).

## Tricky coupling to flag
- Every field binds directly to `TagEditorState`/`TagEditorActions` globals — splitting into sub-components is safe since Slint globals are ambient, no prop threading needed, but each new file must import them from `../state.slint`.
- The repeated "hotkey-guard probe" pattern (`guard-focused` local property mirrored into `UiFocusState.text-input-focused`) appears on every `LineEdit` in this file (7+ times) — flag as a candidate for a shared wrapped `QbzLineEdit` primitive in a follow-up, not required for this split.
- `TagEditorState.tracks[i].*` array-index writes in the track table's `edited` callbacks must keep referencing the outer `for row[i] in TagEditorState.tracks` loop variable — this is why the track table is a natural standalone component (self-contained loop).

## Verify after split
- Compiles; modal still `visible: TagEditorState.open`-gated in AppShell mount order (ADR-009/010) unaffected.
- Manual smoke test: open tag editor, edit album fields, search remote, apply a result, edit a track row, save — all still work end-to-end.
