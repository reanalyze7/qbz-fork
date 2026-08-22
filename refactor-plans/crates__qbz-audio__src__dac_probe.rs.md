# crates/qbz-audio/src/dac_probe.rs (205 lines)

## Summary
Read-only DAC hardware-state probe (HiFi wizard): parses ALSA
`hw_params`/PipeWire `pw-dump` output to report the DAC's actually
negotiated sample rate, independent of what QBZ requested.

## Proposed split
This file is already fairly small (205 lines, ~60 of which are tests) and
cleanly divides into pure-parsing vs IO-shelling-out:

- `dac_probe/mod.rs` (~40 lines) — module doc (lines 1-12), `NegotiatedRate`
  struct (lines 16-28), `pub use` re-exports.
- `dac_probe/parse.rs` (~65 lines) — the two pure parsing functions:
  `parse_hw_params` (lines 34-63) and `parse_alsa_card_for_node` (lines
  67-94). No I/O, directly unit-testable against fixtures — matches the
  file's own doc-comment framing ("Pure ... so it is unit-testable against
  captured fixtures").
- `dac_probe/probe.rs` (~50 lines) — the IO-performing functions:
  `alsa_card_for_node` (shells to `pw-dump`, lines 97-104),
  `read_hw_params_for_card` (reads `/proc/asound/...`, lines 106-119),
  `negotiated_stream_rate` (lines 121-126), `negotiated_active_rate` (lines
  128-142). Depends on `parse::{parse_hw_params, parse_alsa_card_for_node}`.
- `dac_probe/tests.rs` (as `#[cfg(test)] mod tests` at the bottom of
  `dac_probe/parse.rs`, since all 4 existing tests exercise only the pure
  parsers, not the IO functions) — lines 144-205 unchanged.

Given the small total size, an alternative lighter-weight split that also
satisfies the 130-line rule: just two files (`dac_probe.rs` keeping
`NegotiatedRate` + the two pure parsers + tests ~170 lines, and a new
`dac_probe_io.rs` sibling file with the four IO functions ~55 lines) — no
directory needed. Prefer the flat two-file split unless this crate already
uses `foo/mod.rs` directories elsewhere (check sibling files like
`backend.rs`'s existing `backend/` directory precedent noted in this
crate's other plans before deciding).

## Re-export surface
Either `dac_probe/mod.rs` (if directory) or `dac_probe.rs` itself (if flat,
with a `mod dac_probe_io; pub use dac_probe_io::*;` at its top) stays the
public surface. The crate's `lib.rs` line `pub mod dac_probe;` needs no
change either way. `qbz_audio::dac_probe::{NegotiatedRate,
negotiated_stream_rate, negotiated_active_rate, parse_hw_params,
parse_alsa_card_for_node}` all resolve unchanged.

## Coupling / watch out
- `read_hw_params_for_card` (IO) calls `parse_hw_params` (pure) in a loop
  over candidate PCM indices (0..4) — straightforward cross-file call once
  both are `pub(crate)`/`pub(super)` visible; `parse_hw_params` is
  currently `pub fn` already (used directly by callers/tests), so no
  visibility change needed there.
- `alsa_card_for_node` (IO, shells to `pw-dump`) calls
  `parse_alsa_card_for_node` (pure) — same straightforward split, and
  `parse_alsa_card_for_node` is also already `pub fn`.
- `negotiated_active_rate`'s "scan every card 0..16, return first open one"
  approach is DAC-agnostic by design (per its doc comment) — don't
  "optimize" this into something DAC-specific when moving it; the
  comment explaining why is important context to carry over.
- No shared mutable state anywhere in this file — every function is either
  pure or a single self-contained IO call. Lowest-coupling file in this
  batch.

## Verify after split
- `cargo build -p qbz-audio`.
- `cargo test -p qbz-audio dac_probe::` — all 4 existing tests
  (`parses_active_hw_params`, `idle_or_empty_yields_none`,
  `resolves_card_from_node_name`, `resolves_card_when_only_string_prop_present`)
  must still pass unchanged.
- `cargo clippy -p qbz-audio`.
- Smoke-test importers: `grep -rn "dac_probe::" crates` — confirm the HiFi
  setup-wizard code (Slice 8b/N6) that calls
  `negotiated_stream_rate`/`negotiated_active_rate` still compiles.
