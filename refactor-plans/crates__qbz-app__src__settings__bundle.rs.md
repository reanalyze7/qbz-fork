# crates/qbz-app/src/settings/bundle.rs (1444 lines)

## Summary
The settings-portability engine shared by `qbzd` (CLI) and the desktop
Settings modal: `export()` reads a profile's settings into a versioned JSON
`Bundle`, `plan()`/`replan_with_device()` classify every field against the
local system into applied/adapted/skipped buckets + a typed write list, and
`apply()` executes the writes against the daemon-root stores. By far the
biggest file in this batch (11x the limit) — needs a real by-domain split,
not just IO/pure.

## Proposed split
By domain, mirroring the file's own banner sections (types / export / plan /
apply / classification-helpers / apply-dispatch / store-readers / misc):

- `settings/bundle/mod.rs` (~40 lines) — module doc, `pub use` re-exports of
  every public item currently at `bundle::X` (types, `export`, `plan`,
  `replan_with_device`, `apply`, `default_filename`, `write_bundle_file`,
  `SCHEMA_VERSION`), plus `mod tests;` wiring (tests dir moves alongside).
- `settings/bundle/types.rs` (~180 lines) — lines 30-205: `SCHEMA_VERSION`,
  `Bundle`, `BundleSource`, `ProfilePaths`, `ExportSource`, `ExportOptions`,
  `ImportOptions`, `LiveSystem`, `PlanLine`, `DevicePick`, `DeviceChoice`,
  `ImportPlan`, `ImportReport`, `BundleError` + its `Display`/`Error` impls.
  Still a touch over — split `BundleError` + its impls (~35 lines) into
  `settings/bundle/error.rs` if the reviewer wants every file comfortably
  under 130.
- `settings/bundle/serde_impl.rs` (~70 lines) — lines 207-277: the `impl
  Bundle` block (`to_json_string`, `contains_secrets`, `parse`).
- `settings/bundle/export.rs` (~95 lines) — lines 279-373: `export()` and its
  direct helpers `desktop_paths`, `hostname`, `now_rfc3339`,
  `default_filename` (these last three are also used elsewhere in the file,
  so keep them here as the "misc" home and re-export from `mod.rs`).
- `settings/bundle/plan.rs` (~80 lines) — lines 375-454: `plan()`,
  `replan_with_device()`, `build_plan()` (the domain-dispatch match only —
  delegates to `plan_*` in `classify/`).
- `settings/bundle/apply.rs` (~90 lines) — lines 456-548: `apply()` itself
  (write grouping + per-store dispatch orchestration).
- `settings/bundle/classify/mod.rs` (~65 lines) — lines 550-615: shared
  helpers `skip_line`, `applied_line`, `applied_secret_line`, `adapted_line`,
  `render_value` + the `UNKNOWN_WHY`/`VOLUME_SKIP_WHY`/`CACHE_SKIP_WHY`
  constants, used by every `plan_*` fn below.
- `settings/bundle/classify/playback.rs` (~25 lines) — `plan_playback` +
  `PLAYBACK_KEYS` (lines 616-637).
- `settings/bundle/classify/prefs.rs` (~20 lines) — `plan_prefs` (639-657).
- `settings/bundle/classify/audio.rs` (~265 lines, STILL over 130 — split
  further) — lines 659-1009: `plan_audio`, `plan_quality_fallback`,
  `plan_audio_machine` (the big interdependent device/backend/intent/alsa/dsd
  block, ~265 lines alone), `pick_backend_name`, `backend_name`,
  `intent_flag_current`, `alsa_field_no_change`, `resolved_backend_is_alsa`,
  plus the `AUDIO_PORTABLE`/`AUDIO_INTENT_FLAGS`/`AUDIO_NEVER_CACHES` consts.
  Split into:
  - `classify/audio/mod.rs` (~50 lines) — consts + `plan_audio` orchestrator
    + `plan_quality_fallback`, `pub use machine::*`.
  - `classify/audio/machine.rs` (~200 lines, split again if reviewer
    insists) — `plan_audio_machine` (the single hardest-to-split function:
    device/backend/intent-flags/alsa/dsd all read each other's local
    variables like `fallback`, `device_survives`, `forced_device` — see
    coupling notes below) + its small helpers (`pick_backend_name`,
    `backend_name`, `intent_flag_current`, `alsa_field_no_change`,
    `resolved_backend_is_alsa`).
