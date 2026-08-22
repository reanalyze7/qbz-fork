# crates/qbz-core/src/system_capabilities.rs (384 lines)

## Summary
Memory-class detection (`/proc/meminfo` parsing → `MemoryProfile` with
prefetch/buffer/cache tuning knobs for low-memory hosts like a Raspberry Pi)
plus a `MemoryPressure` snapshot used by the runtime watchdog to decide when
to evict caches. About 180 lines of real code + ~185 lines of `#[cfg(test)]`.

## Proposed split
- `system_capabilities/mod.rs` (~75 lines) — module doc, `MemoryClass` enum,
  `MemoryProfile` struct + its `from_total_kb` derivation, the `PROFILE:
  OnceLock` cache and `memory_profile()` accessor (lines 1-83, 173-199) —
  this is the "what tuning does this host get" half.
- `system_capabilities/meminfo.rs` (~70 lines) — `parse_meminfo_total_kb`,
  `parse_meminfo_available_kb`, `parse_meminfo_field_kb`,
  `detect_profile_from_meminfo`, `detect_profile` (lines 85-171) — the pure
  `/proc/meminfo`-parsing half, kept separate since it's the part with the
  richest independent test surface.
- `system_capabilities/pressure.rs` (~40 lines) — `MemoryPressure` struct,
  `pressure_from_figures`, `read_memory_pressure` (lines 113-152) — the
  watchdog-facing pressure snapshot, a distinct concern from profile
  detection even though it reads the same file.
- `system_capabilities/tests.rs` (~185 lines) — the entire `#[cfg(test)]
  mod tests` block (lines 201-384), using `super::*` (needs `use
  super::super::*` or a `use crate::system_capabilities::*;` glob once
  split, since it references items from all three modules above).

## Re-export surface
`system_capabilities/mod.rs` re-exports everything callers use today:
`MemoryClass`, `MemoryProfile`, `MemoryPressure`, `parse_meminfo_total_kb`,
`parse_meminfo_available_kb`, `pressure_from_figures`,
`read_memory_pressure`, `detect_profile_from_meminfo`, `memory_profile` —
via `pub use meminfo::*; pub use pressure::*;` so
`qbz_core::system_capabilities::memory_profile()` etc. keep working
unchanged for callers elsewhere in the crate (audio prefetch, streaming
buffer sizing).

## Coupling / watch out
- `MemoryProfile::from_total_kb` (in `mod.rs`) is called both by
  `detect_profile`/`detect_profile_from_meminfo` (in `meminfo.rs`) and
  directly by tests — keep it `pub(crate)` or `pub` so `meminfo.rs` can
  call back into `mod.rs`. This is the one two-way edge in an otherwise
  linear split (meminfo → profile; pressure is independent of profile
  except for `read_memory_pressure` calling `memory_profile()`).
- `read_memory_pressure` (arguably belongs in `pressure.rs`) calls
  `memory_profile()` (in `mod.rs`) — cross-module call, fine via
  `super::memory_profile()`.
- The test module currently is one big `mod tests` referencing all pure
  functions with zero I/O (by design, per the file's own doc comment) —
  splitting tests along the same mod.rs/meminfo/pressure lines is optional
  polish; a single `tests.rs` importing everything via glob is simplest
  and lowest-risk.

## Verify after split
- `cargo test -p qbz-core system_capabilities` — all ~20 existing tests
  green (meminfo parsing edge cases, Pi 3B/Pi Zero 2W low-memory
  classification, pressure threshold boundaries).
- `cargo build -p qbz-core` and grep for other crates importing
  `qbz_core::system_capabilities::*` to confirm no path breakage.
