# crates/qbz-ui/ui/primitives/DacWizardModal.slint (861 lines)

## Summary
The 6-step HiFi Wizard (DAC setup) modal: scrim + card + stepper rail + a
per-step body (welcome / check audio stack / select DACs / review & apply /
test playback / done) + footer nav, all driven inline by mutating
`DacWizardState` (no Rust round-trip for navigation) via `DacWizardActions`
for the heavier operations (detect, gen-configs, copy-command, test).

## Proposed split
By wizard step — a `primitives/dac_wizard/` sibling directory, each step its
own component, composed by a slimmed `DacWizardModal.slint`.

- `dac_wizard/check_row.slint` (~55 lines) — `CheckRow`, the whole-row
  clickable checkbox shared by step 0 (welcome) and step 3 (review: backup/
  config/restart confirmations).
- `dac_wizard/step_welcome.slint` (~45 lines) — new component
  `DacStepWelcome` (current step-0 block, lines 199-228): intro text +
  `WarningBanner` + `CheckRow` bound to `DacWizardState.welcome-confirmed`.
- `dac_wizard/step_check.slint` (~95 lines) — new component `DacStepCheck`
  (step 1, lines 232-302): sandboxed/health `WarningBanner`s, distro/init
  `QbzSelect`s, remediation `CommandBlock`s.
- `dac_wizard/step_select_dacs.slint` (~175 lines) — new component
  `DacStepSelectDacs` (step 2, lines 304-468): enumerated candidate list +
  escape-hatch toggle + manual node-name entry. Still a bit large; if it
  runs over after extraction, split further into
  `dac_wizard/dac_candidate_row.slint` (the per-candidate row, ~70 lines)
  and keep the manual-entry block + escape-hatch toggle in
  `step_select_dacs.slint` (~110 lines).
- `dac_wizard/step_review.slint` (~110 lines) — new component
  `DacStepReview` (step 3, lines 470-575): backup/per-DAC accordion configs/
  restart, using `CheckRow` + `CommandBlock`.
- `dac_wizard/step_test.slint` (~175 lines) — new component `DacStepTest`
  (step 4, lines 577-748): the 4 curated test tracks, play/stop/prev/next
  controls, live rate read-back, and the polling `Timer`. If still over
  budget, split the track-list block into `dac_wizard/dac_test_track_list.slint`
  (~60 lines) and keep controls + readout + timer in `step_test.slint`
  (~115 lines).
- `dac_wizard/step_done.slint` (~45 lines) — new component `DacStepDone`
  (step 5, lines 750-791): success icon/text + created-paths list.
- `primitives/DacWizardModal.slint` (~180 lines) — the slimmed main export:
  scrim + card + header (wand icon/title/close-x) + stepper rail (steps
  array + progress dots) + the body `Flickable`/`ListScrollbar` switching on
  `DacWizardState.step` to mount one of the 6 step components + footer
  (Back/Close/primary button with its per-step `enabled` expression).

## Re-export surface
`primitives/DacWizardModal.slint` stays the single import surface
(`import { DacWizardModal } from "../primitives/DacWizardModal.slint"`) for
whichever shell file mounts it — unchanged export name.

## Coupling / watch out
- All 6 step components read `DacWizardState`/`DacWizardActions` (and
  `NowPlayingState`, `UiFocusState`) as globals directly, so they need no
  props threaded in from the main modal — same pattern as the state-heavy
  Playlist Manager split. Just import the globals in each new file.
- `root.show-manual` (private property, step 2 only) moves into
  `DacStepSelectDacs` as its own private property — it's not read anywhere
  else.
- `root.test-tracks` (the 4 curated-track labels, step 4 only) moves into
  `DacStepTest` as a local property — not read elsewhere.
- `root.steps` (step labels for the header rail) stays in the main
  `DacWizardModal.slint` since the stepper rail lives there, not in a step
  component.
- The primary footer button's `enabled` expression currently switches on
  `DacWizardState.step` to check step-specific conditions
  (`welcome-confirmed`, `any-dac-selected || manual-valid`,
  `review-backup-done && review-config-done && review-restart-done`) — this
  stays in the main modal file (it's footer logic, not step-body logic), but
  note it duplicates knowledge of what each step considers "complete"; a
  future cleanup could expose a per-step `is-valid` bindable property
  instead, though that's beyond a pure file-split.
- Entering step 1→2 triggers `DacWizardActions.run-detect()` and entering
  step 2→3 triggers `DacWizardActions.gen-configs()` — this side-effect
  wiring lives in the footer's `clicked` handler in the main modal, not in
  the step components; keep it there since it's a transition action, not
  step-body rendering.
- `CheckRow` is imported by both `step_welcome.slint` and `step_review.slint`
  — make sure both import paths point to the same `dac_wizard/check_row.slint`.

## Verify after split
- Slint compile check for the crate.
- Walk the full wizard start-to-finish: welcome checkbox → check-audio-stack
  distro/init selects + remediation copy → DAC candidate multi-select +
  manual entry fallback → review accordions + backup/config/restart
  checkboxes → test playback (play/stop/prev/next, track click-to-jump,
  live read-back polling) → done screen.
- Confirm Back/Close/primary button enabled-state and click behavior is
  unchanged at every step boundary.
- Grep for `DacWizardModal` usage to confirm its import path is unaffected.
