# crates/qbz-audio/src/settings.rs (1056 lines)

## Summary
Audio settings persistence: the `AudioSettings` data struct (~25 fields covering
backend/device/quality/normalization/gapless/DSD/crossfade prefs) plus a SQLite-
backed `AudioSettingsStore` (schema creation, migrations, getters/setters,
reset_all) and a thread-safe `AudioSettingsState` wrapper, with ~215 lines of
unit tests.

## Proposed split
By pure/IO/render-adjacent responsibility (there's no rendering here, so it's
pure-data vs IO vs thread-safety-wrapper):

- `settings/mod.rs` (~15 lines) — module doc, re-exports
  (`pub use types::AudioSettings; pub use store::AudioSettingsStore; pub use
  state::AudioSettingsState;`).
- `settings/types.rs` (~140 lines) — the `AudioSettings` struct definition (all
  fields + doc comments, lines 15-90), `default_dsd_mode()`, and `impl Default
  for AudioSettings` (96-136). Pure data, no IO. Slightly over 130 with all the
  field doc comments; trim by moving `impl Default` to its own ~40-line
  `settings/defaults.rs` if needed.
- `settings/schema.rs` (~130 lines) — `AudioSettingsStore::open_at`'s
  table-creation + all the `ALTER TABLE ... ADD COLUMN` migration statements and
  the one-time `limit_quality_to_device` backfill (lines 143-297). This is pure
  "migration IO" and is the single largest chunk (~155 lines as-is) — keep as
  its own file since it's a cohesive "schema evolution" concern separate from
  the getter/setter CRUD below.
- `settings/store_core.rs` (~60 lines) — `AudioSettingsStore` struct, `new()`,
  `new_at()`, `get_settings()` (299-368) — the single big SELECT/row-mapper.
- `settings/store_setters.rs` (~230 lines) — the large run of small `set_*`
  one-liner setters (370-704: output_device, exclusive_mode, dac_passthrough,
  sample_rate, backend_type + alsa_plugin cross-set, alsa_plugin,
  alsa_hardware_volume, stream_first_track, stream_buffer_seconds,
  streaming_only, limit_quality_to_device, device_max_sample_rate,
  device_sample_rate_limit(s) get/set, normalization_enabled, crossfade_seconds,
  gapless_enabled, allow_quality_fallback, skip_sink_switch,
  reserve_dac_while_running, dsd_mode, pw_force_bitperfect,
  sync_audio_on_startup, quality_fallback_behavior get/set,
  normalization_target_lufs). Still ~230 lines — split further into
  `store_setters_output.rs` (device/backend/alsa/sample-rate, ~110 lines) and
  `store_setters_playback.rs` (streaming/normalization/gapless/dsd/crossfade/
  fallback-behavior, ~120 lines).
- `settings/reset.rs` (~90 lines) — `reset_all()` (706-796): the big multi-column
  UPDATE plus the ADR-003 quality_fallback_behavior save/restore dance.
- `settings/state.rs` (~45 lines) — `AudioSettingsState` (thread-safe
  `Arc<Mutex<Option<AudioSettingsStore>>>` wrapper): new/new_empty/init_at/
  teardown/Default (799-841).
- `settings/tests.rs` (~215 lines) — the entire `#[cfg(test)] mod tests` block
  (843-1056): default-values, store-returns-defaults, backend-null-stays-auto,
  alsa-plugin-default-on-switch, buffer-seconds-clamp, invalid-fallback-value,
  reset-preserves-quality-fallback, persist-and-reopen-new-fields, legacy-JSON
  deserialize. Kept as one file since these are integration-style tests that
  exercise the store end-to-end across schema/setters/reset; could later be
  split per-concern but not required to hit 130 (some individual test fns are
  ~25-30 lines, ~8 tests fits ~215 — still needs one more split, e.g.
  `tests_migration.rs` for backend-null/alsa-default/legacy-JSON and
  `tests_crud.rs` for the rest).

## Re-export surface
`settings/mod.rs` re-exports `AudioSettings`, `AudioSettingsStore`,
`AudioSettingsState` at `crate::settings::*` (this crate's lib.rs already does
`pub use settings::...` or similar — verify and keep that top-level re-export
unchanged so `qbz_audio::AudioSettings` etc. keep resolving for `qbz-nix`'s
Tauri command wrappers, per the file's own doc comment: "NOTE: Tauri command
wrappers remain in qbz-nix").

## Coupling / watch out
- `AudioSettingsStore` methods read a fixed column-index SELECT list in
  `get_settings()` (23 columns) — the column ORDER in the `SELECT` string, the
  numeric `row.get(N)` indices, AND the `ALTER TABLE ADD COLUMN` order in
  `schema.rs` are all coupled by position. Splitting the file must NOT reorder
  any of these three lists relative to each other, or the row indices silently
  point at the wrong column.
- `set_backend_type` has a side effect: switching to ALSA auto-sets
  `alsa_plugin` to `Hw` if unset, calling `self.set_alsa_plugin(...)` — this
  cross-setter call must still resolve once setters are split into multiple
  files (all in the same `impl AudioSettingsStore`, so it's fine as long as
  both live under the same type across files).
- `reset_all()` deliberately preserves `quality_fallback_behavior` (ADR-003) by
  reading it before the UPDATE and rewriting it after — this two-step dance
  must stay intact and is easy to accidentally "simplify away" during a split;
  flag it with a comment in `reset.rs`.
- `get_quality_fallback_behavior()` (validates against an enum-like string set)
  is called both by `reset_all()` (to save) and is itself a public getter —
  keep it in `store_core.rs` or `store_setters_*.rs`, whichever also holds
  `set_quality_fallback_behavior`, so validation logic isn't duplicated.

## Verify after split
- `cargo build -p qbz-audio` and `cargo build -p qbz-nix` (the Tauri wrapper
  crate depends on these types per the module doc).
- `cargo test -p qbz-audio settings` — all ~9 tests green, especially the
  legacy-JSON-deserialize test (schema/serde compatibility) and the
  backend-null-stays-auto test (regression test for bug #470).
- `cargo clippy -p qbz-audio` for any newly-dead `pub(crate)` visibility after
  the split.
