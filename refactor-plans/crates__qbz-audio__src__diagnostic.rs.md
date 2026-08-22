# crates/qbz-audio/src/diagnostic.rs (205 lines)

## 1. Summary
Lock-free audio bit-depth diagnostic: `AudioDiagnostic` accumulates an
OR-mask of samples (converted to i32) across atomics to infer effective bit
depth without storing samples, `BitDepthResult` is the serializable result
returned to the frontend, and `DiagnosticSource<S>` is a transparent
`rodio::Source` wrapper tapping every sample for capture (works for both
rodio/CPAL and ALSA-Direct playback paths).

## 2. Proposed module split
Three clearly-separated concerns, matching the file's own section banners:

| New file | Owns | ~lines |
|---|---|---|
| `diagnostic/mod.rs` | Module decls + re-exports; module doc comment | ~15 |
| `diagnostic/state.rs` | `AudioDiagnostic` struct + its `impl` (`new`, `start_capture`, `is_capturing`, `push_sample`, `stop_and_analyze`) + `impl Default` — the atomic-accumulator core | ~100 |
| `diagnostic/result.rs` | `BitDepthResult` struct (the serde-serializable output type) | ~20 |
| `diagnostic/source.rs` | `DiagnosticSource<S>` + its `impl` (`new`) + `impl Iterator` + `impl Source` — the transparent tap wrapper | ~70 |

This is barely over the line (205 vs 130) so a 3-way split is enough; no
further sub-splitting needed per file.

## 3. Re-export / public API surface
`diagnostic/mod.rs` re-exports the full current public surface:

```rust
mod result;
mod source;
mod state;

pub use result::BitDepthResult;
pub use source::DiagnosticSource;
pub use state::AudioDiagnostic;
```

Every caller doing `use qbz_audio::diagnostic::{AudioDiagnostic,
DiagnosticSource, BitDepthResult};` (the rodio/CPAL backend and the ALSA
Direct backend, per the module doc's "works for both... paths" claim) keeps
working unchanged.

## 4. Tricky coupling / shared-state to watch out for
- `DiagnosticSource<S>` holds an `AudioDiagnostic` by value (it's `Clone` —
  cheap, since it's all `Arc<Atomic*>` fields) — `source.rs` needs
  `use super::state::AudioDiagnostic;`. No shared mutable state beyond the
  atomics themselves, which is the whole point of the design (lock-free) —
  so the split introduces no new synchronization concerns.
- `BitDepthResult` is `#[serde(rename_all = "camelCase")]` — this attribute
  and the exact field names must travel unchanged into `result.rs`, since
  it's almost certainly consumed by a frontend/diagnostics UI expecting
  camelCase JSON keys.
- `stop_and_analyze` computes `trailing_zeros`/`effective_bits`/
  `duration_secs` purely from the four atomics (`or_mask`, `sample_count`,
  `sample_rate`, `channels`) — this method belongs in `state.rs` alongside
  the atomics it reads, not split out into a separate "pure computation"
  file, since the atomics are private fields only accessible from `impl
  AudioDiagnostic` itself.
- Both `alsa_direct.rs` and the rodio/CPAL backend construct a
  `DiagnosticSource` wrapping their own inner `Source` type — confirm via
  grep that both call sites only ever reference `AudioDiagnostic`/
  `DiagnosticSource` by their re-exported paths, never `diagnostic::state::`
  or `diagnostic::source::` directly.

## 5. What to verify after the real split
- `cargo build -p qbz-audio` and `cargo test -p qbz-audio` (check for any
  diagnostic-specific unit tests elsewhere in the crate, e.g.
  `alsa_direct.rs` or a dedicated test file, that exercise
  `start_capture`/`push_sample`/`stop_and_analyze`).
- Grep the workspace for `diagnostic::` to confirm both playback backends
  (rodio/CPAL and ALSA Direct) still resolve their imports.
- Smoke-test: trigger the bit-depth diagnostic capture in the running app
  (likely a debug/diagnostics settings action) on both playback backends if
  feasible, and confirm the reported `effectiveBits`/`sampleRate`/`channels`
  match expectations for a known-format test file.
