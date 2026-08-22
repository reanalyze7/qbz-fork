# crates/qbz-ui/ui/settings/OfflineSettings.slint (167 lines)

## Summary
Settings > Offline panel: two groups — OFFLINE MODE (status row with
"Check now" re-probe, Enable Offline Mode toggle) and OFFLINE CACHE
(manage/open-folder/clear-all buttons for the download cache).

## Proposed split
This file is only modestly over budget (167 lines); extracting the two
small locally-defined helper components (`SecondaryButton`, already
duplicated near-verbatim across several settings panels per the file's own
comment "Matches AudioSettings/AppearanceSettings' SecondaryButton") is
enough to bring the root under 130 — and de-duplicates a component that
apparently exists in 3+ places today.

- `OfflineSettings.slint` (~115 lines) — module doc, imports, `export
  component OfflineSettings`: the `GroupHeader`/`Divider` trivial helpers
  (small enough to keep inline, ~10 lines total), the OFFLINE MODE group
  (status `SettingRow` + Enable-toggle `SettingRow`), and the OFFLINE CACHE
  group (3 `SettingRow`s), now referencing a shared `SecondaryButton`.
- `settings/shared/SecondaryButton.slint` (~35 lines, NEW SHARED FILE) —
  the `SecondaryButton` component (lines 39-70), promoted out of this file
  since the comment already admits it duplicates AudioSettings' and
  AppearanceSettings' own copies. **This is the one file in this chunk
  where the "split" is really a de-duplication opportunity, not just a
  line-count fix** — flagging for whichever agent/pass touches
  `AudioSettings.slint` / `AppearanceSettings.slint`, since they likely
  have byte-identical copies worth consolidating onto this same shared
  file rather than each keeping their own.

## Re-export surface
`OfflineSettings.slint` stays the single import surface for the Settings
page router (`import { OfflineSettings } from
"./settings/OfflineSettings.slint";` unaffected); it gains one new import
`import { SecondaryButton } from "./shared/SecondaryButton.slint";` (path
depends on where the shared file actually lands — if a `settings/shared/`
directory doesn't exist yet, create it, or place it directly in
`settings/` alongside its siblings).

## Coupling / watch out
- **Do NOT perform the `SecondaryButton` de-duplication unilaterally** —
  it touches files outside this chunk (`AudioSettings.slint`,
  `AppearanceSettings.slint`, possibly others). The safe, in-scope action
  for THIS file's split is: extract `SecondaryButton` to its own file
  under `settings/`, have `OfflineSettings.slint` import it, and leave the
  other panels' own copies untouched for now (their own refactor-plan
  agents, if any, can decide separately whether to point at this same
  shared file). Note this cross-file duplication in this report for other
  agents to see.
- `GroupHeader`/`Divider` are trivial (3-4 property lines each) and appear
  inline in many settings panels — not worth extracting to a shared file
  (extraction overhead > the lines saved); keep them inline in
  `OfflineSettings.slint`.
- The panel's `init => { OfflineModeActions.load(); }` re-seed hook and the
  `OfflineState`/`SettingsState` global reads are root-level concerns, no
  coupling risk from the `SecondaryButton` extraction.

## Verify after split
- `cargo build` through the Slint build step to confirm compilation.
- Smoke-test: open Settings > Offline, confirm the status row's tri-state
  text/color (online/no-connection/induced) and "Check now" button work,
  the Enable Offline Mode toggle persists, and the 3 cache buttons
  (manage/open-folder/clear-all) still fire their actions.
