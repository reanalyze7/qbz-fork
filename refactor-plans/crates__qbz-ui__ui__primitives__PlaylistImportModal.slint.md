# crates/qbz-ui/ui/primitives/PlaylistImportModal.slint (562 lines)

## Summary
The "Import Playlist" modal: URL entry -> provider auto-detect -> fetch
preview -> rename/folder customization -> import with progress bar, status
line, append-only log, and a completion Summary block, all driven by
`PlaylistImportState`/`PlaylistImportActions`.

## Proposed split
By region within the single modal (header/body-sections/footer), since this
is one cohesive dialog flow rather than several independent components:

- `primitives/playlist_import/UrlEntryPanel.slint` (~140 lines) — lines
  127-288: the offline banner, error banner, URL `TextInput` box (with
  placeholder-text workaround), and the "ALLOWED SOURCES" provider-logo row
  (Spotify/Apple/Tidal/Deezer with the opacity-homologation styling
  documented in the file's header comment — keep that comment attached here,
  it explains why there's no grayscale filter).
- `primitives/playlist_import/CustomizePanel.slint` (~80 lines) — lines
  293-370: the step-B rename input + optional folder `QbzSelect`, shown only
  when `show-preview` is true.
- `primitives/playlist_import/ProgressPanel.slint` (~150 lines) — lines
  375-517: the progress header (spinner), determinate bar + status/
  current-track lines, the append-only log `Flickable` (with its "POST-LAYOUT
  no autoscroll" comment — keep it, documents deliberate Tauri parity), and
  the completion Summary block.
- `primitives/PlaylistImportModal.slint` (~190 lines) — becomes the shell:
  imports the three panels above, keeps the outer scrim + `TouchArea`, the
  card `Rectangle` sizing math (`Math.min`/`Math.max` height/width formulas —
  these are load-bearing layout constants, do not move them into a sub-panel),
  the header (logo + title + close-x), the footer (Close + the single
  Fetch/Import `QbzPrimaryButton` whose label/enabled ternaries read both
  `show-preview` and `loading`/`import-completed`), and the outer
  `body-flick := Flickable` wrapping the three panels.

## Re-export surface
`primitives/PlaylistImportModal.slint`'s exported `PlaylistImportModal`
component remains the only import surface — the shell/router already imports
it by this path; the three new panel files are internal, never imported
elsewhere.

## Coupling / watch out
- All state lives in the two globals `PlaylistImportState` /
  `PlaylistImportActions` (from `../state.slint`) — every new panel file
  needs its own `import { PlaylistImportState, PlaylistImportActions, ... }
  from "../state.slint";` line; there's no per-panel local state to worry
  about losing.
- `OfflineState` and `UiFocusState` are each used in exactly one place (the
  offline banner in UrlEntryPanel; the two `changed has-focus` guard-focus
  probes in UrlEntryPanel and CustomizePanel) — make sure both panel files
  import what they individually need rather than assuming the shell's imports
  cascade (Slint imports are per-file).
- The footer's confirm button behavior (step A "Fetch" vs step B "Import")
  branches on `PlaylistImportState.show-preview`, which is also the gate that
  shows/hides `CustomizePanel` and hides the URL-entry allowed-sources
  section — if the shell is split from the panels, keep this single ternary
  in the shell (footer) rather than duplicating the branch logic into a panel.
- The card's `height:` binds to `body.preferred-height` (line 52) — this
  requires the inner `body := VerticalLayout` to still exist as a named
  element the shell can reference for sizing; if `body` gets nested one level
  deeper inside sub-panel components, `preferred-height` propagation through
  Slint's layout system should still work (VerticalLayout wraps sum of
  children's preferred heights) but must be verified visually since a modal
  cut short/too tall is an easy regression here.
- CJK/femtovg note in the header comment (fixed width+height, `overflow:
  elide`, never intrinsic sizing) applies to every `Text` across all three
  panels — do not let any moved `Text` lose its explicit `height:` when
  relocated.

## Verify after split
- `slint-viewer` / project's slint compile check on all four files.
- Full app build.
- Manual smoke-test: paste a Spotify/Apple/Tidal/Deezer playlist URL, confirm
  provider logo highlight, fetch preview, rename + pick a folder, run an
  import and confirm progress bar/log/summary all still render, then verify
  Close works both mid-import and post-completion.
