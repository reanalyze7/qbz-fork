# crates/qbz-app/src/settings/bundle/tests.rs (527 lines)

## Summary
The normative test suite for the settings-portability engine
(`qbz_app::settings::bundle`) — per the file's own header comment, "the
classification table IS the test suite": one test per §3/§5 classification
rule from `04-settings-portability.md`. Already split out of `bundle.rs`
into its own file; that file alone is still over budget and needs a
second-level split by test theme.

## Proposed split
Keep this as `bundle/tests.rs`'s own child module tree (`bundle/tests/`),
grouped by the rule area each test targets:
- `bundle/tests/mod.rs` (~85 lines) — shared fixtures used by every group:
  `scratch`, `cleanup`, `live`, `bundle_with`, `find`, `find_contains`,
  `write_of` (lines 11-67), plus `mod classification; mod secrets; mod
  device; mod persistence;` declarations (each test submodule does `use
  super::*;` to reach the fixtures).
- `bundle/tests/classification.rs` (~90 lines) — the §3 field-
  classification rules: `portable_fields_apply_verbatim`,
  `volume_is_never_class_even_hand_added`, `dsd_downgrades_without_trust_flag`,
  `ask_maps_to_always_fallback_in_adapted`, `unknown_field_skipped_never_error`
  (lines 69-173).
- `bundle/tests/secrets.rs` (~110 lines) — the secrets double-gate rules:
  `secrets_double_gate`, `secret_applies_with_gate`,
  `secret_values_never_render_in_summary`,
  `contains_secrets_keys_on_actual_secret_values` (lines 174-222, 459-513
  — note these are NOT contiguous in the current file; the implementer
  should re-verify exact line ranges and may prefer to keep this group's
  ordering as-is rather than force contiguity).
- `bundle/tests/device.rs` (~95 lines) — device-pick / version-gate rules:
  `version_gate_rejects_newer`, `missing_device_non_tty_falls_back_safe`,
  `found_device_applies_verbatim`, `device_pick_names_the_backend` (lines
  222-297, 514-527).
- `bundle/tests/persistence.rs` (~140 lines) — apply/roundtrip/persistence
  rules: `absent_fields_leave_target_untouched`,
  `machine_caches_always_skipped`, `roundtrip_same_box_is_noop`,
  `library_folders_skipped_on_daemon`,
  `apply_writes_are_idempotent_and_persist`, `bundle_json_roundtrips_flat`,
  `parse_rejects_missing_version` (lines 297-458).

The implementer should do one careful top-to-bottom pass to get exact,
non-overlapping line ranges (this pass only sampled function names via
grep) — the groupings above are a thematic starting point, not a verified
line-accurate cut.

## Re-export surface
This is a `#[cfg(test)]`-only module tree with no public API — nothing
needs re-exporting. `bundle.rs` (the parent, non-test file) just needs its
`#[cfg(test)] mod tests;` declaration to keep working whether `tests.rs`
becomes `tests/mod.rs` or stays a single re-exporting file that declares
the sub-modules.

## Coupling / watch out
- Every test does `use super::*;` to reach both the shared fixtures AND
  the engine's own public items (`Bundle`, `ImportPlan`, `PlanLine`,
  `LiveSystem`, `ProfilePaths`, etc. from `bundle.rs`/`mod.rs`) — after
  the split, each submodule needs `use super::super::*;` (engine) plus
  `use super::*;` (shared fixtures in `tests/mod.rs`), or a single `use
  crate::settings::bundle::*; use super::*;` — get this import chain
  right or every test in every submodule fails to compile at once.
- The file's own header comment ("the classification table IS the test
  suite") is a project-level testing philosophy note — keep it on
  whichever file becomes the entry point (`tests/mod.rs`).
- `bundle_with(domains: serde_json::Value)` and `find`/`find_contains`/
  `write_of` are used across essentially every test — these must live in
  the shared fixtures file, not get scattered into whichever test happens
  to use them first.

## Verify after split
- `cargo test -p qbz-app settings::bundle` green — same test count and
  names as before (no test silently dropped or renamed during the
  reorganization).
- `cargo build -p qbz-app`.
