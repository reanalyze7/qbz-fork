# crates/qbz/src/diagnostics.rs (1080 lines)

## Summary
Diagnostics panel controller (Settings > Developer): wires `DiagnosticsState`
to refresh/export, builds a full markdown diagnostics report (system +
audio + graphics + env + playback) used by both the panel export and the
log-viewer's shareable bundle, plus row-builders for each panel section and
a UUID/hex redaction helper for the exported text.

## Proposed split
950 lines over budget — this is the second-largest file in the batch, needs
a real by-domain split, not just a light trim.

- `diagnostics/mod.rs` (~65 lines) — `Runtime` type alias, `DiagController`
  struct, `install()` (registers the two callbacks). The public entry point.
- `diagnostics/controller.rs` (~150 lines) — `impl DiagController`:
  `refresh`, `refresh_async` (the settings-read + core-snapshot + row-build +
  one-event-loop-hop push), and `export_clipboard`. This is the stateful
  orchestration core; keep it together since `refresh_async` is one
  sequential pipeline that's easy to break apart wrongly.
- `diagnostics/report.rs` (~330 lines) — `md_line` + `build_full_report`:
  the full markdown report assembly (System/Audio/Graphics/Environment/
  Playback sections). This is the single biggest function (~310 lines) —
  it's already organized as clearly-commented `## Section` blocks; if still
  over budget, further split into `report/system.rs`, `report/audio.rs`,
  `report/graphics.rs`, `report/env.rs`, `report/playback.rs`, each
  exposing a `write_x_section(&mut String, ...)` fn that `report.rs` calls
  in sequence — mirrors the row-builder split below one-for-one.
- `diagnostics/rows.rs` (~230 lines) — `row`, `yn`, `opt`, `match_status`,
  `trim_khz`, `build_system_rows`, `build_audio_rows`, `build_graphics_rows`,
  `build_env_rows`, `build_playback_rows` — the Slint `DiagRow` builders
  (1:1 with the Svelte row builders per the file's own comment). If over
  budget, split into `rows/helpers.rs` (row/yn/opt/match_status/trim_khz)
  + `rows/builders.rs` (the five `build_*_rows` functions).
- `diagnostics/output_sinks.rs` (~100 lines) — `collect_output_sinks`,
  `active_sink_format`: the CPAL/pactl live-audio-device probing (the one
  piece that shells out to `pactl` and must run inside `spawn_blocking`).
- `diagnostics/export_json.rs` (~25 lines) — `build_playback_json`: the
  camelCase JSON export shape (small but semantically distinct: the JSON
  export contract vs. the human-readable markdown report).
- `diagnostics/redact.rs` (~60 lines) — `redact_id_like`, `uuid_at`: the
  UUID/long-hex redaction helpers, pure string logic, easy to test alone.
- `diagnostics/tests.rs` (~30 lines, `#[cfg(test)] mod tests`) — the three
  existing tests (`redacts_uuid_and_long_hex`, `leaves_short_hex_alone`,
  `match_status_rules`), included via `#[cfg(test)] mod tests;`.

## Re-export surface
`diagnostics/mod.rs` re-exports `pub fn install(...)` (called from shell
setup) and `pub async fn build_full_report(...)` — the second one is
IMPORTANT: it's called directly by `crate::log_viewer::build_share_text`
(see `crates/qbz/src/log_viewer.rs`), not just internally, so it must stay
reachable at `crate::diagnostics::build_full_report` after the split (e.g.
`pub use report::build_full_report;`).

## Coupling / watch out
- **Cross-file dependency with `log_viewer.rs`** (also in this refactor
  batch, possibly a different agent's slice): `log_viewer.rs`'s
  `build_share_text` calls `crate::diagnostics::build_full_report(runtime)`
  directly. Whichever module ends up owning `build_full_report` after the
  split, the path `crate::diagnostics::build_full_report` must keep
  resolving — re-export it from `diagnostics/mod.rs` even if the body lives
  in `diagnostics/report.rs`.
- `refresh_async` and `build_full_report` are near-duplicate pipelines (both
  do: blocking settings-read -> `spawn_blocking` -> async core snapshot ->
  build outputs) that independently re-implement the same three-store read +
  `collect_output_sinks` call. Flag this as a candidate to share a private
  `gather_diagnostics_inputs()` helper in `controller.rs` or a new
  `diagnostics/collect.rs`, called by both `refresh_async` (feeds rows) and
  `build_full_report` (feeds markdown) — but this is a behavior-preserving
  refactor beyond "just split files", so only do it if the split naturally
  makes the duplication obvious; don't force it if it risks subtle drift
  between the panel and the exported report.
- `DiagController::export`'s cached `Arc<Mutex<Option<Value>>>` snapshot is
  built in `refresh_async` and consumed in `export_clipboard` — both must
  stay in `controller.rs` together since they share that field.
- `redact_id_like`/`uuid_at` are UTF-8-safe by operating on `Vec<char>` —
  preserve that; do not "optimize" to byte-slicing during the move.

## Verify after split
- `cargo test -p qbz diagnostics` (all 3 existing tests green).
- `cargo check -p qbz`
- Manual/smoke test: open Settings > Developer > Diagnostics, hit Refresh
  and Export (clipboard), confirm all 5 row sections populate and the
  exported JSON has the expected camelCase keys.
- Confirm `crate::diagnostics::build_full_report` is still callable from
  `log_viewer.rs`'s upload/copy-bundle paths (build the crate, don't just
  check `diagnostics` in isolation).