- `settings/bundle/classify/integrations.rs` (~35 lines) — `plan_integrations`
  + `SCROBBLER_PORTABLE`/`SCROBBLER_SECRET` (1012-1058).
- `settings/bundle/classify/library_folders.rs` (~10 lines) —
  `plan_library_folders` (1061-1069).
- `settings/bundle/classify/auth.rs` (~25 lines) — `plan_auth` (1072-1094).
- `settings/bundle/apply_writes.rs` (~120 lines) — lines 1098-1229:
  `apply_audio_writes`, `apply_playback_writes`, `apply_prefs_quality`,
  `apply_scrobbler_writes`, `persist_auth`, `as_bool` — the per-store apply
  dispatch, one match arm per settable field.
- `settings/bundle/readers.rs` (~130 lines) — lines 1230-1339:
  `read_audio_settings`, `read_playback_prefs`, `playback_to_json`,
  `read_scrobblers`, `read_library_folders`, `read_ui_prefs_streaming_quality`,
  `read_last_user_id`, `write_last_user_id` — side-effect-free store readers
  used by both `export` and `plan`.
- `settings/bundle/token.rs` (~40 lines) — lines 1341-1441:
  `load_decrypted_token`, `write_bundle_file` (0600-enforcing file writer) +
  the file-level doc comment context on why 0600 is load-bearing (04 §6).

## Re-export surface
`settings/bundle/mod.rs` stays the `crate::settings::bundle` module path —
every current caller (`qbzd` CLI commands, the desktop Settings > Import/
Export modal) already writes `bundle::export(...)`, `bundle::plan(...)`,
`bundle::Bundle`, etc., so `mod.rs` must `pub use` every symbol currently
public in this file (types, `export`, `plan`, `replan_with_device`, `apply`,
`default_filename`, `write_bundle_file`) so none of those call sites need a
single import path change.

## Coupling / watch out
- This is the single trickiest split in the whole batch: `plan_audio_machine`
  (lines 746-942, ~200 lines) has five interdependent decisions (device,
  backend, intent flags, alsa fields, dsd_mode) that all read `fallback`,
  `device_survives`, `resolved_device`, and `forced_device` computed earlier
  in the SAME function. Do not split this function across files — it is one
  cohesive state machine and must stay together (its own ~200-line file is
  the practical floor, even if slightly over 130; flag to the human reviewer
  rather than force a fracture that breaks the invariant chain).
- The file's own header comment states the CLASSIFICATION-LIVES-IN-THE-
  IMPORTER invariant (04 §1) — whichever file ends up owning `plan_*` /
  `build_plan` dispatch must keep this doc comment, since it is the
  reasoning for why classification can't be hoisted into `export()` or the
  `Bundle` type.
- `#[cfg(test)] mod tests;` (line 1443) points at an external `tests/` file
  (likely `bundle/tests.rs` today) — check whether that test file itself
  needs updating for new internal `super::` paths once functions move
  between new submodules (e.g. a test calling `super::plan_audio_machine`
  directly would need `super::classify::audio::machine::plan_audio_machine`).
- `AudioSettingsStore`, `PlaybackPreferencesStore`, `ScrobblerSettingsStore`
  (from `qbz_audio`/`crate::settings::*`) are used across export, plan, AND
  apply — keep their `use` statements consistent across whichever files call
  them; do not duplicate incompatible aliases.
- `SCHEMA_VERSION` const is read by both `export()` (in `export.rs`) and
  `build_plan()`'s version gate (in `plan.rs`) — keep it in `types.rs` and
  import from there in both.

## Verify after split
- `cargo check -p qbz-app` and `cargo build -p qbz-app`.
- `cargo test -p qbz-app settings::bundle` — the inline test module is
  explicitly called out as "the normative test suite" (one test per §3/§5
  rule), so it must stay green and ideally unmodified except for `use` paths.
- Smoke-test the `qbzd` export/import CLI path and the desktop Settings
  modal's Import/Export button end-to-end against a real bundle file.
