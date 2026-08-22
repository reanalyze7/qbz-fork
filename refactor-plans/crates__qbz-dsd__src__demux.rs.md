# crates/qbz-dsd/src/demux.rs (463 lines)

## Summary
DSF and DFF (DSDIFF) container demuxers: shared error/tag/stream-info types
plus the `DsdDemuxer` trait, a format-sniffing `open_dsd()` entry point, and
two full reader implementations (`DsfReader`, `DffReader`) each handling
their container's distinct chunk layout and bit order.

## Proposed split
By domain — shared types, then one file per concrete format:

- `demux/mod.rs` (~110 lines) — module doc, `DsdError`, `DsdTags`,
  `DsdStreamInfo` (+ `duration_secs()`), the `DsdDemuxer` trait,
  `VALID_RATES`, `open_dsd()` (the format-sniffing entry point that
  constructs whichever reader), and the shared byte-reading helpers
  (`read_u32_le`, `read_u64_le`, `read_u64_be`, `read_id`, `validate_rate`,
  `read_id3_tags`) since both readers use them.
- `demux/dsf.rs` (~135 lines) — the "DSF" section: `DsfReader` struct +
  its `open()` + its `DsdDemuxer` impl (`info()`, `read_planar()`).
- `demux/dff.rs` (~170 lines) — the "DFF (DSDIFF)" section: `DffReader`
  struct + its `open()` (the chunk-walking state machine: FRM8/PROP/SND/
  FS/CHNL/CMPR/DSD/DST/ID3 handling) + its `DsdDemuxer` impl.

Note: there is no `#[cfg(test)]` block in this file — per the task note,
`crates/qbz-dsd/tests/demux_convert.rs` is a SEPARATE integration-test file
already covered by another agent's plan
(`refactor-plans/crates__qbz-dsd__tests__demux_convert.rs.md` exists in this
directory) — no test code moves as part of this split.

## Re-export surface
`demux/mod.rs` is the target of the existing `mod demux;` (or `pub mod
demux;`) declaration in `crates/qbz-dsd/src/lib.rs`. Every currently-`pub`
symbol — `DsdError`, `DsdTags`, `DsdStreamInfo`, `DsdDemuxer`, `open_dsd` —
is already declared in what becomes `mod.rs`, so NO re-export shuffling is
needed for the public API; only the two concrete reader structs
(`DsfReader`, `DffReader`, both currently private/non-`pub`) move into their
own files and need `pub(super)` or `pub(crate)` visibility so `mod.rs`'s
`open_dsd()` can construct them via `dsf::DsfReader::open(file)` /
`dff::DffReader::open(file)`.

## Coupling / watch out
- `open_dsd()` (in `mod.rs`) directly constructs `Box::new(DsfReader::open(file)?)`
  / `Box::new(DffReader::open(file)?)` — after the split this becomes
  `Box::new(dsf::DsfReader::open(file)?)` / `Box::new(dff::DffReader::open(file)?)`,
  requiring `mod dsf; mod dff;` declarations in `mod.rs` and the reader
  structs to be at least `pub(crate)` visible.
- Both readers call the shared helpers (`read_u32_le`, `read_u64_le`,
  `read_u64_be`, `read_id`, `validate_rate`, `read_id3_tags`) — these must
  become `pub(super)` (or `pub(crate)`) in `mod.rs` so `dsf.rs`/`dff.rs` can
  reach them via `use super::*;`.
- `DsdTags`/`DsdStreamInfo` are constructed inside both readers' `open()` —
  no special handling needed since they're already `pub` in `mod.rs`, just
  `use super::{DsdTags, DsdStreamInfo, DsdError};` in each reader file.
- The DFF reader's chunk-walking loop is the most intricate part of the
  file (nested PROP/SND sub-chunk scanning with even-byte padding) — it's
  fully self-contained within `DffReader::open()`, so it can move as one
  unit without internal restructuring.
- `id3::Tag::read_from2` / `id3::TagLike` usage in `read_id3_tags` is the
  only external-crate-heavy helper — keep it in `mod.rs` since both formats
  depend on it identically (DSF via `metadata_ptr`, DFF via the "ID3 "
  chunk).

## Verify after split
- `cargo check -p qbz-dsd` and `cargo build -p qbz-dsd`.
- `cargo test -p qbz-dsd` — specifically re-run the sibling integration
  test `crates/qbz-dsd/tests/demux_convert.rs` (covered by a separate
  agent's plan) since it almost certainly calls `open_dsd()` end-to-end
  against real/fixture `.dsf`/`.dff` files and would catch any visibility
  or wiring mistake in this split immediately.
- No unit tests exist in-file today to lose; if any get added later they
  should land in `demux/dsf.rs`/`demux/dff.rs` next to the reader they test
  rather than back in `mod.rs`.
