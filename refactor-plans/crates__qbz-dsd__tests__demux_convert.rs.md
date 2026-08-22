# crates/qbz-dsd/tests/demux_convert.rs (231 lines)

## Summary
Integration test suite for the `qbz-dsd` crate: synthesizes minimal DSF/DFF files
in-memory (via `write_dsf`/`write_dff` builders) and exercises `open_dsd`,
`DsdPcmConverter`, and `DopStream` against them (parsing, PCM conversion, DoP
framing, ID3 tags, multi-channel downmix, error rejection).

## Proposed split
Split the synthetic-file builders (I/O-adjacent test fixtures) from the actual test
cases (by feature), following the pure/IO principle applied to test code:

- `demux_convert/fixtures.rs` (~90 lines) — `tmp()`, `write_dsf()`, `write_dff()`
  builders; these are pure byte-buffer construction + one `fs::write` each. Exported
  via `pub` (test-crate-internal) so the other files can call them.
- `demux_convert/dsf_tests.rs` (~95 lines) — all `#[test] fn dsf_*` and
  `fn demux_total_frames` (its only user is a DSF test): parses/converts/DSD128/8ch
  reject/5.1 downmix.
- `demux_convert/dop_tests.rs` (~20 lines) — `dop_stream_frames_and_markers`.
- `demux_convert/dff_tests.rs` (~25 lines) — `dff_parses_stereo`, `dff_dst_rejected`.
- `demux_convert/misc_tests.rs` (~15 lines) — `garbage_rejected` (or fold into
  dff_tests.rs since it's tiny — either is fine).
- Cargo integration tests can't easily be "mod.rs + submodules" the way a lib crate
  can while keeping the single `tests/demux_convert.rs` entry point; the standard
  pattern is to turn this into `tests/demux_convert/main.rs` reached via
  `tests/demux_convert.rs` becoming a 1-line `#[path] mod` shim, OR (simpler, and the
  convention many Rust projects use) rename to a directory:
  `tests/demux_convert/mod.rs` + siblings, with `tests/demux_convert.rs` reduced to
  `include!` or `mod fixtures; mod dsf_tests; ...`. Recommend the latter: keep
  `tests/demux_convert.rs` as the ~15-line entry point declaring the submodules.

## Re-export surface
`tests/demux_convert.rs` becomes the single entry file Cargo discovers (integration
test binaries are one file per `tests/*.rs`), declaring `mod fixtures; mod
dsf_tests; mod dop_tests; mod dff_tests;` — no external importers depend on this
file's internals since it's a test binary, not a lib target.

## Coupling / watch out
- `demux_total_frames` helper (line 184) is defined in the middle of the DSF tests
  and used only by `dsf_5_1_downmixes_to_stereo` — keep it next to that test or move
  to fixtures.rs if reused elsewhere.
- `CARGO_TARGET_TMPDIR` env var usage in `tmp()` is test-harness-specific; must stay
  reachable from all submodules — put it in `fixtures.rs` and `use super::fixtures::tmp;`
  elsewhere.
- Each test file needs `use qbz_dsd::{open_dsd, DsdError, DsdPcmConverter}` (and
  `DopStream` for dop_tests.rs) — repeat imports per submodule since Rust test binaries
  don't share a crate-root prelude automatically.

## Verify after split
- `cargo test -p qbz-dsd --test demux_convert` — all 9 tests must stay green,
  same names (so CI test-name filters don't break).
- Confirm `CARGO_TARGET_TMPDIR`-based temp files still get unique names across
  submodules (no filename collisions between e.g. two `dev("hw:...")` calls in
  different files — check the `tmp(name)` argument stays unique repo-wide).
