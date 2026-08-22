# crates/qbz-audio/src/device_filter.rs (224 lines)

## 1. Summary

Pure string/dedup logic that cleans up CPAL's raw output-device
enumeration on Linux/ALSA: identifies the "discard" null sink, dedups
device entries that share a display name (keeping the best-ranked
re-openable id), and orders real outputs before the discard sink — with a
thorough unit-test suite (6 tests) covering each behavior.

## 2. Proposed module split

The file is small (224 lines) but crosses the 130-line line right at the
test module boundary. Split pure logic from tests:

| New file | Owns | ~lines |
|---|---|---|
| `device_filter/mod.rs` | `is_discard_sink`, `dedup_key`, `id_rank`, `retain_real_outputs` — all pure functions, plus the big module doc comment explaining the ALSA PCM dedup problem | ~130 |
| `device_filter/tests.rs` | The `#[cfg(test)] mod tests` block (the `run()` test helper + 6 tests) | ~95 |

This is the minimal split that satisfies the 130-line rule without
breaking apart tightly-coupled pure functions that call each other
directly (`retain_real_outputs` calls `dedup_key` and `id_rank`
internally — keeping all four in one file avoids adding `pub(crate)`
visibility just to let a test-only split call them, since the logic
functions stay together and only tests move out).

If `device_filter/mod.rs` is still a few lines over 130 after actually
moving the doc comment + functions (the estimate above is close to the
line), a second option is to further split:
- `device_filter/dedup.rs` — `dedup_key`, `id_rank` (the two small
  ranking helpers)
- `device_filter/mod.rs` — `is_discard_sink`, `retain_real_outputs` (the
  two public entry points) + module doc comment

## 3. Re-export / public API surface

`device_filter/mod.rs` stays the import path everyone uses today —
`qbz_audio::device_filter::{is_discard_sink, retain_real_outputs}`. If
the dedup-helpers-in-their-own-file option is taken, `mod.rs` adds:

```rust
mod dedup;
use dedup::{dedup_key, id_rank};

pub fn is_discard_sink(display: &str) -> bool { ... }
pub fn retain_real_outputs<T>(...) -> Vec<T> { ... }
```

`is_discard_sink` and `retain_real_outputs` are the only two functions
called from outside this module (the ALSA/PipeWire backends and the
`output_sinks` diagnostic per the module doc comment) — both stay `pub`
at the same path.

## 4. Tricky coupling to watch out for

- `retain_real_outputs<T>` is generic over the caller's row type via two
  closures (`id_of`, `display_of`) — this is the whole point of the
  module (shared between `AudioDevice` enumeration and `OutputSinkInfo`
  diagnostic per the doc comment), so don't change its signature while
  splitting.
- `id_rank`'s match arms encode a very specific, deliberately-ordered
  preference ladder (default/pipewire/pulse/sysdefault first, then
  `alsa_output.`, then `front:CARD=`, etc., down to `surround*`/`plug:`/
  `dmix`/`dsnoop`/`route` last) — if this becomes its own file, preserve
  the exact match order and the explanatory comment above it verbatim;
  reordering arms changes behavior for entries matching multiple
  prefixes... actually since it's `if`/`else if` chains it's order
  sensitive, double check nothing subtly relies on match fallthrough
  order when relocating.
- Tests construct `(String, String)` tuples via a local `run()` helper
  that calls `retain_real_outputs` with closures over tuple fields —
  `tests.rs` needs `use super::retain_real_outputs;` (or
  `use crate::device_filter::retain_real_outputs;` if it becomes a
  standalone test file outside the module tree — prefer `super::` to
  keep it as an inline child module of `device_filter`).

## 5. What to verify after the real split

- `cargo build -p qbz-audio` and `cargo test -p qbz-audio device_filter::`
  — all 6 tests stay green (`discard_sink_sorted_to_end`,
  `collapses_plugin_wrappers_to_one_per_output`,
  `keeps_genuinely_distinct_outputs`, `passes_pipewire_node_names_through`,
  `first_seen_order_is_preserved`, `drops_blank_displays`).
- Grep for `device_filter::` usages in the ALSA/PipeWire audio backends
  and the `output_sinks` diagnostic command to confirm import paths
  didn't change.
- No runtime smoke test strictly required (pure logic, fully covered by
  unit tests), but a manual "list audio output devices" check in the app
  is cheap insurance given this code directly affects a user-facing
  picker.
