# crates/qbz-library/src/scan.rs (388 lines)

## Summary
Frontend-agnostic, progress-emitting library scan: walks enabled library
folders (or a specified subset), processes CUE sheets and audio files
(sidecar tag overrides, embedded/folder artwork), inserts tracks, and cleans
up rows for files that no longer exist — driven by an `on_event` callback and
an `AtomicBool` cancel flag so any frontend (Slint, TUI) can show progress.

## Proposed split

- `scan/mod.rs` (~30 lines) — module doc, `mod` declarations, `pub use`
  re-exports of `ScanEvent` and `scan_with_progress`.
- `scan/event.rs` (~25 lines) — the `ScanEvent` enum alone.
- `scan/helpers.rs` (~35 lines) — `normalize_path`, `now_secs`,
  `apply_sidecar_override` — small pure/near-pure helpers used by both the
  CUE and audio-file paths.
- `scan/cue.rs` (~55 lines) — `process_cue_file` (CUE parsing -> virtual
  tracks -> sidecar override -> artwork -> insert).
- `scan/orchestrate.rs` (~230 lines) — `scan_with_progress` itself. Still
  over budget as one function; the real fix is extracting its three phases
  into named functions within this file (or split across two files):
  - `scan_targets` / network-refresh preamble (~40 lines: resolving
    `targets` from `folder_ids`, the per-folder network-detection refresh
    loop) -> could become `scan/targets.rs`.
  - the CUE-then-audio-files per-folder loop (the bulk of the function,
    ~150 lines) -> stays in `orchestrate.rs` as the core loop, calling into
    `cue.rs`'s `process_cue_file` and a new `scan/audio_file.rs` for the
    per-audio-file block (artwork resolution + `db.insert_track`, ~60 lines
    of the loop body).
  - the cleanup phase (missing-file deletion + `last_scan` stamping, ~55
    lines) -> `scan/cleanup.rs`.

Concretely:
- `scan/targets.rs` (~50 lines) — folder-set resolution + network-status
  refresh, called once at the top of `scan_with_progress`.
- `scan/audio_file.rs` (~90 lines) — the per-audio-file processing block
  (metadata extraction, sidecar override, artwork fallback chain, insert)
  extracted into a `fn process_audio_file(...)` taking the accumulators it
  needs by `&mut`.
- `scan/cleanup.rs` (~70 lines) — the missing-file cleanup phase
  (`folder_prefix`, `unavailable_prefixes`/`under_unavailable` network-down
  guard, the `missing` id collection + chunked delete) and the `last_scan`
  stamping loop.
- `scan/orchestrate.rs` (~90 lines) — `scan_with_progress` reduced to: resolve
  targets (`targets.rs`), emit `Started`, loop over folders calling
  `cue.rs`/`audio_file.rs` per file with cancel checks and `on_event` calls,
  then call `cleanup.rs`, then emit `Finished`.

## Re-export surface
`scan/mod.rs` re-exports `ScanEvent` and `scan_with_progress` — the only two
items the doc comment implies are consumed externally (by Slint's scan
trigger and the TUI's scan screen) — at `qbz_library::scan::*`, unchanged.

## Tricky coupling / watch out
- `scan_with_progress`'s local accumulators (`all_errors`, `sidecar_cache`,
  `total`, `processed`, `folder_artwork_cache`, `cue_audio_files`) are
  threaded through both the CUE loop and the audio-file loop within one
  function today — extracting `process_audio_file` requires passing several
  of these by `&mut` (or bundling into a small `ScanAccumulator` struct);
  don't accidentally reset a cache per-folder that was meant to persist
  across the whole scan (check which caches are per-folder vs. per-scan:
  `folder_artwork_cache` is per-folder, `sidecar_cache` spans the WHOLE scan).
- The cancel check (`cancel.load(Ordering::Relaxed)`) happens at every file
  boundary in BOTH the CUE loop and the audio-file loop, each with an early
  `return Ok(())` after emitting `Finished{Cancelled}` — if these loops move
  to different files, keep the identical early-return-with-event pattern in
  both.
- The network-down guard (`under_unavailable`) in the cleanup phase reads
  `db.get_folders_with_metadata()` a SECOND time (already fetched once at the
  top as `all`) — this is intentional (folder metadata may have changed
  during the scan via the network-refresh loop) — don't "optimize" it into a
  single fetch during the split.
- `single` (whether this is a full scan or a folder-subset scan) affects both
  which folders get their network status refreshed implicitly (all `targets`
  either way) and which cleanup path runs (`single` -> prefix-filtered
  missing-file check) — keep this one bool threaded consistently if
  `cleanup.rs` and `orchestrate.rs` are separate files.

## What to verify after the real split
- `cargo test -p qbz-library` (no `#[cfg(test)]` block exists in this file
  currently; verify no regressions in whatever integration tests exercise
  scanning, e.g. under `crates/qbz-library/tests/`).
- `cargo build -p qbz-library` and grep for `qbz_library::scan::` /
  `scan_with_progress` call sites (Slint's scan command handler, `qbzd`'s TUI
  network/library screens, and the CLI `qbzd scan` command if one exists).
- Manual smoke test via the `run` skill: trigger a full library scan and a
  single-folder rescan, confirm progress events still update the UI, cancel
  mid-scan, and delete a file then rescan to confirm cleanup still removes
  its row (and that a simulated network-down folder is NOT wiped).
