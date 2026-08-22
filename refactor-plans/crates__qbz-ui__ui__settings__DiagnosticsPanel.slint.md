# crates/qbz-ui/ui/settings/DiagnosticsPanel.slint (302 lines)

## Summary
Settings > Developer's inline Diagnostics panel (1:1 port of the Tauri
`DiagnosticsPanel.svelte`): a master collapsible holding seven saved-vs-
runtime sub-sections (System / Playback / Qobuz Connect / Audio / Graphics /
Environment), five 3-column and two 4-column (Audio, Graphics — gated by
per-section `show-saved`), driven by the `DiagnosticsState` global.

## Proposed split
By component — a `settings/diagnostics/` sibling directory, each existing
inline component its own file, composed by a slimmed `DiagnosticsPanel.slint`.

- `settings/diagnostics/diag_row_view.slint` (~50 lines) — `DiagRowView`, one
  table row (label / optional saved / runtime / status glyph).
- `settings/diagnostics/diag_section.slint` (~90 lines) — `DiagSection`, the
  collapsible section (header + column headers + `for r in rows: DiagRowView`).
  Imports `DiagRowView` from the sibling file above.
- `settings/diagnostics/diag_button.slint` (~35 lines) — `DiagButton`, the
  small auto-width action button (Refresh / Export to clipboard).
- `settings/DiagnosticsPanel.slint` (~135 lines) — the slimmed main export:
  master toggle header, the Refresh/Export action row (using `DiagButton`),
  the error text, and the version line + five `DiagSection` instances (one
  per row-model: system/playback/audio/graphics/env — note the doc comment
  says seven sections but only five `DiagSection` instances currently exist
  in the body; carry that discrepancy forward verbatim, don't "fix" it
  during a pure split).

## Re-export surface
`settings/DiagnosticsPanel.slint` stays the file other `.slint` imports
(`import { DiagnosticsPanel } from "../settings/DiagnosticsPanel.slint"`) —
unchanged export name, now a thin composition of the three new sibling
files.

## Coupling / watch out
- `DiagRow` and `DiagnosticsState` are both imported from `../state.slint`
  — `diag_row_view.slint` only needs `DiagRow`; `diag_section.slint` needs
  both `DiagRow` (for its `in property <[DiagRow]> rows`) and nothing from
  `DiagnosticsState` directly (it's the main panel that reads
  `DiagnosticsState.*-rows`); only `DiagnosticsPanel.slint` itself needs
  `DiagnosticsState` for `loaded`/`error`/`copied`/`app-version`/`refresh()`/
  `export-clipboard()`.
- `DiagSection`'s `open` is a `private property` seeded from `default-open`
  — this is genuinely private per-instance UI state (no Rust round-trip),
  so nothing needs to change here; just don't accidentally promote it to an
  `in-out property` during the split.
- `DiagButton`'s `enabled` gating (Export button disabled until
  `DiagnosticsState.loaded`) stays wired from the main panel file, not
  inside `DiagButton` itself (which only knows about styling, not the
  diagnostics-loaded state) — no change needed here, just noting the
  boundary.
- Column-header widths (`width: 38%` for the Setting column, the `Rectangle`
  spacer for the glyph column) are duplicated between `DiagSection`'s header
  block and `DiagRowView`'s row layout — they must stay numerically
  consistent (both files use the same `38%`/`20px`) since Slint has no
  shared-constant import for this without a small shared module; consider a
  documented duplication note in both files' comments rather than silently
  diverging.

## Verify after split
- Slint compile check for the crate.
- Manual smoke test in Settings > Developer: expand the master Diagnostics
  toggle (confirm first-expand triggers `DiagnosticsState.refresh()` when
  not yet loaded), expand/collapse each of the five sections, confirm
  Audio/Graphics show the Saved column and the others don't, confirm the
  status glyph (·/✓/✗) renders correctly per row, and confirm
  Refresh/Export-to-clipboard buttons work (Export disabled until loaded,
  label flips to "Exported").
- Grep for `DiagnosticsPanel` usage to confirm its import path is unaffected.
