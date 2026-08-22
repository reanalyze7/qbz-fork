# crates/qbzd/src/cli/transport.rs (493 lines)

## 1. Summary

Implements the 10 `qbzd` CLI transport verbs (`now play pause toggle stop
next prev seek volume mute`) as thin async wrappers around `ApiClient`
HTTP calls, plus the pure argument-parsing/response-rendering helpers each
verb needs, and a large `#[cfg(test)]` block covering the pure functions.

## 2. Proposed module split

Convert into `cli/transport/` with a `mod.rs` barrel, split along the
existing `// ==== section ====` comment markers already in the file (by
verb-group) plus a pure/IO separation within each group:

| New file | Owns | ~lines |
|---|---|---|
| `cli/transport/mod.rs` | Module decls + re-exports of the public async verb functions and the two public parse types | ~20 |
| `cli/transport/now.rs` | `now()` (async, does the HTTP call) + `render_now()` (pure formatter) | ~55 |
| `cli/transport/state_verbs.rs` | `transport_state()` (shared async helper) + `play`/`pause`/`toggle`/`stop` (all one-liners over it) | ~35 |
| `cli/transport/advance.rs` | `transport_advance()`, `render_advance()`, `next`, `prev` | ~40 |
| `cli/transport/seek.rs` | `SeekArg` enum, `parse_seek_arg`, `seek_body`, `seek()` async command | ~75 |
| `cli/transport/volume.rs` | `VolumeArg` enum, `parse_volume_arg`, `pct_to_fraction`, `fraction_to_pct`, `volume_body`, `volume()` async command | ~110 |
| `cli/transport/mute.rs` | `mute_body`, `mute()` async command | ~40 |
| `cli/transport/format.rs` | Shared pure renderers: `fmt_mmss`, `fmt_khz` | ~20 |
| `cli/transport/tests.rs` | The full `#[cfg(test)] mod tests` block (11 tests), split into per-file test modules OR kept as one file with explicit imports — see §4 | ~105 |

This mirrors the file's own section comments 1:1, so the split is
low-risk (copy each `// ==== X ====` block into its own file).

## 3. Re-export / public API surface

`cli/transport/mod.rs` re-exports the async verb functions and any type
used by the CLI dispatcher (`main.rs`/`cli/mod.rs`, wherever `qbzd`'s
arg-parsing matches subcommands to these functions):

```rust
mod advance;
mod format;
mod mute;
mod now;
mod seek;
mod state_verbs;
mod volume;
#[cfg(test)]
mod tests;

pub use advance::{next, prev};
pub use mute::mute;
pub use now::now;
pub use seek::{seek, SeekArg, parse_seek_arg, seek_body};
pub use state_verbs::{play, pause, toggle, stop};
pub use volume::{volume, VolumeArg, parse_volume_arg, pct_to_fraction, fraction_to_pct, volume_body};
```

Anything currently doing `use crate::cli::transport::{now, play, seek,
...}` keeps working unchanged since `transport` becomes a directory
module re-exporting the same names.

## 4. Tricky coupling to watch out for

- `fmt_mmss`/`fmt_khz` in `format.rs` are used by **both** `now.rs` and
  `seek.rs` — make sure both files `use super::format::{fmt_mmss,
  fmt_khz};` (or `fmt_khz` alone is only used by `now.rs`; confirm exact
  usage before assuming both need both).
- `transport_state()` and `transport_advance()` are private (`async fn`,
  no `pub`) helpers each used by exactly the 4 / 2 verb functions in
  their own file — keep them non-`pub` (module-private is fine since the
  call sites live in the same file).
- The test module currently does one flat `use super::*;` — splitting
  tests to mirror the source files means each test file needs to import
  from multiple sibling modules (e.g. `render_now` tests need `now::render_now`,
  `seek_body` tests need `seek::{SeekArg, seek_body}`). Given the tests are
  short and this is a CLI-parsing crate with low cross-file coupling,
  it's simplest to keep ONE `tests.rs` with explicit `use
  crate::cli::transport::{now::render_now, seek::*, volume::*,
  advance::render_advance, format::fmt_khz};` rather than fragmenting
  tests across 7 files — flag this as a deliberate exception to strict
  1:1 test/source mirroring, in the interest of not multiplying tiny
  test files.
- All verb functions share the exact same `Ok(v) => {...} Err(e) => {
  eprintln!("{e}"); e.exit_code() }` error-handling shape — this
  is intentional duplication (not a bug) per the file's own doc comment
  about exit codes; don't "DRY" it into a generic helper as a side effect
  of the file split — that would be an unrelated behavior-preserving
  refactor beyond scope.

## 5. What to verify after the real split

- `cargo build -p qbzd` and `cargo test -p qbzd cli::transport::` — all
  11 existing tests stay green (`seek_arg_parses_absolute_relative_and_mmss`,
  `seek_body_maps_to_legacy_position_or_additive_delta`,
  `volume_arg_parses_absolute_and_relative`,
  `cli_percent_and_api_fraction_convert_both_ways`,
  `volume_body_converts_absolute_and_delta_percent_to_fraction`,
  `mute_body_maps_bare_on_off_to_the_three_states`,
  `render_now_matches_the_documented_playing_example`,
  `render_now_stopped_state_shows_queue_count_and_no_track`,
  `render_advance_shows_landing_track_or_queue_finished`,
  `render_advance_shows_spawn_and_ack_queued`,
  `fmt_khz_rounds_only_when_not_exact`).
- Confirm the CLI dispatcher (wherever `qbzd`'s subcommand matching calls
  into `transport::now`/`play`/`seek`/etc.) still compiles — grep
  `cli::transport::` usages outside this file.
- Manual smoke test against a running `qbzd`: `qbzd now`, `qbzd play`,
  `qbzd seek +10`, `qbzd volume 50`, `qbzd mute` still print the same
  human-readable lines documented in the doc comments (02-cli-and-api.md
  §2.2).
