# crates/qbz-cmaf/src/parser.rs (310 lines)

## Summary
Pure ISO-BMFF/CMAF box-walking parser for Qobuz's proprietary FLAC-in-CMAF
container: extracts the FLAC STREAMINFO header + per-segment table from the
init segment, and per-frame crypto info (offsets/IVs) from audio segments,
plus a `#[cfg(test)]` truncation-error test.

## Proposed split
By responsibility — box-walking primitives vs the two payload parsers vs
public entry points:

- `parser/mod.rs` (~75 lines) — module doc, the two UUID constants
  (`QBZ_INIT_UUID`, `QBZ_SEGMENT_UUID`), `FLAC_MAGIC`, the public structs
  (`SegmentTableEntry`, `InitInfo`, `FrameEntry`, `SegmentCrypto`), and the
  two public entry points `parse_init_segment` / `parse_segment_crypto`
  (lines 85-120) which are thin — they call `find_uuid_box`/box-walk then
  delegate to the payload parsers.
- `parser/boxes.rs` (~55 lines) — lines 46-82 + 277-287: `find_uuid_box`,
  `find_mdat_box` (currently `#[allow(dead_code)]` — keep the attribute),
  `read_box_size`. The generic ISO-BMFF box-walking primitives, reusable and
  independent of the Qobuz-specific payload layouts.
- `parser/init_payload.rs` (~95 lines) — lines 124-217:
  `parse_init_uuid_payload` (the init segment's FLAC-header + segment-table
  extraction).
- `parser/segment_payload.rs` (~60 lines) — lines 219-275:
  `parse_segment_uuid_payload` (per-frame crypto entries extraction).
- `parser/tests.rs` (~25 lines) — lines 289-310: the
  `#[cfg(test)] mod parse_truncation_tests` block.

## Re-export surface
`parser/mod.rs` becomes the `mod parser;` target already referenced from
`crates/qbz-cmaf/src/lib.rs` (or wherever `qbz_cmaf::parser::parse_init_
segment` / `parse_segment_crypto` are called from, likely the CMAF demux
layer). Both entry-point fns and all four public structs
(`SegmentTableEntry`, `InitInfo`, `FrameEntry`, `SegmentCrypto`) must stay at
`qbz_cmaf::parser::X` — mod.rs directly defines/houses the structs and entry
points, and does `use boxes::*;`/`use init_payload::parse_init_uuid_payload;`
/`use segment_payload::parse_segment_uuid_payload;` internally (these two
payload-parse fns are private `fn`, not `pub`, so no external re-export
needed for them, only internal `use`).

## Coupling / watch out
- `find_uuid_box`/`read_box_size` in `boxes.rs` are called from BOTH
  `parse_init_segment` (mod.rs) and `parse_segment_crypto` (mod.rs) — both
  callers stay in mod.rs itself in this plan, so `boxes.rs` items just need
  `pub(super)` or `pub(crate)` visibility, not full `pub`.
- `parse_init_uuid_payload` is also called directly by the
  `#[cfg(test)]` block (`truncated_raw_len_is_error` test uses `super::*`)
  — when moving the test to `parser/tests.rs`, its `use super::*;` must
  become `use super::super::init_payload::parse_init_uuid_payload;` (or
  re-export it as `pub(super)` from init_payload.rs and adjust the `use`).
- The byte-offset arithmetic in both payload parsers is dense and easy to
  break with even a whitespace-level typo during copy — this is a pure/
  no-I/O file with real behavior risk from transcription errors, not from
  the split boundaries themselves (the boundaries are clean: no shared
  mutable state, no cross-parser calls other than box-walking).

## Verify after split
- `cargo test -p qbz-cmaf` — the existing
  `parse_truncation_tests::truncated_raw_len_is_error` test must stay green.
- `cargo check -p qbz-cmaf` / `cargo build -p qbz-cmaf`.
- If the crate has integration/fixture tests elsewhere (e.g. a real Qobuz
  CMAF sample decoded end-to-end), run those too — this parser feeds
  directly into audio decode correctness, so a subtle offset bug from the
  split would only show up in playback, not compile.
