# crates/qbz-ui/ui/shell/ReportIssueModal.slint (142 lines)

## 1. Summary
The "Report an issue" modal (1:1 with the Tauri `ReportIssueModal.svelte`):
explains the manual redacted log-sharing flow and offers two actions — "Go
to logs" (opens the in-app log viewer) and "Create issue report" (opens the
GitHub new-issue page via a Rust callback).

## 2. Proposed module split
This file is only marginally over the 130-line limit (142 lines) and is a
single cohesive, small modal with no internally-reusable sub-pieces (unlike
`SongCard.slint` or `AlbumCollectionView.slint`, it has no repeated
structure or extractable helper component). The pragmatic split is to
extract the two action buttons (a repeated "pill button with hover state"
pattern already duplicated once per button) into a tiny shared component:

| New file | Owns | ~lines |
|---|---|---|
| `shell/ReportIssueModal.slint` | Stays the re-export/orchestrator: module doc, imports, `export component ReportIssueModal` — the backdrop, card shell, header (title + close X), and the intro `Text` block; composes the two extracted action buttons | ~95 |
| `primitives/ModalActionButton.slint` (or `shell/ReportIssueActionButton.slint` if this pattern is judged too Report-Issue-specific to promote to `primitives/`) | A small reusable pill button: `label` + `primary`/`secondary` color scheme + `clicked` callback, replacing the two near-identical `Rectangle { width: label.preferred-width + 28px; height: 36px; ... }` blocks (the "Go to logs" secondary button and "Create issue report" primary button) | ~45 |

If a shared button component is judged out of scope for a pure line-count
split (it does change a visual primitive used only here today), the
fallback is simply to extract the header block (title + close-X, lines
47–74) into its own tiny helper — but since this modal has almost no
reusable substructure, the action-button extraction is the more natural cut
and also reduces near-duplicate code, which is a bonus alignment with the
"clean folders" pure/IO/render principle (here: repeated-widget extraction).

## 3. Re-export / public API surface
`shell/ReportIssueModal.slint` remains the only file other `.slint` files
import (`export component ReportIssueModal`) — it takes no `in property`
today (all its state comes from the global `ReportIssueState`/
`ReportIssueActions`/`LogViewerState` singletons), so its call site in
`AppShell.slint` (mounted in declaration order per ADR-009, conditional on
`ReportIssueState.open` per ADR-010) needs zero edits.

## 4. Tricky coupling / shared-state to watch out for
- The modal reads/writes THREE global singletons directly
  (`ReportIssueState.open`, `ReportIssueActions.create-issue()`,
  `LogViewerState.open`) rather than taking properties/callbacks — if the
  action buttons are extracted into a shared component, keep those global
  writes in the CALLBACK HANDLERS inside `ReportIssueModal.slint` itself
  (i.e. the extracted button only exposes `clicked()`; `ReportIssueModal`
  wires `clicked => { ReportIssueState.open = false; LogViewerState.open =
  true; }` at the call site), not inside the shared button component — a
  generic `ModalActionButton` must not know about `ReportIssueState`.
- The backdrop's `TouchArea` (clicked => close) and the card's own
  passthrough `TouchArea { }` (swallowing clicks so they don't reach the
  backdrop) are a specific click-propagation pattern common to every modal
  in this codebase (ADR-009/ADR-010 area) — do not disturb this structure
  when reorganizing the file; it's the mechanism that makes "click outside
  to close" work.
- `AppShell.slint`'s declaration-order mounting (ADR-009) means this
  modal's position in `AppShell`'s child list matters for z-ordering versus
  other modals — a pure content-split within `ReportIssueModal.slint` (this
  refactor) does not touch that ordering, but worth a note for whoever does
  the actual split to not accidentally change import/mount order elsewhere.

## 5. What to verify after the real split
- `cargo build -p qbz-ui` (Slint compile-time check).
- Manual smoke test: open the hamburger menu > "Report an Issue", verify the
  modal opens, click the backdrop to close (should close), reopen, click
  "Go to logs" (should close this modal and open the log viewer), reopen,
  click "Create issue report" (should close this modal and trigger the
  GitHub new-issue page open via `ReportIssueActions.create-issue()`).
